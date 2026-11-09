"""Authenticate an author-owned status read, without submitting or collecting."""
import argparse,hashlib,importlib.util,json,os,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent; TOOLS=ROOT.parents[1]/'control-cache'/'hf-author-control'/'004'/'files'; SDK=ROOT.parents[1]/'sdk'/'site-packages'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 if not __debug__:raise RuntimeError('Optimized interpreter forbidden')
 p=argparse.ArgumentParser();p.add_argument('--plan',required=True);p.add_argument('--plan-sha256',required=True);p.add_argument('--entry-sha256',required=True);p.add_argument('--prestart',required=True);p.add_argument('--prestart-sha256',required=True);a=p.parse_args()
 assert os.environ.get('CCC_SESSION_ID')=='a57b09b1-fcf5-4aea-b25e-7f19910af934'
 prepath=Path(a.prestart).resolve();assert prepath.parent==ROOT and sha(prepath)==a.prestart_sha256
 pre=json.loads(prepath.read_bytes());intent=json.loads((ROOT/'exclusive-intent.json').read_bytes())
 assert pre['owner']==intent['owner']==os.environ['CCC_SESSION_ID'] and pre['planSHA256']==intent['planSHA256']==a.plan_sha256 and intent['prestartSHA256']==a.prestart_sha256
 assert a.entry_sha256==pre['files']['status.py'] and sha(ROOT/'status.py')==pre['files']['status.py']
 adapter=TOOLS/'hf-project-access-v4.py';helper=TOOLS/'authenticated-child-snapshot-command.py';entry=ROOT/'status.py'
 assert sha(adapter)=='fab4bc92dc8601143ce61e910f8121774c22a0d19be6d2617bba9449f7131326' and sha(entry)==a.entry_sha256
 source=helper.read_bytes();assert hashlib.sha256(source).hexdigest()=='00ff45d75be1eae3bc8f04995cfb8bcbe2bc5adc28c1162b843cfe88c87024f9'
 mod=importlib.util.module_from_spec(importlib.util.spec_from_loader('case_snapshot_helper',loader=None));exec(compile(source,str(helper),'exec'),mod.__dict__)
 cmd=mod.authenticated_pilot_command(sys.executable,adapter,sha(adapter),entry,a.entry_sha256,a.plan,sdk_target=SDK)+['--plan-sha256',a.plan_sha256]
 subprocess.run(cmd,check=True)
if __name__=='__main__':main()
