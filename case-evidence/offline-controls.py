"""Offline control-only exercise of authority parser and guard's refusal boundary; no game/provider."""
import hashlib, importlib.util, json, os, pathlib, sys, tempfile, types
from unittest.mock import patch
HERE=pathlib.Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[1]/'sdk'/'site-packages'))
def load(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
def h(path):return hashlib.sha256(path.read_bytes()).hexdigest()
auth=load('wc_authority',HERE/'authority.py'); guard=load('wc_guard',HERE/'provider_guard.py')
owner=guard.OWNER; reviewer='ed3a4125-645c-4266-8b86-4e199dd5bc63'; mc='eaae9ca0-2f2b-4c05-85cd-c3e70178aeac'
def transcript(value):return json.dumps({'turns':[{'role':'assistant','kind':'text','ts':'2026-11-09T00:00:00Z','text':json.dumps(value)}]}).encode()
def expect_refuse(fn):
 try:fn()
 except (ValueError,AssertionError,RuntimeError):return
 raise AssertionError('expected refusal')
# Real accepted parser, exact distinct turns: wrong and duplicate turn must refuse.
for role,session in [('owner',owner),('peer',reviewer),('mc',mc)]:
 v={'role':role,'generation':'current'}; assert auth.exact_turn(transcript(v),session,v)['session']==session
 expect_refuse(lambda:auth.exact_turn(transcript({'role':role,'generation':'old'}),session,v))
 t=json.loads(transcript(v));t['turns'].append(t['turns'][0]);expect_refuse(lambda:auth.exact_turn(json.dumps(t).encode(),session,v))
 print(role,'wrong/duplicate turn controls PASS')
# Actual provider_guard.main, synthetic non-game inputs and mocked external boundaries.
# Two refreshes: first passes; after provider scan second refuses. No intent/pilot may occur.
with tempfile.TemporaryDirectory(prefix='wc-control-') as td:
 root=pathlib.Path(td)/'before';root.mkdir();(root.parent/'mc-unique-turns.json').write_bytes((HERE.parent/'mc-unique-turns.json').read_bytes()); (root/'authority.py').write_bytes((HERE/'authority.py').read_bytes());(root/'prestart.py').write_text('# inert snapshot\n');(root/'pilot.py').write_text("raise RuntimeError('PILOT_REACHED')\n")
 plan={'source':guard.SOURCE,'name':guard.NAME,'attempts':1,'timeout':1800,'expectedInputsRef':{'sha256':'a'*64}}
 pp=root/'pilot.json';pp.write_text(json.dumps(plan));sel={'owner':owner,'action':'SELECT_ONCE','attempts':1,'source':guard.SOURCE,'planSHA256':h(pp)};sp=root/'selection.json';sp.write_text(json.dumps(sel))
 files={n:h(root/n) for n in ('authority.py','prestart.py','pilot.py')}
 subject={'owner':owner,'source':guard.SOURCE,'planSHA256':h(pp),'manifestSHA256':'a'*64,'connectedDiffSHA256':'b5f60a95b4c4d1d066a87dbf255b16ea0fcf49133e1f31127ffc4d5fed8acc4c','files':files,'authAdapterSHA256':'fab4bc92dc8601143ce61e910f8121774c22a0d19be6d2617bba9449f7131326','authHelperSHA256':'00ff45d75be1eae3bc8f04995cfb8bcbe2bc5adc28c1162b843cfe88c87024f9'}
 identity=json.loads((HERE.parent/'mc-authority-value.json').read_bytes()); assert hashlib.sha256(auth.canonical(identity)).hexdigest()=='4e3369be85775623307b4d51186e2fb5bfa996899ed2046b3b61aba859574cd2'
 review={'reviewer':reviewer,'verdict':'EXACT_INPUT_PROFILE_GO','subject':subject}; values={'peer':(reviewer,review),'owner':(owner,sel),'mc':(mc,identity)}
 pre={'source':guard.SOURCE,'owner':owner,'planSHA256':h(pp),'selectionSHA256':h(sp),'createdUTC':__import__('datetime').datetime.now(__import__('datetime').timezone.utc).isoformat(),'files':files,'authoritySubject':subject,'identityValue':identity,'forkRef':'fixture','connectedDifferences':[],'model':'gpt-6.1-terra','effort':'medium','reviewPath':str(root/'review.json'),'reviewSHA256':'b'*64,'identityPath':str(root/'identity.json'),'identitySHA256':'c'*64}
 for label,(session,value) in values.items():pre[label+'Proof']=auth.exact_turn((root.parent/'mc-unique-turns.json').read_bytes() if label=='mc' else transcript(value),session,value)
 pr=root/'prestart-receipt.json';pr.write_text(json.dumps(pre));calls=[]
 class API:
  def whoami(self):return {'name':'Autismo-Yah'}
  def list_jobs(self,namespace):calls.append('scan');return []
 def readturn(cmd,**kw):
  session=cmd[4];return next(transcript(value) for _,(sid,value) in values.items() if sid==session)
 def run(cmd,**kw):
  calls.append('refresh'); out=pathlib.Path(cmd[cmd.index('--output')+1]);
  if calls.count('refresh')==2:return types.SimpleNamespace(returncode=70,stdout=b'',stderr=b'changed ref or diff')
  out.write_text(json.dumps(pre));return types.SimpleNamespace(returncode=0,stdout=b'OK',stderr=b'')
 argv=['provider_guard.py','--plan',str(pp),'--plan-sha256',h(pp),'--prestart',str(pr),'--prestart-sha256',h(pr),'--selection',str(sp),'--selection-sha256',h(sp)]
 with patch.object(guard,'ROOT',root),patch.object(guard,'HfApi',API),patch.object(guard.subprocess,'check_output',side_effect=readturn),patch.object(guard.subprocess,'run',side_effect=run),patch.object(sys,'argv',argv),patch.dict(os.environ,{'CCC_SESSION_ID':owner}):
  expect_refuse(guard.main)
 assert calls==['refresh','scan','refresh'] and not (root/'exclusive-intent.json').exists() and not (root/'submission.json').exists(),calls
 assert list(root.glob('guard-prestart-*-stderr.txt'))
 print('post-scan refresh failure before intent/pilot PASS; retained diagnostic')
# Execute real prestart.main with seeded control inputs, stubbing only source/config/OS boundaries.
# The point is to reach the actual ref/diff checks and show their changed values refuse.
prestart=load('wc_prestart',HERE/'prestart.py')
with tempfile.TemporaryDirectory(prefix='wc-prestart-control-') as td:
 base=pathlib.Path(td); root=base/'case'/'before';root.mkdir(parents=True);sdk=base/'sdk';sdk.mkdir();(sdk/'setup-receipt.json').write_text(json.dumps({'SDK':'2.4.0'}))
 pp=root/'plan.json';pp.write_text(json.dumps({'source':guard.SOURCE,'name':guard.NAME,'attempts':1,'timeout':1800}));sp=root/'selection.json';sp.write_text(json.dumps({'owner':owner,'action':'SELECT_ONCE','attempts':1,'source':guard.SOURCE,'planSHA256':h(pp)}))
 ident=root/'identity.json';ident.write_bytes((HERE.parent/'mc-authority-value.json').read_bytes());review=root/'review.json';review.write_text('{}')
 log=root/'turns.jsonl';log.write_text('\n'.join(json.dumps(x) for x in [{'type':'session_meta','payload':{'session_id':'synthetic'}},{'type':'turn_context','payload':{'model':'gpt-6.1-terra','effort':'medium'}}])+'\n')
 class Dist: metadata={'Name':'huggingface_hub'};version='2.4.0'
 config=json.dumps({'configOptions':[{'id':'model','currentValue':'gpt-6.1-terra'},{'id':'reasoning_effort','currentValue':'medium'}]}).encode()
 for scenario in ('changed-ref','changed-diff'):
  output=root/(scenario+'.json');calls=[]
  def git(cmd,**kwargs):
   calls.append(cmd)
   if cmd[:2]==['git','ls-remote']:return ('0'*40+' '+prestart.REF+'\n') if scenario=='changed-ref' else (guard.SOURCE+' '+prestart.REF+'\n')
   if cmd[:3]==['git','cat-file','-t']:return 'commit\n'
   if cmd[:3]==['git','rev-parse','origin/main']:return '1'*40+'\n'
   if cmd[:3]==['git','diff','--name-only']:return 'server/src/main/typescript/elvarg/game/entity/impl/object/ObjectManager.ts\nserver/src/main/typescript/elvarg/game/model/container/shop/ShopManager.ts\nserver/src/main/typescript/elvarg/game/task/impl/TimedObjectReplacementTask.ts\n'
   if cmd[:2]==['git','diff']:return b'unreviewed connected hunk'
   raise AssertionError(cmd)
  argv=['prestart.py','--plan',str(pp),'--selection',str(sp),'--output',str(output),'--review',str(review),'--review-sha256',h(review),'--identity',str(ident),'--identity-sha256',h(ident)]
  with patch.object(prestart,'ROOT',root),patch.object(prestart,'REPO',base),patch.object(prestart,'read_exact_route',return_value=config),patch.object(prestart.importlib.metadata,'distributions',return_value=[Dist()]),patch.object(prestart.glob,'glob',return_value=[str(log)]),patch.object(prestart.subprocess,'check_output',side_effect=git),patch.object(prestart.subprocess,'run',return_value=types.SimpleNamespace(returncode=0)),patch.object(sys,'argv',argv),patch.dict(os.environ,{'CCC_SESSION_ID':owner,'CODEX_SESSION_ID':'synthetic','CCC_TOKEN':'ccc_pat_test','CCC_BASE_URL':'https://example.invalid'}):
   expect_refuse(prestart.main)
  assert not output.exists() and any(c[:2]==['git','ls-remote'] for c in calls)
  if scenario=='changed-diff':assert any(c[:2]==['git','diff'] for c in calls)
  print(scenario,'actual prestart refusal before receipt PASS')
# Actual route reader refuses redirects, denial and overlong responses under controlled fake opener.
import contextlib,io,urllib.request,urllib.error
assert prestart.NoRedirect().redirect_request(None,None,301,'','',None) is None
class FakeResponse:
 status=200
 def __init__(self,url,data):self.url=url;self.data=data
 def __enter__(self):return self
 def __exit__(self,*a):pass
 def geturl(self):return self.url
 def read(self,n):return self.data[:n]
class Opener:
 def __init__(self,result):self.result=result
 def open(self,req,timeout):
  if isinstance(self.result,Exception):raise self.result
  return self.result
req=urllib.request.Request('https://example.invalid/config',headers={'Authorization':'Bearer redacted-control'})
for code in (301,403):
 err=urllib.error.HTTPError(req.full_url,code,'refused',{},None);out=io.StringIO()
 with patch.object(prestart.urllib.request,'build_opener',return_value=Opener(err)),contextlib.redirect_stderr(out):
  try:prestart.read_exact_route(req,16);raise AssertionError('route accepted')
  except urllib.error.HTTPError as e:assert e.code==code
 assert str(code) in out.getvalue() and 'redacted-control' not in out.getvalue()
with patch.object(prestart.urllib.request,'build_opener',return_value=Opener(FakeResponse(req.full_url,b'x'*17))):expect_refuse(lambda:prestart.read_exact_route(req,16))
with patch.object(prestart.urllib.request,'build_opener',return_value=Opener(FakeResponse('https://example.invalid/other',b'x'))):expect_refuse(lambda:prestart.read_exact_route(req,16))
print('route redirects/denial/size/final URL controls PASS')
# Real matcher reconstructs literal pilot request from exact plan and payload.
matcher=load('wc_job_identity',HERE/'job_identity.py')
import copy,dataclasses
payload=(HERE/'payload.cjs').read_bytes();fullplan={'source':guard.SOURCE,'name':guard.NAME,'payloadFile':'payload.cjs','image':'reviewed-image','flavor':'cpu-upgrade','timeout':1800,'attempts':1,'scope':'synthetic'}
rendered=payload.decode().replace('__PLAN__',json.dumps(fullplan));req={k:fullplan[k] for k in ('name','image','flavor','timeout','attempts')};req.update(command=['node','-e',rendered],labels={'project':'tsps','source':guard.SOURCE,'purpose':'overflow-qualification'})
job={'id':'job1','docker_image':req['image'],'flavor':req['flavor'],'command':req['command'],'labels':req['labels'],'owner':{'name':'Autismo-Yah'}}
submission={'namespace':'Autismo-Yah','request':req,'job':job,'plan':fullplan,'payloadSHA256':hashlib.sha256(rendered.encode()).hexdigest()}
assert matcher.verify_job(copy.deepcopy(job),copy.deepcopy(submission),fullplan,payload)=='job1'
for field,value in [('command',['node','different']),('docker_image','wrong'),('labels',{'source':'wrong'}),('owner',{'name':'wrong'}),('id','wrong')]:
 bad=copy.deepcopy(job);bad[field]=value;expect_refuse(lambda:matcher.verify_job(bad,copy.deepcopy(submission),fullplan,payload))
bad=copy.deepcopy(submission);bad['request']['name']='wrong';expect_refuse(lambda:matcher.verify_job(copy.deepcopy(job),bad,fullplan,payload))
bad=copy.deepcopy(submission);bad['request']['image']='wrong';bad['request']['command']=['node','-e',guard.NAME];bad['job']['docker_image']='wrong';bad['job']['command']=bad['request']['command'];inspected=copy.deepcopy(bad['job']);expect_refuse(lambda:matcher.verify_job(inspected,bad,fullplan,payload))
print('literal reconstructed request wrong image/command/name and job identity controls PASS')
collector=load('wc_collector_entry',HERE/'collector_entry.py')
@dataclasses.dataclass
class FakeJob:
 id:str;docker_image:str;flavor:str;command:list;labels:dict;owner:dict;status:object
@dataclasses.dataclass
class FakeStatus:stage:str
for scenario in ('command','docker_image','labels','owner','name','partial','positive'):
 with tempfile.TemporaryDirectory(prefix='wc-collector-control-') as td:
  root=pathlib.Path(td)
  for n in ('job_identity.py','pilot.py','payload.cjs'):(root/n).write_bytes((HERE/n).read_bytes())
  planpath=root/'pilot.json';planpath.write_text(json.dumps(fullplan));sub=copy.deepcopy(submission);status=copy.deepcopy(job);status['status']={'stage':'COMPLETED'};statuspath=root/'terminal.json';statuspath.write_text(json.dumps(status));calls=[];bad=copy.deepcopy(status)
  if scenario=='command':bad['command']=['node','different']
  elif scenario=='docker_image':bad['docker_image']='wrong'
  elif scenario=='labels':bad['labels']={'source':'wrong'}
  elif scenario=='owner':bad['owner']={'name':'wrong'}
  elif scenario=='name':sub['request']['name']='wrong'
  (root/'submission.json').write_text(json.dumps(sub))
  class FakeAPI:
   def __init__(self):calls.append('api')
   def whoami(self):return {'name':'Autismo-Yah'}
   def inspect_job(self,job_id,namespace):calls.append('inspect');return FakeJob(**{**bad,'status':FakeStatus('COMPLETED')})
   def fetch_job_logs(self,*a,**kw):
    calls.append('logs')
    if scenario=='partial':
     yield 'partial-line';raise RuntimeError('synthetic partial')
    return
  frame=types.ModuleType('frame_transport');frame.reconstruct_child_outputs=lambda *a:None
  argv=['collector_entry.py','--plan',str(planpath),'--plan-sha256',h(planpath),'--status',str(statuspath),'--status-sha256',h(statuspath)]
  with patch.object(collector,'ROOT',root),patch.object(collector.huggingface_hub,'HfApi',FakeAPI),patch.object(sys,'argv',argv),patch.dict(sys.modules,{'frame_transport':frame}),contextlib.redirect_stdout(io.StringIO()):
   if scenario=='positive':collector.main()
   else:expect_refuse(collector.main)
  if scenario=='name':assert calls==[],calls
  elif scenario in ('command','docker_image','labels','owner'):assert calls==['api','inspect'] and len(list(root.glob('collector-inspect-*.json')))==1,calls
  elif scenario=='partial':
   paths=list(root.glob('collector-raw-stream-*.txt'));assert calls==['api','inspect','logs'] and len(paths)==1 and paths[0].read_text()=='partial-line\n'
  else:assert calls==['api','inspect','logs'],calls
  print('collector',scenario,'control PASS')
# Actual status invocation refuses a modified entry even if the caller supplies its new digest.
statusinvoke=load('wc_status_invoke',HERE/'status_invoke.py')
with tempfile.TemporaryDirectory(prefix='wc-status-entry-control-') as td:
 root=pathlib.Path(td);entry=root/'status.py';entry.write_text('modified status entry')
 pre={'owner':owner,'planSHA256':'a'*64,'files':{'status.py':'b'*64}};prepath=root/'prestart.json';prepath.write_text(json.dumps(pre));(root/'exclusive-intent.json').write_text(json.dumps({'owner':owner,'planSHA256':'a'*64,'prestartSHA256':h(prepath)}));calls=[]
 argv=['status_invoke.py','--plan',str(root/'pilot.json'),'--plan-sha256','a'*64,'--entry-sha256',h(entry),'--prestart',str(prepath),'--prestart-sha256',h(prepath)]
 with patch.object(statusinvoke,'ROOT',root),patch.object(statusinvoke.subprocess,'run',side_effect=lambda *a,**k:calls.append('adapter')),patch.object(sys,'argv',argv),patch.dict(os.environ,{'CCC_SESSION_ID':owner}):expect_refuse(statusinvoke.main)
 assert calls==[];print('altered status entry with matching caller hash refused before adapter PASS')
# Bind each helper reference to actual accepted artifact bytes, not a transcribed digest.
helper=HERE.parents[1]/'control-cache'/'hf-author-control'/'004'/'files'/'authenticated-child-snapshot-command.py';actual=h(helper);assert len(actual)==64
for name in ('prestart.py','provider_guard.py','invoke.py','status_invoke.py','collect.py'):
 text=(HERE/name).read_text();assert actual in text
print('all helper pins match verified artifact bytes PASS')
