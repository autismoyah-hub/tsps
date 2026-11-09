"""Finite local control, no provider requests or game imports."""
import hashlib,importlib.util,json,tempfile
from pathlib import Path
from huggingface_hub._jobs_api import JobInfo
root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('guard',root/'guard-v2.py');g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
job=JobInfo(id='fixture-id',createdAt='2026-11-09T00:00:00Z',startedAt='2026-11-09T00:01:00Z',dockerImage='node@example',command=['node','-e','fixture'],flavor='cpu-upgrade',labels={'project':'tsps','name':'fixture'},status={'stage':'RUNNING'},owner={'id':'fixture','name':'Autismo-Yah','type':'org'},retry=0)
result={'id':job.id,'command':job.command,'owner':job.owner.name,'docker_image':job.docker_image,'flavor':job.flavor,'labels':job.labels,'retry':job.retry,'created_at':str(job.created_at),'status':str(job.status.stage)}
with tempfile.TemporaryDirectory() as d:
 p=Path(d)/'receipt.json';g.open_exclusive(p,result); assert json.loads(p.read_bytes())==result
 try:g.open_exclusive(p,result)
 except FileExistsError:pass
 else:raise AssertionError('duplicate receipt accepted')
 q=Path(d)/'partial.json';q.write_text('{')
 try:g.open_exclusive(q,result)
 except FileExistsError:pass
 else:raise AssertionError('partial receipt overwritten')
print('real JobInfo datetime summary serialized; duplicate and partial receipt refused')

# Synthetic reviewer/selection provenance: recipient text alone is insufficient.
import datetime
from unittest.mock import patch
subject={'overlapEvidenceSHA256':'d573e1c986fa0e92ab08c15a8cb46ac0d60ec43c642c9fd2062072987849c863','offlineRecoverySHA256':'5aa3161278e0d67881c51bd01205c54c095d5d8eb4fa81500c772ec868dcb73b','planSHA256':g.PLAN_SHA,'source':g.SOURCE,'payloadSHA256':g.PINS['payload.cjs'],'guardSHA256':g.sha((root/'guard-v2.py').read_bytes()),'collectorSHA256':g.PINS['collector.py'],'preflightSHA256':'b5f3c9022c47f742055951f795383d002b35cb139b04ac4edaea4cc68b7640ca','selectorSHA256':'a218d6d594c2133cc9411ca653d591566ab3abcf1de7fab68d7ddf71e43e9235','sdkManifestSHA256':'32d610a2fba76f609491a4aa2c33cf7431bc6007b96d8f90364e3737ce4797dd'}
message='EXACT_CASE_GO '+' '.join(subject.values())
stamp=datetime.datetime.now(datetime.timezone.utc).isoformat()
owner={'turns':[{'role':'user','ts':stamp,'text':'[From Cuttle session "peer" · '+g.REVIEWER+']\n'+message}]}
peer={'turns':[{'role':'assistant','kind':'tool','ts':stamp,'tool':{'name':'cuttle sessions send '+g.OWNER+" --stdin <<'EOF'\n"+message+'\nEOF','id':'t1'}},{'role':'user','kind':'tool_result','tool':{'id':'t1','ok':True}}]}
def stub(args,**kwargs):return json.dumps(owner if args[-1]==g.OWNER else peer).encode()
with tempfile.TemporaryDirectory() as d:
 g.ROOT=Path(d);shutil_placeholder=None
 import shutil as sh;sh.copyfile(root/'offline-recovery.py',g.ROOT/'offline-recovery.py');sh.copyfile(root/'reviewed-overlaps.json',g.ROOT/'reviewed-overlaps.json')
 with patch.object(g,'bounded',side_effect=stub): review=g.current_review(None)
assert review['subject']==subject
g.ROOT=root
bad={'turns':[]}
def missing(args,**kwargs):return json.dumps(owner if args[-1]==g.OWNER else bad).encode()
with patch.object(g,'bounded',side_effect=missing):
 try:g.current_review(None)
 except AssertionError:pass
 else:raise AssertionError('forged recipient accepted')
