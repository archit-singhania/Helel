import { useRef, useState } from "react";
import { workspaceApi } from "../api";
import { Icon } from "../App";

type Recorder = { context: AudioContext; stream: MediaStream; source: MediaStreamAudioSourceNode; processor: ScriptProcessorNode; chunks: Float32Array[]; samples: number };

export function VoiceInput({ program, model, onTranscript }: { program: string; model: string; onTranscript: (text: string) => void }) {
  const recorder = useRef<Recorder | undefined>(undefined);
  const timer = useRef<number | undefined>(undefined);
  const [state, setState] = useState<"idle" | "recording" | "transcribing">("idle");

  const stop = async () => {
    window.clearTimeout(timer.current);
    const active = recorder.current;
    recorder.current = undefined;
    if (!active) return;
    active.processor.disconnect(); active.source.disconnect(); active.stream.getTracks().forEach((track) => track.stop());
    const sampleRate = active.context.sampleRate;
    await active.context.close();
    if (!program.trim() || !model.trim()) { setState("idle"); window.alert("Configure a local whisper.cpp executable and model in Settings first."); return; }
    const approved = window.confirm(`Transcribe this recording locally?\n\nExecutable: ${program}\nModel: ${model}\n\nAudio is deleted immediately after transcription.`);
    if (!approved) { setState("idle"); return; }
    setState("transcribing");
    try {
      const wav = encodeWav(active.chunks, active.samples, sampleRate);
      const transcript = await workspaceApi.transcribeVoice(Array.from(wav), program.trim(), model.trim(), true);
      if (transcript) onTranscript(transcript);
    } catch (error) { window.alert(String(error)); }
    finally { setState("idle"); }
  };

  const start = async () => {
    if (state === "recording") { await stop(); return; }
    if (state !== "idle") return;
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: { channelCount: 1, echoCancellation: true, noiseSuppression: true, autoGainControl: true } });
      const context = new AudioContext();
      const source = context.createMediaStreamSource(stream);
      const processor = context.createScriptProcessor(4096, 1, 1);
      const silent = context.createGain(); silent.gain.value = 0;
      const value: Recorder = { context, stream, source, processor, chunks: [], samples: 0 };
      processor.onaudioprocess = (event) => { const chunk = new Float32Array(event.inputBuffer.getChannelData(0)); value.chunks.push(chunk); value.samples += chunk.length; };
      source.connect(processor); processor.connect(silent); silent.connect(context.destination);
      recorder.current = value; setState("recording"); timer.current = window.setTimeout(() => void stop(), 30_000);
    } catch (error) { window.alert(`Microphone unavailable: ${String(error)}`); }
  };

  return <button type="button" className={`voice-button ${state}`} onClick={() => void start()} disabled={state === "transcribing"} title={state === "recording" ? "Stop recording" : "Dictate task locally"} aria-label={state === "recording" ? "Stop voice recording" : "Dictate agent objective locally"}><Icon name="mic" /><span>{state === "recording" ? "Listening" : state === "transcribing" ? "Transcribing" : "Voice"}</span></button>;
}

function encodeWav(chunks: Float32Array[], sampleCount: number, sourceRate: number) {
  const source = new Float32Array(sampleCount); let offset = 0;
  for (const chunk of chunks) { source.set(chunk, offset); offset += chunk.length; }
  const targetRate = 16_000; const ratio = sourceRate / targetRate; const length = Math.floor(source.length / ratio);
  const buffer = new ArrayBuffer(44 + length * 2); const view = new DataView(buffer);
  const write = (at: number, text: string) => { for (let index = 0; index < text.length; index += 1) view.setUint8(at + index, text.charCodeAt(index)); };
  write(0, "RIFF"); view.setUint32(4, 36 + length * 2, true); write(8, "WAVE"); write(12, "fmt "); view.setUint32(16, 16, true); view.setUint16(20, 1, true); view.setUint16(22, 1, true); view.setUint32(24, targetRate, true); view.setUint32(28, targetRate * 2, true); view.setUint16(32, 2, true); view.setUint16(34, 16, true); write(36, "data"); view.setUint32(40, length * 2, true);
  for (let index = 0; index < length; index += 1) { const position = index * ratio; const before = Math.floor(position); const fraction = position - before; const sample = source[before] * (1 - fraction) + (source[Math.min(before + 1, source.length - 1)] ?? 0) * fraction; view.setInt16(44 + index * 2, Math.max(-1, Math.min(1, sample)) * 0x7fff, true); }
  return new Uint8Array(buffer);
}
