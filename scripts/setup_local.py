#!/usr/bin/env python3
"""Prepare Helel's project-local Python launcher without downloading packages."""
from __future__ import annotations
import argparse, pathlib, subprocess, sys
ROOT=pathlib.Path(__file__).resolve().parents[1]

def main()->None:
    parser=argparse.ArgumentParser();parser.add_argument('--train-smoke',action='store_true');args=parser.parse_args()
    environment=ROOT/'.venv'
    if not (environment/'bin/python').exists():subprocess.run([sys.executable,'-m','venv','--system-site-packages',str(environment)],check=True)
    launcher=environment/'bin/helel-inference'
    launcher.write_text(f'''#!{environment/'bin/python'}\nimport sys\nsys.path.insert(0,{str(ROOT/'ml')!r})\nfrom helel_ml.inference import main\nmain()\n''',encoding='utf-8');launcher.chmod(0o755)
    output=ROOT/'ml/checkpoints/smoke'
    if args.train_smoke:subprocess.run([str(environment/'bin/python'),'-m','helel_ml.smoke','--output',str(output),'--steps','2'],env={'PYTHONPATH':str(ROOT/'ml'),'PATH':str(environment/'bin')+':/usr/bin:/bin'},check=True)
    print(f'Local inference launcher: {launcher}')
    print(f'Smoke artifacts: {output}')
if __name__=='__main__':main()