spoof={'turns':[{'role':'assistant','kind':'tool','ts':stamp,'tool':{'name':'echo '+peer['turns'][0]['tool']['name'],'id':'t1'}},peer['turns'][1]]}
def inert(args,**kwargs):return json.dumps(owner if args[-1]==g.OWNER else spoof).encode()
with patch.object(g,'bounded',side_effect=inert):
 try:g.current_review(None)
 except AssertionError:pass
 else:raise AssertionError('inert sender accepted')
with tempfile.TemporaryDirectory() as d:
 g.ROOT=Path(d); now=datetime.datetime.now(datetime.timezone.utc); r={'action':'SELECT_ONCE','owner':g.OWNER,'source':g.SOURCE,'planSHA256':g.PLAN_SHA,'reviewCaptureSHA256':review['correlatedCaptureSHA256'],'createdUTC':now.isoformat()}; f=g.ROOT/'selection-v2.json';f.write_text(json.dumps(r));f.chmod(0o600)
 cmd='/usr/bin/python3 -E -S -B '+str(g.ROOT/'select-v2.py'); turns={'turns':[{'role':'assistant','kind':'tool','ts':(now-datetime.timedelta(seconds=1)).isoformat(),'tool':{'name':cmd,'id':'s1'}},{'role':'user','kind':'tool_result','tool':{'id':'s1','ok':True},'text':''}]}
 with patch.object(g,'bounded',return_value=json.dumps(turns).encode()):assert g.selected(review)['sha256']==g.sha(f.read_bytes())
 with patch.object(g,'bounded',return_value=b'{"turns":[]}'):
  try:g.selected(review)
  except AssertionError:pass
  else:raise AssertionError('unobserved selection accepted')
print('synthetic two-sided review and successful selection call accepted; forged/missing/inert sender refused')

# Real collector module, synthetic JobInfo and private temporary receipts; no network.
import os,shutil
spec=importlib.util.spec_from_file_location('collector',root/'collector.py');c=importlib.util.module_from_spec(spec);spec.loader.exec_module(c)
with tempfile.TemporaryDirectory() as d:
 c.ROOT=Path(d);c.CASE=c.ROOT/'candidate';c.CASE.mkdir()
 for name in ('plan.json','payload.cjs','retain-contiguous-inputs003.py'):shutil.copyfile(root/name,c.CASE/name)
 plan=json.loads((c.CASE/'plan.json').read_bytes());request={'name':plan['name'],'image':plan['image'],'flavor':plan['flavor'],'command':['node','-e','fixture'],'labels':{'project':'tsps','source':plan['source'],'purpose':'overflow-qualification'}}
 labels={**request['labels'],'name':request['name']};jobrow={'id':'fixture-id','command':request['command'],'owner':'Autismo-Yah','docker_image':request['image'],'flavor':request['flavor'],'labels':labels}
 receipt={'plan':plan,'namespace':'Autismo-Yah','request':request,'job':jobrow,'payloadSHA256':c.digest(b'fixture')};intent={'owner':c.OWNER,'collector':c.OWNER,'planSHA256':c.PLAN,'payloadSHA256':c.digest((c.CASE/'payload.cjs').read_bytes()),'requestSHA256':c.digest(json.dumps(request,sort_keys=True,separators=(',',':')).encode())}
 for path,obj in [(c.CASE/'submission.json',receipt),(c.ROOT/'submission-intent.json',intent),(c.ROOT/'submitted-job-id.json',{'jobId':'fixture-id','planSHA256':c.PLAN})]:path.write_text(json.dumps(obj))
 class ExplicitTokenApi:
  def __init__(self,job):self.job=job
  def whoami(self):return {'name':'Autismo-Yah'}
  def inspect_job(self,**kwargs):return self.job
  def fetch_job_logs(self,**kwargs):raise AssertionError('must not fetch')
 def info(stage='RUNNING',image=None,lab=None):return JobInfo(id='fixture-id',createdAt='2026-11-09T00:00:00Z',dockerImage=image or request['image'],command=request['command'],flavor=request['flavor'],labels=labels if lab is None else lab,status={'stage':stage},owner={'id':'fixture','name':'Autismo-Yah','type':'org'})
 import huggingface_hub
 with patch.dict(os.environ,{'CCC_SESSION_ID':c.OWNER}),patch.object(c.sys,'argv',['collector']):
  with patch.object(huggingface_hub,'HfApi',lambda:ExplicitTokenApi(info())):c.main()
  assert not (c.ROOT/'collection-intent.json').exists()
  for wrong in (info(image='wrong'),info(lab={})):
   with patch.object(huggingface_hub,'HfApi',lambda:ExplicitTokenApi(wrong)):
    try:c.main()
    except ValueError:pass
    else:raise AssertionError('wrong provider tuple accepted')
  assert not (c.ROOT/'collection-intent.json').exists()
  (c.ROOT/'collection-intent.json').write_text('{}')
  with patch.object(c.sys,'argv',['collector','--recover-offline']):
   try:c.main()
   except ValueError:pass
   else:raise AssertionError('missing raw accepted')
   stamp='20261109T000000Z';status=c.CASE/f'status-{stamp}.json';log=c.CASE/f'logs-{stamp}.txt';status.write_text(json.dumps({'id':'fixture-id','status':{'stage':'COMPLETED'},'command':request['command']}));log.write_text('malformed raw\n')
   (c.CASE/'input-manifest.json').write_text('{}')
   (c.ROOT/'iterator-ended.json').write_text(json.dumps({'jobId':'fixture-id','stamp':stamp,'stage':'COMPLETED','statusSHA256':c.digest(status.read_bytes()),'rawSHA256':c.digest(log.read_bytes())}))
   try:c.main()
   except ValueError:pass
   else:raise AssertionError('mismatched collection intent accepted')
   assert not (c.ROOT/'offline-retain-intent.json').exists()
   (c.ROOT/'collection-intent.json').write_text(json.dumps({'jobId':'fixture-id','collector':c.OWNER,'planSHA256':c.PLAN,'stamp':stamp,'state':'COLLECTION_STARTING_NO_REPLAY'}))
   try:c.main()
   except ValueError:pass
   else:raise AssertionError('malformed native raw accepted')
   assert log.read_text()=='malformed raw\n' and (c.ROOT/'offline-retain-intent.json').exists()
   try:c.main()
   except ValueError:pass
   else:raise AssertionError('offline recovery replay accepted')
