"""Held source candidate: one selected case, single submission, no replay."""
from pathlib import Path
import argparse, datetime, hashlib, json, os, re, resource, subprocess, sys, tempfile, urllib.request

if sys.flags.optimize or not sys.flags.dont_write_bytecode: raise RuntimeError('required interpreter flags')
ROOT=Path(__file__).resolve().parent
CASE=ROOT/'candidate'
OWNER='3f20dcbd-b1d9-4e4a-bc47-a3b02d6ee6bc'
REVIEWER='10e752da-405a-47bd-8d0e-21f4054a25f9'
SOURCE='57e96673bf57e6dc65de5508d60c383b90064ed2'
PLAN_SHA='a77dc6fa3f6e3c23b9e17fc6b7f0f2e5ff3726f6caec3205983071f6805c669c'
PINS={'plan.json':PLAN_SHA,'payload.cjs':'4e2aa37bd921c81b49f3bb9a96546b288628a360dc26760ae65797d4832224a1','pilot.py':'2d7c5e44bb37f3043311962f8c05deee90e83ab6fa5ee5e3b499df238ed98200','package.json':'00dc1f0f6032fabd7a953de4eb35f29afdd84ed65266fab2992009a87ce56b6b','collector.py':'5b018f3473410644ba296ed9c8919de45f67cc67dc1ceef15683f612f1e7f8ca','frame_transport.py':'f53689d4f2389af93ad8f6a43fe0de94caf8dcaabde3eacbd13b1c957b129a76','retain-contiguous-inputs003.py':'b399ca13e8fbfee6f7633a9d430d85c8204774c2941a2e4d5f74c642820fa690'}
sha=lambda raw:hashlib.sha256(raw).hexdigest()
def bound(path,digest):
 raw=Path(path).read_bytes();assert sha(raw)==digest,f'Identity mismatch: {path}';return raw

def bounded(argv,timeout=90,limit=8*1024*1024,**kw):
 def cap():resource.setrlimit(resource.RLIMIT_FSIZE,(limit,limit))
 with tempfile.TemporaryFile() as out,tempfile.TemporaryFile() as err:
  run=subprocess.run(argv,stdout=out,stderr=err,timeout=timeout,preexec_fn=cap,**kw)
  assert run.returncode==0,'bounded command failed'
  assert out.tell()<=limit and err.tell()<=limit,'bounded command output'
  out.seek(0);return out.read()
def git(*args):return bounded(['git','-C',str(ROOT.parents[1]),*args])
def open_exclusive(path, data):
 with path.open('x',encoding='utf8') as f:
  json.dump(data,f,indent=2,default=str);f.write('\n');f.flush();os.fsync(f.fileno())
 fd=os.open(path.parent,os.O_RDONLY);os.fsync(fd);os.close(fd)

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
 assert len(tree)==ref['count']==5083
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
  raw=bounded(['/workspace/runtime/bin/cuttle','--json','sessions','read','--turns','300',sid],timeout=30)
  data=json.loads(raw);assert isinstance(data.get('turns'),list);return data['turns']
 bound(ROOT/'offline-recovery.py','5aa3161278e0d67881c51bd01205c54c095d5d8eb4fa81500c772ec868dcb73b')
 subject={'offlineRecoverySHA256':'5aa3161278e0d67881c51bd01205c54c095d5d8eb4fa81500c772ec868dcb73b','planSHA256':PLAN_SHA,'source':SOURCE,'payloadSHA256':PINS['payload.cjs'],'guardSHA256':sha(Path(__file__).read_bytes()),'collectorSHA256':PINS['collector.py'],'preflightSHA256':'b5f3c9022c47f742055951f795383d002b35cb139b04ac4edaea4cc68b7640ca','selectorSHA256':'c4766605f8fdc83f28f8768c757fd64b1a735b3366abe91d8982e211b2a247d2','sdkManifestSHA256':'32d610a2fba76f609491a4aa2c33cf7431bc6007b96d8f90364e3737ce4797dd'}
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
  if row.get('role')!='assistant' or row.get('kind')!='tool':continue
  expected='cuttle sessions send '+OWNER+" --stdin <<'EOF'\n"+msg+'\nEOF'
  if command.strip()!=expected:continue
  result=next((x for x in peer_turns[index+1:] if x.get('kind')=='tool_result' and x.get('tool',{}).get('id')==tool.get('id')),None)
  if result and result.get('tool',{}).get('ok') is True:
   sent_at=datetime.datetime.fromisoformat(row['ts'].replace('Z','+00:00'));received=datetime.datetime.fromisoformat(owner_row['ts'].replace('Z','+00:00'))
   if 0 <= (received-sent_at).total_seconds() <= 120:sent.append({'call':row,'result':result})
 assert len(sent)==1,'Missing or ambiguous reviewer-side send'
 proof={'subject':subject,'reviewer':REVIEWER,'recipient':OWNER,'recipientTurn':owner_row,'sender':sent[0]}
 return {'subject':subject,'correlatedCaptureSHA256':sha(json.dumps(proof,sort_keys=True,separators=(',',':')).encode()),'proof':proof}

