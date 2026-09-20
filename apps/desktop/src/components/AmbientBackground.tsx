import { useEffect, useRef } from "react";

const vertexShader = `
  attribute vec2 position;
  void main() { gl_Position = vec4(position, 0.0, 1.0); }
`;

const fragmentShader = `
  precision mediump float;
  uniform vec2 resolution;
  uniform vec2 pointer;
  uniform float time;
  uniform float lightTheme;

  float orb(vec2 uv, vec2 center, float radius) {
    return 1.0 - smoothstep(0.0, radius, length(uv - center));
  }

  void main() {
    vec2 uv = (gl_FragCoord.xy * 2.0 - resolution.xy) / min(resolution.x, resolution.y);
    vec2 cursor = (pointer * 2.0 - 1.0) * vec2(resolution.x / resolution.y, 1.0);
    float drift = time * 0.10;
    float a = orb(uv, vec2(sin(drift) * .55 - .38, cos(drift * .8) * .25 + .2), 1.35);
    float b = orb(uv, vec2(cos(drift * .75) * .5 + .62, sin(drift) * .32 - .28), 1.18);
    float c = orb(uv, cursor * .44, .72);
    float lattice = pow(max(0.0, sin((uv.x + uv.y) * 3.2 + drift) * .5 + .5), 7.0) * .035;
    vec3 darkBase = vec3(.025, .032, .055);
    vec3 lightBase = vec3(.93, .945, .97);
    vec3 base = mix(darkBase, lightBase, lightTheme);
    vec3 violet = mix(vec3(.22, .08, .56), vec3(.48, .34, .88), lightTheme);
    vec3 cyan = mix(vec3(.02, .30, .43), vec3(.08, .45, .62), lightTheme);
    vec3 amber = vec3(.91, .42, .12);
    vec3 color = base + violet * a * .48 + cyan * b * .34 + amber * c * .10 + lattice;
    float vignette = smoothstep(1.85, .25, length(uv));
    color *= mix(.52, 1.0, vignette);
    gl_FragColor = vec4(color, 1.0);
  }
`;

function shader(gl: WebGLRenderingContext, type: number, source: string) {
  const value = gl.createShader(type);
  if (!value) return;
  gl.shaderSource(value, source);
  gl.compileShader(value);
  if (!gl.getShaderParameter(value, gl.COMPILE_STATUS)) {
    gl.deleteShader(value);
    return;
  }
  return value;
}

export function AmbientBackground() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const gl = canvas?.getContext("webgl", { alpha: false, antialias: false, powerPreference: "low-power" });
    if (!canvas || !gl) return;
    const vertex = shader(gl, gl.VERTEX_SHADER, vertexShader);
    const fragment = shader(gl, gl.FRAGMENT_SHADER, fragmentShader);
    if (!vertex || !fragment) return;
    const program = gl.createProgram();
    if (!program) return;
    gl.attachShader(program, vertex);
    gl.attachShader(program, fragment);
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) return;
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
    gl.useProgram(program);
    const position = gl.getAttribLocation(program, "position");
    gl.enableVertexAttribArray(position);
    gl.vertexAttribPointer(position, 2, gl.FLOAT, false, 0, 0);
    const resolution = gl.getUniformLocation(program, "resolution");
    const pointerLocation = gl.getUniformLocation(program, "pointer");
    const time = gl.getUniformLocation(program, "time");
    const lightTheme = gl.getUniformLocation(program, "lightTheme");
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const pointer = { x: 0.62, y: 0.42 };
    let frame = 0;
    let active = true;
    let lastDraw = 0;

    const resize = () => {
      const ratio = Math.min(window.devicePixelRatio || 1, 1.5);
      const width = Math.max(1, Math.floor(canvas.clientWidth * ratio));
      const height = Math.max(1, Math.floor(canvas.clientHeight * ratio));
      if (canvas.width !== width || canvas.height !== height) {
        canvas.width = width;
        canvas.height = height;
        gl.viewport(0, 0, width, height);
      }
    };
    const move = (event: PointerEvent) => {
      const bounds = canvas.getBoundingClientRect();
      pointer.x = (event.clientX - bounds.left) / bounds.width;
      pointer.y = 1 - (event.clientY - bounds.top) / bounds.height;
    };
    const draw = (now: number) => {
      if (!reduced && now - lastDraw < 33) {
        frame = requestAnimationFrame(draw);
        return;
      }
      lastDraw = now;
      gl.uniform2f(resolution, canvas.width, canvas.height);
      gl.uniform2f(pointerLocation, pointer.x, pointer.y);
      gl.uniform1f(time, reduced ? 0 : now / 1000);
      gl.uniform1f(lightTheme, document.documentElement.dataset.theme === "light" ? 1 : 0);
      gl.drawArrays(gl.TRIANGLES, 0, 3);
      if (active && !reduced) frame = requestAnimationFrame(draw);
    };
    const resizeObserver = new ResizeObserver(() => { resize(); if (reduced) frame = requestAnimationFrame(draw); });
    const visibility = () => {
      active = !document.hidden;
      cancelAnimationFrame(frame);
      if (active) frame = requestAnimationFrame(draw);
    };
    const themeObserver = new MutationObserver(() => { if (reduced) frame = requestAnimationFrame(draw); });
    themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });
    window.addEventListener("pointermove", move, { passive: true });
    document.addEventListener("visibilitychange", visibility);
    resizeObserver.observe(canvas);
    resize();
    frame = requestAnimationFrame(draw);
    return () => {
      active = false;
      cancelAnimationFrame(frame);
      window.removeEventListener("pointermove", move);
      document.removeEventListener("visibilitychange", visibility);
      resizeObserver.disconnect();
      themeObserver.disconnect();
      gl.deleteProgram(program);
      gl.deleteShader(vertex);
      gl.deleteShader(fragment);
      gl.deleteBuffer(buffer);
    };
  }, []);

  return <canvas ref={canvasRef} className="ambient-canvas" aria-hidden="true" />;
}