print('synthetic collector nonterminal, wrong tuple, partial raw, retainer failure and no-replay controls passed')

# Actual native retainer invocation on synthetic input-event bytes with the real pinned manifest.
with tempfile.TemporaryDirectory() as d:
 c.ROOT=Path(d);c.CASE=c.ROOT/'candidate';c.CASE.mkdir()
 for name in ('plan.json','payload.cjs','retain-contiguous-inputs003.py','input-manifest.json'):shutil.copyfile(root/name,c.CASE/name)
 plan=json.loads((c.CASE/'plan.json').read_bytes()); manifest=json.loads((c.CASE/'input-manifest.json').read_bytes()); command=['node','-e','fixture']; stamp='20261109T000000Z'
 (c.CASE/'submission.json').write_text(json.dumps({'plan':plan,'job':{'id':'fixture-id','command':command},'request':{'command':command}}))
 status=c.CASE/f'status-{stamp}.json';status.write_text(json.dumps({'id':'fixture-id','status':{'stage':'COMPLETED'},'command':command}))
 log=c.CASE/f'logs-{stamp}.txt';log.write_text(json.dumps({'event':'inputs','source':plan['source'],'inputs':manifest['expectedInputs']},separators=(',',':'))+'\n')
 (c.ROOT/'iterator-ended.json').write_text(json.dumps({'jobId':'fixture-id','stamp':stamp,'stage':'COMPLETED','statusSHA256':c.digest(status.read_bytes()),'rawSHA256':c.digest(log.read_bytes())}))
 with patch.object(c.subprocess,'run',wraps=c.subprocess.run) as invoked:
  with patch.dict(os.environ,{'CCC_TOKEN':'fixture-secret'}): c.retain(stamp,False)
  assert invoked.call_args.kwargs['env']=={'PATH':'/usr/bin:/bin'}
 assert json.loads((c.ROOT/'native-retain-result.json').read_bytes())['exit']==0
 assert (c.CASE/f'native-input-{stamp}'/'fragment-receipt.json').is_file()
print('actual offline native retainer accepted synthetic full input event')