def checked_pr_files(detail, rows, touched):
 count=detail.get('changed_files'); assert type(count) is int and 0<=count<3000, 'PR file count unknown or endpoint cap'
 assert isinstance(rows,list) and all(isinstance(page,list) for page in rows)
 files=[file for page in rows for file in page]; assert len(files)==count and all(isinstance(file,dict) and isinstance(file.get('filename'),str) and file['filename'] and file.get('status') in ('added','removed','modified','renamed','copied','changed','unchanged') and (file['status']!='renamed' or isinstance(file.get('previous_filename'),str) and file['previous_filename']) for file in files), 'PR files incomplete'
 assert len({file['filename'] for file in files})==len(files), 'duplicate PR paths'
 return any(file.get('filename') in touched or file.get('previous_filename') in touched for file in files)

def unrelated_receipt(old):
 assert isinstance(old,dict), 'malformed receipt'
 plan=old.get('plan',{}); assert isinstance(plan,dict), 'malformed plan'
 source=old.get('source',plan.get('source')); digest=old.get('planSHA256')
 assert (isinstance(source,str) and source) or (isinstance(digest,str) and re.fullmatch('[0-9a-f]{64}',digest)), 'unclassifiable receipt'
 assert source!=SOURCE and digest!=PLAN_SHA, 'same case receipt'
 return True

