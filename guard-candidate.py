"""Held source candidate: one selected case, single submission, no replay."""
from pathlib import Path
import argparse, datetime, hashlib, json, os, re, subprocess, sys, urllib.request

ROOT=Path(__file__).resolve().parent
CASE=ROOT/'candidate'
OWNER='d427048d-86cc-4ad2-b3cc-bebade89e457'
REVIEWER='ad80bdbf-4243-4fee-9928-88fd6836505c'
SOURCE='47a1234bd1650b8f7b3697398993bce75c556fe7'
PLAN_SHA='28c17b69fceaddba1ce7b1617fd926193cc79142238a2f294ced56586d312f09'
PINS={'plan.json':PLAN_SHA,'payload.cjs':'0212ed48a9da3a1b78bd4a65c3e4b25399e77a4ba0cee089e33847f89a03c126','pilot.py':'2d7c5e44bb37f3043311962f8c05deee90e83ab6fa5ee5e3b499df238ed98200','package.json':'ac9cb75167ec66ac1c1f4345d65bda679a20ac1b9a85aba45dd1919dcd18ac78','frame_transport.py':'f53689d4f2389af93ad8f6a43fe0de94caf8dcaabde3eacbd13b1c957b129a76'}
sha=lambda raw:hashlib.sha256(raw).hexdigest()
def bound(path,digest):
 raw=Path(path).read_bytes();assert sha(raw)==digest,f'Identity mismatch: {path}';return raw

def git(*args):return subprocess.check_output(['git','-C',str(ROOT.parents[1]),*args])
def open_exclusive(path, data):
 with path.open('x',encoding='utf8') as f:
  json.dump(data,f,indent=2,default=str);f.write('\n');f.flush();os.fsync(f.fileno())

class NoRedirect(urllib.request.HTTPRedirectHandler):
 def redirect_request(self,*args,**kwargs):raise ValueError('redirect refused')

def validate_source(plan):
 ref=plan['expectedInputsRef'];assert ref['source']==SOURCE and ref['repository']==plan['repository']
 url=f"https://raw.githubusercontent.com/{ref['repository']}/{ref['artifactCommit']}/{ref['path']}"
 with urllib.request.build_opener(NoRedirect).open(url,timeout=60) as resp:
  assert resp.status==200 and resp.geturl()==url
  raw=resp.read(ref['bytes']+1)
 assert len(raw)==ref['bytes'] and sha(raw)==ref['sha256']
 doc=json.loads(raw);assert set(doc)=={'schema','source','tests','expectedTestCounts','expectedInputs','sourceBlobs','sourceModes'}
 assert doc['schema']==1 and doc['source']==SOURCE and doc['tests']==plan['tests'] and doc['expectedTestCounts']==plan['expectedTestCounts']
 tree={}
 for row in git('ls-tree','-r','-z',SOURCE).split(b'\0'):
  if not row:continue
  meta,name=row.split(b'\t',1);mode,kind,oid=meta.decode().split();name=name.decode()
  if name.startswith('server/') or name in ('.yarnrc.yml','package.json','yarn.lock','scripts/run-with-env.mjs'):
   assert kind=='blob' and mode in ('100644','100755');tree[name]=(mode,oid)
 assert len(tree)==ref['count']==5031
 assert set(tree)==set(doc['expectedInputs'])==set(doc['sourceBlobs'])==set(doc['sourceModes'])
 for file,(mode,oid) in tree.items():
  assert doc['sourceBlobs'][file]==oid and doc['sourceModes'][file]==mode
  data=git('cat-file','blob',oid);assert sha(data)==doc['expectedInputs'][file]
 return {'url':url,'sha256':sha(raw),'count':len(tree)}

def current_review(plan):
 # This is a readback captured directly from this authenticated session, not a self-authored verdict.
 capture=ROOT/'exact-execution-review.json';data=json.loads(capture.read_bytes())
 assert data['owner']==OWNER and data['reviewer']==REVIEWER and data['verdict']=='EXACT_CASE_GO'
 subject={'planSHA256':PLAN_SHA,'source':SOURCE,'payloadSHA256':PINS['payload.cjs'],'guardSHA256':sha(Path(__file__).read_bytes())}
 assert data['subject']==subject
 receipt=Path(data['directCapture']);raw=bound(receipt,data['directCaptureSHA256']); transcript=json.loads(raw)
 texts=[r.get('text','') for r in transcript['turns'] if r.get('role')=='user']
 assert any(REVIEWER in text and 'EXACT_CASE_GO' in text and all(v in text for v in subject.values()) for text in texts)
 return {'subject':subject,'captureSHA256':sha(raw)}

