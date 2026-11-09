"""Authenticated pilot adapter: inspect and match exact job before fetching any logs."""
import argparse, dataclasses, datetime, hashlib, json, os, sys, types
from pathlib import Path
import huggingface_hub
ROOT=Path(__file__).resolve().parent
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 if not __debug__:raise RuntimeError('Optimized interpreter forbidden')
 p=argparse.ArgumentParser();p.add_argument('--plan',required=True);p.add_argument('--plan-sha256',required=True);p.add_argument('--status',required=True);p.add_argument('--status-sha256',required=True);a=p.parse_args()
 plan=Path(a.plan).resolve();statuspath=Path(a.status).resolve();assert plan.parent==ROOT and statuspath.parent==ROOT and sha(plan)==a.plan_sha256 and sha(statuspath)==a.status_sha256
 submission=json.loads((ROOT/'submission.json').read_bytes());status=json.loads(statuspath.read_bytes());assert submission['plan']==json.loads(plan.read_bytes())
 raw=(ROOT/'job_identity.py').read_bytes();assert hashlib.sha256(raw).hexdigest()=='aa6b12b2ac3d8b9a1774223278eb5d8c02edf2ecceb2b8118c120a3037020d13';identity=types.ModuleType('job_identity');exec(compile(raw,str(ROOT/'job_identity.py'),'exec'),identity.__dict__)
 payload=(ROOT/'payload.cjs').read_bytes();assert hashlib.sha256(payload).hexdigest()=='220ef3554ec1bf8951c22dc8e50b849feeeaaa5b0b0b06f664ea7b393cb1dde4'
 planvalue=json.loads(plan.read_bytes());jobid=identity.verify_job(status,submission,planvalue,payload);assert status['status']['stage'] in ('COMPLETED','ERROR','CANCELED','CANCELLED')
 original=huggingface_hub.HfApi
 class VerifiedApi:
  def __init__(self):self.api=original();self.verified=False
  def whoami(self):return self.api.whoami()
  def inspect_job(self,job_id,namespace):
   assert job_id==jobid and namespace=='Autismo-Yah';info=self.api.inspect_job(job_id=job_id,namespace=namespace);observed=dataclasses.asdict(info)
   stamp=datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S%fZ');path=ROOT/('collector-inspect-'+stamp+'.json')
   with path.open('x') as f:json.dump(observed,f,default=str,indent=2);f.flush();os.fsync(f.fileno())
   identity.verify_job(observed,submission,planvalue,payload);assert observed['status']['stage'] in ('COMPLETED','ERROR','CANCELED','CANCELLED');self.verified=True;return info
  def fetch_job_logs(self,job_id,namespace,follow=False):
   assert self.verified and job_id==jobid and namespace=='Autismo-Yah' and follow is False
   stream=self.api.fetch_job_logs(job_id=job_id,namespace=namespace,follow=follow)
   stamp=datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S%fZ');path=ROOT/('collector-raw-stream-'+stamp+'.txt')
   def retain():
    with path.open('x',encoding='utf-8') as out:
     for line in stream:
      if not isinstance(line,str):raise TypeError('Unexpected log line type')
      out.write(line+'\n');out.flush();os.fsync(out.fileno());yield line
   return retain()
 pilot=ROOT/'pilot.py';pilotraw=pilot.read_bytes();assert hashlib.sha256(pilotraw).hexdigest()=='2d7c5e44bb37f3043311962f8c05deee90e83ab6fa5ee5e3b499df238ed98200'
 huggingface_hub.HfApi=lambda *args,**kwargs:VerifiedApi()
 sys.argv=[str(pilot),'--plan',str(plan),'--collect'];m=types.ModuleType('__main__');m.__dict__.update(__file__=str(pilot),__name__='__main__',__package__='',__spec__=None,__cached__=None);sys.modules['__main__']=m
 exec(compile(pilotraw,str(pilot),'exec'),m.__dict__)
if __name__=='__main__':main()
