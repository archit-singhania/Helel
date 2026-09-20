"""Local command-line training entry point for Helel-22M and Helel-46M."""
from __future__ import annotations
import argparse, json, pathlib, shutil
from .model import HELEL_22M
from .product import HELEL_46M
from .tokenizer import ByteBPETokenizer
from .training import TrainingConfig, language_balanced_texts, packed_sequences, train

def main()->None:
    parser=argparse.ArgumentParser(description="Train a Helel model entirely on this machine")
    parser.add_argument("--dataset",type=pathlib.Path,required=True);parser.add_argument("--tokenizer",type=pathlib.Path,required=True);parser.add_argument("--output",type=pathlib.Path,required=True);parser.add_argument("--model",choices=("22m","46m"),default="22m");parser.add_argument("--steps",type=int,default=100);parser.add_argument("--batch-size",type=int,default=2);parser.add_argument("--sequence-length",type=int,default=512);parser.add_argument("--resume",type=pathlib.Path);parser.add_argument("--language-temperature",type=float,default=0.5);parser.add_argument("--fim-rate",type=float,default=0.5);parser.add_argument("--learning-rate",type=float,default=3e-4);parser.add_argument("--minimum-learning-rate",type=float,default=3e-5)
    args=parser.parse_args();model=HELEL_22M if args.model=="22m" else HELEL_46M;tokenizer=ByteBPETokenizer.load(args.tokenizer)
    if tokenizer.vocabulary_size!=model.vocabulary_size:raise ValueError(f"{args.model} requires vocabulary size {model.vocabulary_size}")
    records=[json.loads(line) for line in args.dataset.read_text(encoding="utf-8").splitlines() if line.strip()]
    texts,language_counts=language_balanced_texts(records,temperature=args.language_temperature)
    config=TrainingConfig(batch_size=args.batch_size,sequence_length=min(args.sequence_length,model.context_length-1),maximum_steps=args.steps,warmup_steps=min(100,max(1,args.steps//20)),fim_rate=args.fim_rate,learning_rate=args.learning_rate,minimum_learning_rate=args.minimum_learning_rate)
    sequences=packed_sequences(texts,tokenizer,config.sequence_length,config.seed,config.fim_rate)
    history=train(model,config,sequences,args.output,resume_from=args.resume);model.save(args.output/"config.json");shutil.copy2(args.tokenizer,args.output/"tokenizer.json");(args.output/"training-report.json").write_text(json.dumps({"model":args.model,"documents":len(texts),"sequences":len(sequences),"language_temperature":args.language_temperature,"sampled_languages":language_counts,"history":history},indent=2)+"\n",encoding="utf-8")
    print(f"trained Helel-{args.model.upper()} for {len(history)} steps at {args.output}")
if __name__=="__main__":main()