def current_fit(plan):
 repo=ROOT.parents[1]
 remote=subprocess.check_output(['git','-C',str(repo),'ls-remote','https://github.com/RSPSApp/tsps.git','refs/heads/main'],text=True,timeout=40).split()
 assert len(remote)==2 and re.fullmatch('[0-9a-f]{40}',remote[0]) and remote[1]=='refs/heads/main'
 main=remote[0]
 subprocess.run(['git','-C',str(repo),'-c','maintenance.auto=false','fetch','--no-tags','--no-write-fetch-head','https://github.com/RSPSApp/tsps.git',main],capture_output=True,check=True,timeout=90)
 connected=['server/plugins/quests/quests/TheFremennikIsles.Quest.js','server/tests/fremennik-isles-bork-and-chest.test.cjs','server/plugins/quests/QuestRuntime.js','server/plugins/quests/Quests.plugin.js','server/src/main/typescript/elvarg/plugins/PluginManager.ts','server/src/main/typescript/elvarg/plugins/PluginTypes.ts','server/src/main/typescript/elvarg/net/packet/impl/UseItemPacketListener.ts','server/src/main/typescript/elvarg/game/model/Item.ts','server/src/main/typescript/elvarg/game/content/skill/SkillManager.ts','server/src/main/typescript/elvarg/game/model/container/impl/Inventory.ts','server/src/main/typescript/elvarg/game/model/container/ItemContainer.ts','server/src/main/typescript/elvarg/game/definition/ItemDefinition.ts','server/plugins/npcs/NpcDialogues.plugin.js','server/target.txt','server/data/definitions/npc-dialogues.json','server/data/definitions/npc-dialogue-index.json']
 diff=git('diff','--name-only','fad19aaea3d66a304618a37606c4affeeffbb57c',main,'--',*connected).decode().splitlines()
 assert not diff, f'Connected main paths changed: {diff}'
 pages=json.loads(bounded(['gh','api','--paginate','--slurp','repos/RSPSApp/tsps/pulls?state=open&per_page=100'],timeout=90));assert sum(len(x) for x in pages)<=500
 assert isinstance(pages,list) and all(isinstance(page,list) for page in pages)
 prs=[pr for page in pages for pr in page];touched=set(connected);overlap=[]
 for pr in prs:
  number=pr.get('number');assert type(number) is int and number>0
  detail=json.loads(bounded(['gh','api',f'repos/RSPSApp/tsps/pulls/{number}'],timeout=90))
  rows=json.loads(bounded(['gh','api','--paginate','--slurp',f'repos/RSPSApp/tsps/pulls/{number}/files?per_page=100'],timeout=90))
  if checked_pr_files(detail,rows,touched):overlap.append(number)
 assert not overlap,f'Open PR overlaps: {overlap}'
 return {'main':main,'connectedPaths':connected,'openPRCount':len(prs),'exactOverlap':overlap}

def selected(review):
 st=(ROOT/'selection.json').stat();assert st.st_uid==os.getuid() and st.st_mode & 0o077 == 0
 raw=(ROOT/'selection.json').read_bytes();record=json.loads(raw)
 assert set(record)=={'action','owner','source','planSHA256','reviewCaptureSHA256','createdUTC'}
 assert record['action']=='SELECT_ONCE' and record['owner']==OWNER and record['source']==SOURCE and record['planSHA256']==PLAN_SHA and record['reviewCaptureSHA256']==review['correlatedCaptureSHA256']
 age=(datetime.datetime.now(datetime.timezone.utc)-datetime.datetime.fromisoformat(record['createdUTC'])).total_seconds()
 assert 0<=age<=300,'Selection stale or future'
 # Confirm an actual successful invocation by this owner, not just a JSON file.
 turns=json.loads(bounded(['/workspace/runtime/bin/cuttle','--json','sessions','read','--turns','300',OWNER],timeout=30))['turns']
 wanted='/usr/bin/python3 -E -S -B '+str(ROOT/'select.py')
 calls=[]
 for i,row in enumerate(turns):
  tool=row.get('tool',{});name=tool.get('name','')
  if row.get('role')!='assistant' or row.get('kind')!='tool' or name.strip()!=wanted:continue
  result=next((x for x in turns[i+1:] if x.get('kind')=='tool_result' and x.get('tool',{}).get('id')==tool.get('id')),None)
  if result and result.get('tool',{}).get('ok') is True:
   started=datetime.datetime.fromisoformat(row['ts'].replace('Z','+00:00'))
   created=datetime.datetime.fromisoformat(record['createdUTC'])
   if 0 <= (created-started).total_seconds() <= 30: calls.append(result)
 assert len(calls)==1,'Selection invocation not correlated'
 return {'sha256':sha(raw),'createdUTC':record['createdUTC']}

