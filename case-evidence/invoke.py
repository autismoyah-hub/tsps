"""Candidate author-owned authenticated invocation; no implicit selection or retry."""
import argparse, hashlib, importlib.util, json, os, subprocess, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent
TOOLS=ROOT.parents[1]/'control-cache'/'hf-author-control'/'004'/'files'
SDK=ROOT.parents[1]/'sdk'/'site-packages'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 if not __debug__: raise RuntimeError('Optimized interpreter forbidden')
 p=argparse.ArgumentParser()
 for n in ('plan','plan-sha256','prestart','prestart-sha256','selection','selection-sha256','guard-sha256'):p.add_argument('--'+n,required=True)
 a=p.parse_args()
 if os.environ.get('CCC_SESSION_ID')!='a57b09b1-fcf5-4aea-b25e-7f19910af934':raise RuntimeError('Wrong owner')
 pre=json.loads(Path(a.prestart).read_bytes()); assert sha(Path(a.prestart))==a.prestart_sha256 and sys.executable=='/usr/bin/python3' and sha(Path(sys.executable))==pre['pythonSHA256']
 adapter=TOOLS/'hf-project-access-v4.py'; helper=TOOLS/'authenticated-child-snapshot-command.py'; guard=ROOT/'provider_guard.py'
 assert sha(adapter)=='fab4bc92dc8601143ce61e910f8121774c22a0d19be6d2617bba9449f7131326'
 source=helper.read_bytes(); assert hashlib.sha256(source).hexdigest()=='00ff45d75be1eae3bc8f04995cf8b8bcbe2bc5adc28c1162b843cfe88c87024f9' and sha(guard)==a.guard_sha256
 module=importlib.util.module_from_spec(importlib.util.spec_from_loader('case_snapshot_helper',loader=None));exec(compile(source,str(helper),'exec'),module.__dict__)
 # Accepted helper binds adapter and entry snapshots. This guard is the entry.
 cmd=module.authenticated_pilot_command(sys.executable,adapter,sha(adapter),guard,a.guard_sha256,a.plan,sdk_target=SDK)
 # Replace pilot-style arguments with this exact guard's supported argument vector.
 cmd=cmd[:-2]+['--plan',a.plan,'--plan-sha256',a.plan_sha256,'--prestart',a.prestart,'--prestart-sha256',a.prestart_sha256,'--selection',a.selection,'--selection-sha256',a.selection_sha256]
 if any('\0' in v or len(v.encode())+1>=131072 for v in cmd):raise RuntimeError('Control argument bounds')
 subprocess.run(cmd,check=True)
if __name__=='__main__':main()
