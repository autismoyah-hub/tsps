"""Held case-specific submit guard; read-only until explicitly invoked after review and selection."""
from pathlib import Path
import argparse, dataclasses, datetime, hashlib, json, os, subprocess, urllib.parse, urllib.request

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parent.parent
CASE=ROOT/'candidate'
OWNER='3f20dcbd-b1d9-4e4a-bc47-a3b02d6ee6bc'
REVIEWER='10e752da-405a-47bd-8d0e-21f4054a25f9'
SOURCE='57e96673bf57e6dc65de5508d60c383b90064ed2'
PLAN='a77dc6fa3f6e3c23b9e17fc6b7f0f2e5ff3726f6caec3205983071f6805c669c'
PINS={'plan.json':PLAN,'plan-held.json':'af9f4f5147512be4192375f0ccb885b9225c491292aece84f74b400560e6c7cb','payload.cjs':'4e2aa37bd921c81b49f3bb9a96546b288628a360dc26760ae65797d4832224a1','input-manifest.json':'cc162a420cdf6768da2b1928f17d7513d1b4227712d1010288d0ca5b419aad6c','pilot.py':'2d7c5e44bb37f3043311962f8c05deee90e83ab6fa5ee5e3b499df238ed98200','package.json':'ac9cb75167ec66ac1c1f4345d65bda679a20ac1b9a85aba45dd1919dcd18ac78','frame_transport.py':'f53689d4f2389af93ad8f6a43fe0de94caf8dcaabde3eacbd13b1c957b129a76','retain-job-result.py':'bdcb7f104ebfa8c26d2aa68c1dd31cf7780f12174eb92d3e98ffb928797a9071'}
def sha(b):return hashlib.sha256(b).hexdigest()
def require(ok,msg):
 if not ok:raise RuntimeError(msg)
