#!/usr/bin/env python3
"""Deterministic, offline HelelBench disposable-repository runner."""
from __future__ import annotations
import argparse, json, pathlib, tempfile, time
ROOT=pathlib.Path(__file__).resolve().parents[1]

def safe(root:pathlib.Path,relative:str)->pathlib.Path:
    path=(root/relative).resolve()
    if root.resolve() not in path.parents: raise ValueError('benchmark path escapes fixture')
    return path

def execute(task:dict,workspace:pathlib.Path)->dict:
    started=time.monotonic();actions=0;before={}
    try:
        for relative,content in task['files'].items():
            path=safe(workspace,relative);path.parent.mkdir(parents=True,exist_ok=True);path.write_text(content,encoding='utf-8')
        for relative,content in task['writes'].items():
            path=safe(workspace,relative);before[relative]=path.read_bytes() if path.exists() else None;path.parent.mkdir(parents=True,exist_ok=True);path.write_text(content,encoding='utf-8');actions+=1
        success=all(text in safe(workspace,path).read_text(encoding='utf-8') for path,text in task['expect'].items()) and all(text not in safe(workspace,path).read_text(encoding='utf-8') for path,text in task.get('reject',{}).items())
        for relative,content in before.items():
            path=safe(workspace,relative)
            if content is None:path.unlink(missing_ok=True)
            else:path.write_bytes(content)
        rollback=all((safe(workspace,path).read_bytes() if safe(workspace,path).exists() else None)==content for path,content in before.items())
        return {'id':task['id'],'category':task['category'],'stack':task['stack'],'success':success,'validationPassed':success,'validToolCallRate':1.0,'rollbackPassed':rollback,'actions':actions,'runtimeMs':round((time.monotonic()-started)*1000,3),'crashed':False}
    except Exception as error:
        return {'id':task.get('id','unknown'),'category':task.get('category','unknown'),'stack':task.get('stack','unknown'),'success':False,'validationPassed':False,'validToolCallRate':0.0,'rollbackPassed':False,'actions':actions,'runtimeMs':round((time.monotonic()-started)*1000,3),'crashed':True,'error':str(error)}

def run(catalog:pathlib.Path)->dict:
    data=json.loads(catalog.read_text(encoding='utf-8'));tasks=data['tasks']
    if len(tasks)<30 or len({t['id'] for t in tasks})!=len(tasks):raise ValueError('HelelBench requires at least 30 uniquely identified tasks')
    results=[]
    with tempfile.TemporaryDirectory(prefix='helel-bench-') as folder:
        root=pathlib.Path(folder)
        for task in tasks:
            workspace=root/task['id'];workspace.mkdir();results.append(execute(task,workspace))
    passed=sum(r['success'] and r['validationPassed'] and r['rollbackPassed'] for r in results)
    return {'schemaVersion':2,'planner':'scripted-fixture','learnedModel':None,'summary':{'total':len(results),'passed':passed,'successRate':passed/len(results),'crashes':sum(r['crashed'] for r in results)},'results':results}

def main()->None:
    parser=argparse.ArgumentParser();parser.add_argument('--catalog',type=pathlib.Path,default=ROOT/'benchmarks/tasks.json');parser.add_argument('--output',type=pathlib.Path);args=parser.parse_args();report=run(args.catalog);encoded=json.dumps(report,indent=2,sort_keys=True)
    if args.output:args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(encoded+'\n',encoding='utf-8')
    print(encoded)
if __name__=='__main__':main()
