"""Held source candidate: one selected case, single submission, no replay."""
from pathlib import Path
import argparse, datetime, hashlib, json, os, re, subprocess, sys, urllib.request

ROOT=Path(__file__).resolve().parent
CASE=ROOT/'candidate'
OWNER='d427048d-86cc-4ad2-b3cc-bebade89e457'
REVIEWER='ad80bdbf-4243-4fee-9928-88fd6836505c'
SOURCE='47f4dc6875e51099f91a18151835f482b34401ee'
PLAN_SHA='3355f4d839fcfff9d8de9020901bfca45a69a5c745eb3ee59d943cb308e79e44'
PINS={'plan.json':PLAN_SHA,'payload.cjs':'0212ed48a9da3a1b78bd4a65c3e4b25399e77a4ba0cee089e33847f89a03c126','pilot.py':'2d7c5e44bb37f3043311962f8c05deee90e83ab6fa5ee5e3b499df238ed98200','package.json':'a3478faa655b2305474e46bb6ba52d2faddb67cce70f98e67592f0346dc88b9e','collector.py':'f492a863d3edc8f3f09e386efeeabaaa9abacfe92b6b357512c09d4c46f5d1e8','frame_transport.py':'f53689d4f2389af93ad8f6a43fe0de94caf8dcaabde3eacbd13b1c957b129a76'}
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
 assert len(tree)==ref['count']==5046
 assert set(tree)==set(doc['expectedInputs'])==set(doc['sourceBlobs'])==set(doc['sourceModes'])
 proc=subprocess.Popen(['git','-C',str(ROOT.parents[1]),'cat-file','--batch'],stdin=subprocess.PIPE,stdout=subprocess.PIPE)
 try:
  for file,(mode,oid) in tree.items():
   assert doc['sourceBlobs'][file]==oid and doc['sourceModes'][file]==mode
   proc.stdin.write((oid+'\n').encode());proc.stdin.flush()
   header=proc.stdout.readline().decode().split();assert len(header)==3 and header[0]==oid and header[1]=='blob'
   n=int(header[2]);assert 0<=n<=128*1024*1024
   data=proc.stdout.read(n);assert len(data)==n and proc.stdout.read(1)==b'\n' and sha(data)==doc['expectedInputs'][file]
 finally:
  proc.stdin.close()
  if proc.wait()!=0:raise ValueError('git batch failed')
 return {'url':url,'sha256':sha(raw),'count':len(tree)}

def current_review(plan):
 # Fetch both authenticated Cuttle transcripts fresh; correlate sender call and recipient message.
 def read_session(sid):
  raw=subprocess.check_output(['/workspace/runtime/bin/cuttle','sessions','read','--turns','300',sid],timeout=30)
  data=json.loads(raw);assert isinstance(data.get('turns'),list);return data['turns']
 subject={'planSHA256':PLAN_SHA,'source':SOURCE,'payloadSHA256':PINS['payload.cjs'],'guardSHA256':sha(Path(__file__).read_bytes()),'collectorSHA256':PINS['collector.py']}
 owner_turns=read_session(OWNER);peer_turns=read_session(REVIEWER)
 selected=[]
 for row in owner_turns:
  text=row.get('text','')
  if row.get('role')!='user' or not text.startswith('[From Cuttle session ') or REVIEWER not in text.split('\n',1)[0]:continue
  msg=text.split('\n',1)[1] if '\n' in text else ''
  if 'EXACT_CASE_GO' in msg and all(value in msg for value in subject.values()):selected.append((row,msg))
 assert len(selected)==1,'Missing or ambiguous peer verdict'
 owner_row,msg=selected[0]
 sent=[]
 for index,row in enumerate(peer_turns):
  tool=row.get('tool',{});command=tool.get('name','')
  if row.get('role')!='assistant' or row.get('kind')!='tool' or msg not in command:continue
  if f'cuttle sessions send {OWNER}' not in command:continue
  result=next((x for x in peer_turns[index+1:] if x.get('kind')=='tool_result' and x.get('tool',{}).get('id')==tool.get('id')),None)
  if result and result.get('tool',{}).get('ok') is True:sent.append({'call':row,'result':result})
 assert len(sent)==1,'Missing or ambiguous reviewer-side send'
 proof={'subject':subject,'reviewer':REVIEWER,'recipient':OWNER,'recipientTurn':owner_row,'sender':sent[0]}
 return {'subject':subject,'correlatedCaptureSHA256':sha(json.dumps(proof,sort_keys=True,separators=(',',':')).encode()),'proof':proof}

def current_fit(plan):
 repo=ROOT.parents[1]
 remote=subprocess.check_output(['git','-C',str(repo),'ls-remote','https://github.com/RSPSApp/tsps.git','refs/heads/main'],text=True,timeout=40).split()
 assert len(remote)==2 and re.fullmatch('[0-9a-f]{40}',remote[0]) and remote[1]=='refs/heads/main'
 main=remote[0]
 subprocess.run(['git','-C',str(repo),'-c','maintenance.auto=false','fetch','--no-tags','--no-write-fetch-head','https://github.com/RSPSApp/tsps.git',main],capture_output=True,check=True,timeout=90)
 connected=['server/plugins/quests/quests/EnakhrasLament.Quest.js','server/tests/qh728-q032.test.cjs','server/plugins/quests/QuestRuntime.js','server/plugins/quests/Quests.plugin.js','server/src/main/typescript/elvarg/plugins/PluginManager.ts','server/src/main/typescript/elvarg/plugins/PluginTypes.ts','server/src/main/typescript/elvarg/net/packet/impl/UseItemPacketListener.ts','server/src/main/typescript/elvarg/game/model/container/impl/Inventory.ts']
 diff=git('diff','--name-only','c77054e654410ddf1453a87cb99369579bc3bc85',main,'--',*connected).decode().splitlines()
 assert not diff, f'Connected main paths changed: {diff}'
 prs=json.loads(subprocess.check_output(['gh','pr','list','--repo','RSPSApp/tsps','--state','open','--limit','200','--json','number,files'],timeout=60))
 assert len(prs)<200,'Open PR listing incomplete'
 touched=set(connected[:2]);overlap=[]
 for pr in prs:
  assert isinstance(pr.get('files'),list) and len(pr['files'])<100,'PR file listing may be truncated'
  if any(x.get('path') in touched for x in pr['files']):overlap.append(pr['number'])
 assert not overlap,f'Open PR overlaps: {overlap}'
 return {'main':main,'connectedPaths':connected,'openPRCount':len(prs),'exactOverlap':overlap}