def check():
 assert os.environ.get('CCC_SESSION_ID')==OWNER and re.fullmatch('[a-f0-9-]{36}',REVIEWER) and REVIEWER!=OWNER
 for name,digest in PINS.items():bound(CASE/name,digest)
 plan=json.loads((CASE/'plan.json').read_bytes());assert plan['source']==SOURCE and plan['timeout']==1800 and plan['attempts']==1 and not plan.get('executionHold')
 assert plan['image']=='node@sha256:b04ce4ae4e95b522112c2e5c52f781471a5cbc3b594527bcddedee9bc48c03a0' and plan['flavor']=='cpu-upgrade'
 assert git('rev-parse','HEAD').decode().strip()==SOURCE and not git('status','--porcelain')
 ref=subprocess.check_output(['git','ls-remote','https://github.com/autismoyah-hub/tsps.git','refs/heads/enakhra-camel-mask-d427'],text=True).split()
 assert len(ref)==2 and ref[0]==SOURCE
 source=validate_source(plan);review=current_review(plan)
 assert not (CASE/'submission.json').exists() and not (ROOT/'submission-intent.json').exists()
 # Scan all case-owned retained intent/receipt files; malformed files fail closed.
 for file in ROOT.rglob('submission.json'):
  old=json.loads(file.read_bytes());oldplan=old.get('plan',{})
  assert not(oldplan.get('source')==SOURCE and set(oldplan.get('tests',[])) & set(plan['tests'])),str(file)
 return plan,source,review

def main():
 parser=argparse.ArgumentParser();parser.add_argument('--submit-once',action='store_true',required=True);parser.parse_args()
 plan,source,review=check()
 from huggingface_hub import HfApi
 api=HfApi();identity=api.whoami()['name'];assert identity=='Autismo-Yah'
 matches=[]
 for job in api.list_jobs(namespace=identity,labels={'project':'tsps'}):
  labels=getattr(job,'labels',None);job_id=getattr(job,'id',None)
  assert isinstance(job_id,str) and job_id and isinstance(labels,dict),'Unclassifiable provider listing'
  if labels.get('source')==SOURCE:matches.append(job_id)
 assert not matches,'Matching prior provider job: reconcile without submission'
 # Recheck all local and remote state immediately after provider reconciliation.
 again,source2,review2=check();assert again==plan and source2==source and review2==review
 preview=json.loads(subprocess.check_output([sys.executable,str(CASE/'pilot.py'),'--plan',str(CASE/'plan.json')]))
 request=preview['request'];assert preview['plan']==plan and request['attempts']==1 and request['timeout']==1800
 assert all(len(arg.encode())+1<131072 and '\0' not in arg for arg in request['command'])
 intent={'owner':OWNER,'collector':OWNER,'source':SOURCE,'planSHA256':PLAN_SHA,'payloadSHA256':PINS['payload.cjs'],'requestSHA256':sha(json.dumps(request,sort_keys=True,separators=(',',':')).encode()),'review':review,'sourceReadback':source,'createdUTC':datetime.datetime.now(datetime.timezone.utc).isoformat(),'state':'SUBMISSION_STARTING_RECONCILIATION_REQUIRED','noReplay':True}
 open_exclusive(ROOT/'submission-intent.json',intent)
 # Direct call once: intent is durable before the network mutation. Any error is ambiguous.
 job=api.run_job(namespace=identity,**request)
 job_id=getattr(job,'id',None);assert isinstance(job_id,str) and job_id
 open_exclusive(ROOT/'submitted-job-id.json',{'jobId':job_id,'namespace':identity,'owner':OWNER,'planSHA256':PLAN_SHA,'noReplay':True})
 returned_command=getattr(job,'command',None)
 result={'id':job_id,'command':returned_command}
 open_exclusive(CASE/'submission.json',{'stage':'SUBMITTED','namespace':identity,'request':request,'plan':plan,'job':result,'payloadSHA256':sha(request['command'][-1].encode())})
 assert returned_command==request['command'],'Returned command mismatch; retain same-job custody'
 print(json.dumps({'stage':'SUBMITTED','jobId':job_id,'source':SOURCE,'collector':OWNER,'noReplay':True}))
if __name__=='__main__':main()