def check():
 assert sys.flags.optimize==0 and sys.flags.dont_write_bytecode
 assert ROOT.stat().st_uid==os.getuid() and ROOT.stat().st_mode & 0o077 == 0
 assert sha((ROOT/'preflight.py').read_bytes())=='b5f3c9022c47f742055951f795383d002b35cb139b04ac4edaea4cc68b7640ca'
 assert sha((ROOT/'select.py').read_bytes())=='c4766605f8fdc83f28f8768c757fd64b1a735b3366abe91d8982e211b2a247d2'
 assert sha((ROOT/'sdk-manifest.json').read_bytes())=='32d610a2fba76f609491a4aa2c33cf7431bc6007b96d8f90364e3737ce4797dd'
 req=urllib.request.Request(os.environ['CCC_BASE_URL']+'/api/acp/sessions/'+OWNER+'/config',headers=({'Authorization':'Bearer '+os.environ['CCC_TOKEN']} if os.environ['CCC_TOKEN'].startswith('ccc_pat_') else {'X-CCC-Control-Token':os.environ['CCC_TOKEN']}))
 with urllib.request.urlopen(req,timeout=20) as r: cfg=json.load(r)
 opts={x['id']:x.get('currentValue') for x in cfg['configOptions']}
 assert opts.get('model')=='gpt-6.1-terra' and opts.get('reasoning_effort')=='medium'
 assert os.environ.get('CCC_SESSION_ID')==OWNER and re.fullmatch('[a-f0-9-]{36}',REVIEWER) and REVIEWER!=OWNER
 for name,digest in PINS.items():bound(CASE/name,digest)
 plan=json.loads((CASE/'plan.json').read_bytes());held=json.loads(bound(CASE/'plan-held.json','af9f4f5147512be4192375f0ccb885b9225c491292aece84f74b400560e6c7cb'));assert held.pop('executionHold',None)=='Held for case source/profile/guard review; not selected' and held==plan
 assert plan['source']==SOURCE and plan['timeout']==1800 and plan['attempts']==1 and not plan.get('executionHold')
 assert plan['image']=='node@sha256:b04ce4ae4e95b522112c2e5c52f781471a5cbc3b594527bcddedee9bc48c03a0' and plan['flavor']=='cpu-upgrade'
 assert git('rev-parse','HEAD').decode().strip()==SOURCE and not git('status','--porcelain')
 ref=subprocess.check_output(['git','ls-remote','https://github.com/autismoyah-hub/tsps.git','refs/heads/cuttle/fremennik-noted-king-3f20'],text=True).split()
 assert len(ref)==2 and ref[0]==SOURCE
 source=validate_source(plan);review=current_review(plan);selection=selected(review);fit=current_fit(plan)
 assert not (CASE/'submission.json').exists() and not (ROOT/'submission-intent.json').exists()
 # Audit owned git-control roots; skip Git internals, reject malformed relevant receipts.
 scanned=0
 for base,dirs,files in os.walk(ROOT.parent,followlinks=False):
  scanned+=len(dirs)+len(files);assert scanned<=50000,'control scan bound'
  dirs[:]=[d for d in dirs if d not in ('objects','logs','refs','hooks') and not (Path(base)/d).is_symlink()]
  for name in files:
   if name not in ('submission.json','submission-intent.json','submitted-job-id.json'):continue
   file=Path(base)/name;old=json.loads(file.read_bytes());unrelated_receipt(old)
 # Exact-case sibling processes are not admitted. Exclude this process and ancestors.
 ancestors=set();pid=os.getpid()
 while pid>1:
  ancestors.add(pid)
  try: pid=int(Path('/proc/'+str(pid)+'/stat').read_text().rsplit(')',1)[1].split()[1])
  except (OSError,ValueError,IndexError):break
 for line in bounded(['ps','-eo','pid=,args=']).decode().splitlines():
  parts=line.strip().split(None,1)
  if len(parts)<2 or int(parts[0]) in ancestors:continue
  assert not (str(ROOT) in parts[1] and any(x in parts[1] for x in ('guard-v2.py','preflight.py','collector.py','pilot.py'))),'matching process'
 return plan,source,review,selection,fit