def selected(review):
 raw=(ROOT/'selection.json').read_bytes();record=json.loads(raw)
 assert set(record)=={'action','owner','source','planSHA256','reviewCaptureSHA256','createdUTC'}
 assert record['action']=='SELECT_ONCE' and record['owner']==OWNER and record['source']==SOURCE and record['planSHA256']==PLAN_SHA and record['reviewCaptureSHA256']==review['correlatedCaptureSHA256']
 age=(datetime.datetime.now(datetime.timezone.utc)-datetime.datetime.fromisoformat(record['createdUTC'])).total_seconds()
 assert 0<=age<=300,'Selection stale or future'
 return {'sha256':sha(raw),'createdUTC':record['createdUTC']}

def check():
 assert os.environ.get('CCC_SESSION_ID')==OWNER and re.fullmatch('[a-f0-9-]{36}',REVIEWER) and REVIEWER!=OWNER
 for name,digest in PINS.items():bound(CASE/name,digest)
 plan=json.loads((CASE/'plan.json').read_bytes());assert plan['source']==SOURCE and plan['timeout']==1800 and plan['attempts']==1 and not plan.get('executionHold')
 assert plan['image']=='node@sha256:b04ce4ae4e95b522112c2e5c52f781471a5cbc3b594527bcddedee9bc48c03a0' and plan['flavor']=='cpu-upgrade'
 assert git('rev-parse','HEAD').decode().strip()==SOURCE and not git('status','--porcelain')
 ref=subprocess.check_output(['git','ls-remote','https://github.com/autismoyah-hub/tsps.git','refs/heads/enakhra-camel-mask-d427'],text=True).split()
 assert len(ref)==2 and ref[0]==SOURCE
 source=validate_source(plan);review=current_review(plan);selection=selected(review);fit=current_fit(plan)
 assert not (CASE/'submission.json').exists() and not (ROOT/'submission-intent.json').exists()
 prior=ROOT.parent/'enakhra-mask-control'
 bound(prior/'selection.json','7173a500fb478abdaf1a60a9a5fabc06fe4e294d70438b20556cf58a60b0464c')
 assert not (prior/'submission-intent.json').exists() and not (prior/'submitted-job-id.json').exists() and not (prior/'candidate/submission.json').exists(),'Prior selected case needs reconciliation'
 # Scan all case-owned retained intent/receipt files; malformed files fail closed.
 for file in ROOT.rglob('submission.json'):
  old=json.loads(file.read_bytes());oldplan=old.get('plan',{})
  assert not(oldplan.get('source')==SOURCE and set(oldplan.get('tests',[])) & set(plan['tests'])),str(file)
 return plan,source,review,selection,fit

def main():
 parser=argparse.ArgumentParser();parser.add_argument('--submit-once',action='store_true',required=True);parser.parse_args()
 plan,source,review,selection,fit=check()
 from huggingface_hub import HfApi
 api=HfApi();identity=api.whoami()['name'];assert identity=='Autismo-Yah'
 matches=[]
 for job in api.list_jobs(namespace=identity,labels={'project':'tsps'}):
  labels=getattr(job,'labels',None);job_id=getattr(job,'id',None)
  assert isinstance(job_id,str) and job_id and isinstance(labels,dict),'Unclassifiable provider listing'
  if labels.get('source') in (SOURCE,'47a1234bd1650b8f7b3697398993bce75c556fe7'):matches.append(job_id)
 assert not matches,'Matching prior provider job: reconcile without submission'
 # Recheck all local and remote state immediately after provider reconciliation.
 again,source2,review2,selection2,fit2=check();assert again==plan and source2==source and review2==review and selection2==selection and fit2==fit
 preview=json.loads(subprocess.check_output([sys.executable,'-E','-S',str(CASE/'pilot.py'),'--plan',str(CASE/'plan.json')],env={'PATH':'/usr/bin:/bin','LANG':'C.UTF-8','TZ':'UTC'}))
 request=preview['request'];assert preview['plan']==plan and request['attempts']==1 and request['timeout']==1800
 assert all(len(arg.encode())+1<131072 and '\0' not in arg for arg in request['command'])
 intent={'owner':OWNER,'collector':OWNER,'source':SOURCE,'planSHA256':PLAN_SHA,'payloadSHA256':PINS['payload.cjs'],'requestSHA256':sha(json.dumps(request,sort_keys=True,separators=(',',':')).encode()),'review':review,'selection':selection,'currentFit':fit,'sourceReadback':source,'createdUTC':datetime.datetime.now(datetime.timezone.utc).isoformat(),'state':'SUBMISSION_STARTING_RECONCILIATION_REQUIRED','noReplay':True}
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
