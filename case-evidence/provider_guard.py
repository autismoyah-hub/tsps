"""Case candidate: authenticated duplicate check then unchanged pilot submit. Not selected."""
import argparse, datetime, hashlib, json, os, runpy, sys
from pathlib import Path
from huggingface_hub import HfApi

ROOT = Path(__file__).resolve().parent
SOURCE = 'daff5ac899dc1bae79ab6fea6d66df15f0ef855f'
NAME = 'tsps-woodcutting-eat-before-a57b-001'
OWNER = 'a57b09b1-fcf5-4aea-b25e-7f19910af934'

def digest(path): return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def main():
    p=argparse.ArgumentParser()
    p.add_argument('--plan', required=True); p.add_argument('--plan-sha256', required=True)
    p.add_argument('--prestart', required=True); p.add_argument('--prestart-sha256', required=True)
    p.add_argument('--selection', required=True); p.add_argument('--selection-sha256', required=True)
    a=p.parse_args()
    planpath=Path(a.plan).resolve(); prepath=Path(a.prestart).resolve(); selpath=Path(a.selection).resolve()
    assert all(x.parent==ROOT for x in (planpath,prepath,selpath))
    assert digest(planpath)==a.plan_sha256 and digest(prepath)==a.prestart_sha256 and digest(selpath)==a.selection_sha256
    plan=json.loads(planpath.read_bytes()); pre=json.loads(prepath.read_bytes()); sel=json.loads(selpath.read_bytes())
    assert plan['source']==SOURCE and plan['name']==NAME and plan['attempts']==1 and plan['timeout']==1800 and not plan.get('executionHold')
    assert pre['source']==SOURCE and pre['planSHA256']==a.plan_sha256 and pre['selectionSHA256']==a.selection_sha256 and pre['owner']==OWNER
    age=(datetime.datetime.now(datetime.timezone.utc)-datetime.datetime.fromisoformat(pre['createdUTC'])).total_seconds(); assert 0<=age<=300
    assert sel['source']==SOURCE and sel['planSHA256']==a.plan_sha256 and sel['owner']==OWNER and sel['action']=='SELECT_ONCE' and sel['attempts']==1
    for path,sha in pre['files'].items(): assert digest(ROOT/path)==sha
    assert os.environ.get('CCC_SESSION_ID')==OWNER
    assert not (ROOT/'submission.json').exists() and not (ROOT/'exclusive-intent.json').exists()
    api=HfApi(); assert api.whoami()['name']=='Autismo-Yah'
    count=0
    for job in api.list_jobs(namespace='Autismo-Yah'):
        count+=1
        labels=job.labels or {}; command=job.command or []
        # Name may be encoded in the submitted program; no log/secret is printed.
        if labels.get('source')==SOURCE or any(NAME in str(part) for part in command):
            raise RuntimeError('Matching provider source or name: stop and reconcile')
    record={'owner':OWNER,'source':SOURCE,'name':NAME,'checkedUTC':datetime.datetime.now(datetime.timezone.utc).isoformat(),'jobsChecked':count,'planSHA256':a.plan_sha256,'prestartSHA256':a.prestart_sha256,'selectionSHA256':a.selection_sha256,'action':'SUBMISSION_STARTING_NO_RETRY','collector':OWNER}
    with (ROOT/'exclusive-intent.json').open('x') as out: json.dump(record,out,indent=2)
    # pilot owns its own exclusive submission.json before run_job. Any ambiguity is retained.
    sys.argv=[str(ROOT/'pilot.py'),'--plan',str(planpath),'--submit']
    runpy.run_path(str(ROOT/'pilot.py'),run_name='__main__')
if __name__=='__main__': main()
