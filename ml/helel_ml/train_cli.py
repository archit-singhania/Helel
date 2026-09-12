"""Local command-line training entry point for Helel-22M and Helel-46M."""
from __future__ import annotations
import argparse, json, pathlib, shutil
from .model import HELEL_22M
from .product import HELEL_46M
from .tokenizer import ByteBPETokenizer
from .training import TrainingConfig, packed_sequences, train

def main()->None:
    parser=argparse.ArgumentParser(description="Train a Helel model entirely on this machine")
    parser.add_argument("--dataset",type=pathlib.Path,required=True);parser.add_argument("--tokenizer",type=pathlib.Path,required=True);parser.add_argument("--output",type=pathlib.Path,required=True);parser.add_argument("--model",choices=("22m","46m"),default="22m");parser.add_argument("--steps",type=int,default=100);parser.add_argument("--batch-size",type=int,default=2);parser.add_argument("--sequence-length",type=int,default=512);parser.add_argument("--resume",type=pathlib.Path)
    args=parser.parse_args();model=HELEL_22M if args.model=="22m" else HELEL_46M;tokenizer=ByteBPETokenizer.load(args.tokenizer)
    if tokenizer.vocabulary_size!=model.vocabulary_size:raise ValueError(f"{args.model} requires vocabulary size {model.vocabulary_size}")
    texts=[json.loads(line)["text"] for line in args.dataset.read_text(encoding="utf-8").splitlines() if line.strip()]
    config=TrainingConfig(batch_size=args.batch_size,sequence_length=min(args.sequence_length,model.context_length-1),maximum_steps=args.steps,warmup_steps=min(100,max(1,args.steps//20)))
    sequences=packed_sequences(texts,tokenizer,config.sequence_length,config.seed,config.fim_rate)
    history=train(model,config,sequences,args.output,resume_from=args.resume);model.save(args.output/"config.json");shutil.copy2(args.tokenizer,args.output/"tokenizer.json");(args.output/"training-report.json").write_text(json.dumps({"model":args.model,"documents":len(texts),"sequences":len(sequences),"history":history},indent=2)+"\n",encoding="utf-8")
    print(f"trained Helel-{args.model.upper()} for {len(history)} steps at {args.output}")
if __name__=="__main__":main()