def git(*args):return subprocess.check_output(['git','-C',str(REPO),*args])
def exclusive(path,obj):
 with path.open('x',encoding='utf8') as f:
  json.dump(obj,f,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
def check_local():
 require(os.environ.get('CCC_SESSION_ID')==OWNER,'wrong owner')
 for name,want in PINS.items():require(sha((CASE/name).read_bytes())==want,'file pin '+name)
 plan=json.loads((CASE/'plan.json').read_bytes()); held=json.loads((CASE/'plan-held.json').read_bytes()); require(held.pop('executionHold',None)=='Held for case source/profile/guard review; not selected' and held==plan,'held/proposed delta'); doc=json.loads((CASE/'input-manifest.json').read_bytes()); ref=plan['expectedInputsRef']
 require(plan['source']==SOURCE and plan['timeout']==1800 and plan['attempts']==1 and not plan.get('executionHold'),'plan')
 require(plan['image']=='node@sha256:b04ce4ae4e95b522112c2e5c52f781471a5cbc3b594527bcddedee9bc48c03a0' and plan['flavor']=='cpu-upgrade','profile')
 require(plan['tests']==['fremennik-isles-bork-and-chest.test.cjs'] and plan['expectedTestCounts']=={'fremennik-isles-bork-and-chest.test.cjs':18},'suite')
 require(git('rev-parse','HEAD').decode().strip()==SOURCE and not git('status','--porcelain'),'workspace')
 remote=subprocess.check_output(['git','ls-remote','https://github.com/autismoyah-hub/tsps.git','refs/heads/cuttle/fremennik-noted-king-3f20'],text=True).split()
 require(len(remote)==2 and remote[0]==SOURCE,'remote head')
 require(ref['source']==SOURCE and ref['sha256']==PINS['input-manifest.json'] and ref['count']==5083,'manifest ref')
 require(doc['source']==SOURCE and doc['tests']==plan['tests'] and doc['expectedTestCounts']==plan['expectedTestCounts'],'manifest binding')
 tree={}
 for row in git('ls-tree','-r','-z',SOURCE).split(b'\0'):
  if not row:continue
  meta,path=row.split(b'\t',1);mode,kind,oid=meta.decode().split();path=path.decode()
  if path.startswith('server/') or path in ('.yarnrc.yml','package.json','yarn.lock','scripts/run-with-env.mjs'):
   require(kind=='blob' and mode in ('100644','100755'),'tree mode');tree[path]=(mode,oid)
 keys=set(tree);require(len(keys)==5083 and keys==set(doc['expectedInputs'])==set(doc['sourceBlobs'])==set(doc['sourceModes']),'tree keyset')
 ordered=sorted(keys)
 for path in ordered:require(tree[path]==(doc['sourceModes'][path],doc['sourceBlobs'][path]),'tree identity '+path)
 for path in ordered:
  data=git('cat-file','blob',doc['sourceBlobs'][path])
  require(sha(data)==doc['expectedInputs'][path],'blob '+path)
 url='https://raw.githubusercontent.com/'+ref['repository']+'/'+ref['artifactCommit']+'/'+ref['path']
 class NoRedirect(urllib.request.HTTPRedirectHandler):
  def redirect_request(self,*args,**kwargs):raise RuntimeError('redirect')
 with urllib.request.build_opener(NoRedirect).open(url,timeout=60) as response:
  require(response.geturl()==url and response.status==200,'remote manifest response'); body=response.read(ref['bytes']+1)
 require(len(body)==ref['bytes'] and sha(body)==ref['sha256'],'remote manifest')
 require(not (ROOT/'submission-intent.json').exists() and not (CASE/'submission.json').exists(),'existing intent')
 for path in ROOT.rglob('submission.json'):
  old=json.loads(path.read_bytes()); oldplan=old.get('plan',{})
  require(not(oldplan.get('source')==SOURCE and set(oldplan.get('tests',[])) & set(plan['tests'])),'prior case receipt')
 return plan

def check_config():
 base=os.environ['CCC_BASE_URL']; token=os.environ['CCC_TOKEN']; url=base+'/api/acp/sessions/'+urllib.parse.quote(OWNER,safe='')+'/config'
 with urllib.request.urlopen(urllib.request.Request(url,headers={'Authorization':'Bearer '+token}),timeout=20) as r: result=json.load(r)
 options={x['id']:x.get('currentValue') for x in result['configOptions']}
 require(options.get('model')=='gpt-6.1-terra' and options.get('reasoning_effort')=='medium','config')

def review_and_selection():
 subject={'source':SOURCE,'plan':PLAN,'payload':PINS['payload.cjs'],'manifest':PINS['input-manifest.json'],'guard':sha(Path(__file__).read_bytes())}
 def matches(raw):
  transcript=json.loads(raw)
  return any(t.get('role')=='user' and REVIEWER in t.get('text','') and 'EXACT_CASE_GO' in t.get('text','') and all(x in t.get('text','') for x in subject.values()) for t in transcript['turns'])
 raw=(ROOT/'review-capture.json').read_bytes()
 require(matches(raw),'captured direct exact review absent')
 fresh=subprocess.check_output(['cuttle','--json','sessions','read','--turns','200',OWNER])
 require(matches(fresh),'fresh direct exact review absent')
 selection=json.loads((ROOT/'owner-selection.json').read_bytes())
 require(selection.get('owner')==OWNER and selection.get('subject')==subject and selection.get('choice')=='SELECT_ONCE' and selection.get('collector')==OWNER and selection.get('reviewCaptureSHA256')==sha(raw),'owner selection')

 return subject,sha(raw)

def main():
 p=argparse.ArgumentParser();p.add_argument('--submit-once',action='store_true',required=True);p.parse_args()
 check_config();plan=check_local();subject,capture=review_and_selection()
 from huggingface_hub import HfApi
 api=HfApi();require(type(api).__name__=='ExplicitTokenApi','authenticated adapter required'); identity=api.whoami()['name'];require(identity=='Autismo-Yah','namespace')
 matches=[]
 for job in api.list_jobs(namespace=identity,labels={'project':'tsps'}):
  name=getattr(job,'name',None);labels=getattr(job,'labels',None)
  require(isinstance(name,str) and isinstance(labels,dict),'unclassifiable provider listing')
  if name==plan['name'] or (labels.get('source')==SOURCE and labels.get('purpose')=='overflow-qualification'):matches.append(str(getattr(job,'id','')))
 require(not matches,'matching prior job')
 check_config();require(check_local()==plan,'local drift');subject2,capture2=review_and_selection();require(subject2==subject,'review drift')
 preview=json.loads(subprocess.check_output([sys.executable,str(CASE/'pilot.py'),'--plan',str(CASE/'plan.json')]))
 request=preview['request'];require(preview['plan']==plan and request['attempts']==1 and request['timeout']==1800,'preview')
 require(all('\0' not in arg and len(arg.encode())+1<131072 for arg in request['command']),'argv')
 intent={'owner':OWNER,'collector':OWNER,'subject':subject,'reviewCaptureSHA256':capture2,'requestSHA256':sha(json.dumps(request,sort_keys=True,separators=(',',':')).encode()),'createdUTC':datetime.datetime.now(datetime.timezone.utc).isoformat(),'state':'SUBMISSION_STARTING_RECONCILIATION_REQUIRED','noReplay':True}
 exclusive(ROOT/'submission-intent.json',intent)
 job=api.run_job(namespace=identity,**request)
 result=dataclasses.asdict(job);require(result.get('id'),'job id missing')
 exclusive(CASE/'submission.json',{'stage':'SUBMITTED','namespace':identity,'request':request,'plan':plan,'job':result,'payloadSHA256':sha(request['command'][-1].encode())})
 require(result.get('command')==request['command'],'returned command mismatch')
 print(json.dumps({'stage':'SUBMITTED','jobId':result['id'],'source':SOURCE,'collector':OWNER,'noReplay':True}))
if __name__=='__main__':main()