def main():
 parser=argparse.ArgumentParser();group=parser.add_mutually_exclusive_group(required=True);group.add_argument('--submit-once',action='store_true');group.add_argument('--precheck',action='store_true');args=parser.parse_args()
 plan,source,review,selection,fit=check()
 if args.precheck:
  print(json.dumps({'state':'PRECHECK_PASS','source':SOURCE,'planSHA256':PLAN_SHA}));return
 from huggingface_hub import HfApi
 api=HfApi();assert type(api).__name__=='ExplicitTokenApi';identity=api.whoami()['name'];assert identity=='Autismo-Yah'
 matches=[]
 for job in api.list_jobs(namespace=identity,labels={'project':'tsps'}):
  labels=getattr(job,'labels',None);job_id=getattr(job,'id',None)
  assert isinstance(job_id,str) and job_id and isinstance(labels,dict),'Unclassifiable provider listing'
  if labels.get('source')==SOURCE or labels.get('name')==plan['name']:matches.append(job_id)
 assert not matches,'Matching prior provider job: reconcile without submission'
 # Recheck all local and remote state immediately after provider reconciliation.
 again,source2,review2,selection2,fit2=check();assert again==plan and source2==source and review2==review and selection2==selection and fit2==fit
 preview=json.loads(subprocess.check_output([sys.executable,'-E','-S','-B',str(CASE/'pilot.py'),'--plan',str(CASE/'plan.json')],env={'PATH':'/usr/bin:/bin','LANG':'C.UTF-8','TZ':'UTC'}))
 request=preview['request'];assert preview['plan']==plan and request['attempts']==1 and request['timeout']==1800
 assert all(len(arg.encode())+1<131072 and '\0' not in arg for arg in request['command'])
 intent={'owner':OWNER,'collector':OWNER,'source':SOURCE,'planSHA256':PLAN_SHA,'payloadSHA256':PINS['payload.cjs'],'requestSHA256':sha(json.dumps(request,sort_keys=True,separators=(',',':')).encode()),'review':review,'selection':selection,'currentFit':fit,'sourceReadback':source,'createdUTC':datetime.datetime.now(datetime.timezone.utc).isoformat(),'state':'SUBMISSION_STARTING_RECONCILIATION_REQUIRED','noReplay':True}
 open_exclusive(ROOT/'submission-intent.json',intent)
 # Direct call once: intent is durable before the network mutation. Any error is ambiguous.
 job=api.run_job(namespace=identity,**request)
 job_id=getattr(job,'id',None);assert isinstance(job_id,str) and job_id
 open_exclusive(ROOT/'submitted-job-id.json',{'jobId':job_id,'namespace':identity,'owner':OWNER,'planSHA256':PLAN_SHA,'noReplay':True})
 returned_command=getattr(job,'command',None)
 labels=getattr(job,'labels',None);owner=getattr(getattr(job,'owner',None),'name',None)
 result={'id':job_id,'command':returned_command,'owner':owner,'docker_image':getattr(job,'docker_image',None),'flavor':getattr(job,'flavor',None),'labels':labels,'retry':getattr(job,'retry',None),'created_at':str(getattr(job,'created_at',None)),'status':str(getattr(getattr(job,'status',None),'stage',None))}
 open_exclusive(ROOT/'returned-summary.json',result)
 assert returned_command==request['command'] and owner==identity and result['docker_image']==request['image'] and result['flavor']==request['flavor'],'Returned provider identity mismatch; retain same-job custody'
 assert isinstance(labels,dict) and all(labels.get(k)==v for k,v in request['labels'].items()) and labels.get('name')==request['name'],'Returned labels mismatch; retain same-job custody'
 open_exclusive(CASE/'submission.json',{'stage':'SUBMITTED','namespace':identity,'request':request,'plan':plan,'job':result,'payloadSHA256':sha(request['command'][-1].encode())})
 # JobInfo.retry is a provider field, not proven equivalent to requested attempts.
 # Timeout is absent from JobInfo. Preserve both as unclassified rather than infer.

 print(json.dumps({'stage':'SUBMITTED','jobId':job_id,'source':SOURCE,'collector':OWNER,'noReplay':True}))
if __name__=='__main__':main()
