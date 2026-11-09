"""Author's read-only status for the already-submitted exact job."""
import argparse, dataclasses, datetime, hashlib, json, os, types
from pathlib import Path
from huggingface_hub import HfApi
ROOT=Path(__file__).resolve().parent
def main():
 if not __debug__:raise RuntimeError('Optimized interpreter forbidden')
 p=argparse.ArgumentParser();p.add_argument('--plan',required=True);p.add_argument('--plan-sha256',required=True);a=p.parse_args()
 plan=Path(a.plan).resolve(); raw=plan.read_bytes(); assert plan.parent==ROOT and hashlib.sha256(raw).hexdigest()==a.plan_sha256
 submission=json.loads((ROOT/'submission.json').read_bytes()); intent=json.loads((ROOT/'exclusive-intent.json').read_bytes())
 assert intent['owner']==os.environ.get('CCC_SESSION_ID')=='a57b09b1-fcf5-4aea-b25e-7f19910af934' and intent['planSHA256']==a.plan_sha256
 assert submission['plan']==json.loads(raw) and submission['namespace']=='Autismo-Yah' and submission['job']['id']
 api=HfApi();assert api.whoami()['name']=='Autismo-Yah'; job=api.inspect_job(job_id=submission['job']['id'],namespace='Autismo-Yah')
 identityraw=(ROOT/'job_identity.py').read_bytes(); assert hashlib.sha256(identityraw).hexdigest()=='a782983f86a573f01b2e1a019c2b5ab29b1ad82fa1ff369f342f2571c224482f'
 identity=types.ModuleType('job_identity');exec(compile(identityraw,str(ROOT/'job_identity.py'),'exec'),identity.__dict__)
 actual=dataclasses.asdict(job)
 stamp=datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S%fZ'); target=ROOT/f'precollect-status-{stamp}.json'
 with target.open('x') as f:json.dump(actual,f,default=str,indent=2);f.flush();os.fsync(f.fileno())
 identity.verify_job(actual,submission)
 print(json.dumps({'path':str(target),'stage':str(getattr(job.status.stage,'value',job.status.stage)),'jobId':job.id}))
if __name__=='__main__':main()