assert g.checked_pr_files({'changed_files':1},[[{'filename':'new','status':'renamed','previous_filename':'old'}]],{'old'})
for detail,rows in [({'changed_files':3000},[[]]),({'changed_files':2},[[{'filename':'x','status':'modified'}]]),({},[[]])]:
 try:g.checked_pr_files(detail,rows,{'x'})
 except AssertionError:pass
 else:raise AssertionError('incomplete PR list accepted')
print('PR cap, truncation, missing count and renamed-path controls passed')

assert c.classify_stage('SCHEDULING')=='NONTERMINAL' and c.classify_stage('COMPLETED')=='TERMINAL'
for stage in (None,'DELETED','UNKNOWN','CANCELLED'):
 try:c.classify_stage(stage)
 except ValueError:pass
 else:raise AssertionError('unsupported stage accepted')
print('recognized active and unsupported stage controls passed')

for detail,rows in [({'changed_files':1},[[{'filename':'x','status':'renamed'}]]),({'changed_files':2},[[{'filename':'x','status':'modified'},{'filename':'x','status':'modified'}]])]:
 try:g.checked_pr_files(detail,rows,set())
 except AssertionError:pass
 else:raise AssertionError('malformed rename or duplicate accepted')
assert g.unrelated_receipt({'planSHA256':'a'*64})
for receipt in ({'planSHA256':g.PLAN_SHA},{},{'plan':[]},{'source':'b'*40,'plan':{'source':g.SOURCE}},{'plan':{'source':g.SOURCE}},{'source':'invalid'},{'source':'b'*40,'planSHA256':'invalid'}):
 try:g.unrelated_receipt(receipt)
 except AssertionError:pass
 else:raise AssertionError('same or unclassifiable receipt accepted')
print('rename, duplicate, sibling same-plan and malformed receipt controls passed')

with tempfile.TemporaryDirectory() as d:
 g.ROOT=Path(d); (g.ROOT/'offline-recovery.py').write_text('tampered'); shutil.copyfile(root/'reviewed-overlaps.json',g.ROOT/'reviewed-overlaps.json')
 try:g.current_review(None)
 except AssertionError:pass
 else:raise AssertionError('altered offline launcher accepted')
print('altered launcher refused; native retainer environment excludes credentials')

assert g.unrelated_receipt({'source':'b'*40,'plan':{'source':'b'*40},'planSHA256':'c'*64})
print('conflicting, nested and malformed identity controls passed')

for marker in ('submission.json','submission-intent.json','submitted-job-id.json','submission.intent','submit.intent','submitted.json'):
 assert marker in g.RECEIPT_MARKERS
 for receipt in ({'source':g.SOURCE},{}, {'planSHA256':g.PLAN_SHA}):
  try:g.unrelated_receipt(receipt)
  except AssertionError:pass
  else:raise AssertionError('same or unknown marker accepted: '+marker)
 assert g.unrelated_receipt({'source':'b'*40})
print('all six marker names apply same-case/unknown refusals and unrelated positive')

reviewed=json.loads((root/'reviewed-overlaps.json').read_bytes())
for number,head,base in [(3948,'25a959471d0809d355f8a3024d38cc0511592ca4','5bbbd1d57b87956cac1e0ca7785dff3e23c0f4e9'),(3681,'bddfc726a4567f9317015ff5fe015d21f0cef972','d9d2c95eb39596b32ea465a1808b3962fc2461b8')]:
 rows=[reviewed[str(number)]]; touched={x['filename'] for x in rows[0]}; detail={'state':'open','head':{'sha':head},'base':{'sha':base}}
 assert g.reviewed_overlap(number,detail,rows,touched)
 for wrong in ({'state':'open','head':{'sha':'0'*40},'base':{'sha':base}},{'state':'open','head':{'sha':head},'base':{'sha':'0'*40}}):
  try:g.reviewed_overlap(number,wrong,rows,touched)
  except AssertionError:pass
  else:raise AssertionError('changed PR accepted')
 changed=[rows[0][:-1]]
 try:g.reviewed_overlap(number,detail,changed,touched)
 except AssertionError:pass
 else:raise AssertionError('changed PR paths accepted')
 try:g.reviewed_overlap(1234,detail,rows,touched)
 except AssertionError:pass
 else:raise AssertionError('unreviewed overlap accepted')
print('exact PR overlap accepted; changed head/base/path and extra PR refused')
