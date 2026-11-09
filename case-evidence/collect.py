"""One author collector for the existing exact submitted plan; never submits."""
import argparse, hashlib, importlib.util, json, os, subprocess, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent
TOOLS=ROOT.parents[1]/'control-cache'/'hf-author-control'/'004'/'files'
SDK=ROOT.parents[1]/'sdk'/'site-packages'
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 if not __debug__:raise RuntimeError('Optimized interpreter forbidden')
 p=argparse.ArgumentParser();p.add_argument('--plan',required=True);p.add_argument('--plan-sha256',required=True);p.add_argument('--status',required=True);p.add_argument('--status-sha256',required=True);p.add_argument('--prestart',required=True);p.add_argument('--prestart-sha256',required=True);a=p.parse_args()
 if os.environ.get('CCC_SESSION_ID')!='a57b09b1-fcf5-4aea-b25e-7f19910af934':raise RuntimeError('Wrong owner')
 prepath=Path(a.prestart).resolve();assert prepath.parent==ROOT and sha(prepath)==a.prestart_sha256;pre=json.loads(prepath.read_bytes())
 for n in ('collect.py','collector_entry.py','job_identity.py','pilot.py','status.py','status_invoke.py','payload.cjs'):assert sha(ROOT/n)==pre['files'][n]
 plan=Path(a.plan).resolve(); assert plan.parent==ROOT and sha(plan)==a.plan_sha256
 submission=json.loads((ROOT/'submission.json').read_bytes()); assert submission['plan']==json.loads(plan.read_bytes()) and submission['job']['id'] and submission['namespace']=='Autismo-Yah'
 statuspath=Path(a.status).resolve(); assert statuspath.parent==ROOT and sha(statuspath)==a.status_sha256
 status=json.loads(statuspath.read_bytes()); assert status['id']==submission['job']['id'] and status['status']['stage'] in ('COMPLETED','ERROR','CANCELED','CANCELLED')
 intent=json.loads((ROOT/'exclusive-intent.json').read_bytes()); assert intent['owner']==os.environ['CCC_SESSION_ID'] and intent['planSHA256']==a.plan_sha256 and intent['prestartSHA256']==a.prestart_sha256
 marker=ROOT/'collector-intent.json'; data={'owner':intent['owner'],'jobId':submission['job']['id'],'planSHA256':a.plan_sha256,'collection':'one designated owner; no submission'}
 assert not marker.exists(), 'Existing collector intent: retain and reconcile without replay'
 with marker.open('x') as f: json.dump(data,f,indent=2);f.flush();os.fsync(f.fileno())
 fd=os.open(ROOT,os.O_RDONLY|os.O_DIRECTORY)
 try:os.fsync(fd)
 finally:os.close(fd)
 adapter=TOOLS/'hf-project-access-v4.py';helper=TOOLS/'authenticated-child-snapshot-command.py';entry=ROOT/'collector_entry.py'
 assert sha(adapter)=='fab4bc92dc8601143ce61e910f8121774c22a0d19be6d2617bba9449f7131326'
 source=helper.read_bytes();assert hashlib.sha256(source).hexdigest()=='00ff45d75be1eae3bc8f04995cfb8bcbe2bc5adc28c1162b843cfe88c87024f9'
 mod=importlib.util.module_from_spec(importlib.util.spec_from_loader('case_snapshot_helper',loader=None));exec(compile(source,str(helper),'exec'),mod.__dict__)
 cmd=mod.authenticated_pilot_command(sys.executable,adapter,sha(adapter),entry,sha(entry),plan,sdk_target=SDK)+['--plan-sha256',a.plan_sha256,'--status',str(statuspath),'--status-sha256',a.status_sha256]
 subprocess.run(cmd,check=True)
if __name__=='__main__':main()
