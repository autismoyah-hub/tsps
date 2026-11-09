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
subject={'planSHA256':g.PLAN_SHA,'source':g.SOURCE,'payloadSHA256':g.PINS['payload.cjs'],'guardSHA256':g.sha((root/'guard-v2.py').read_bytes()),'collectorSHA256':g.PINS['collector.py'],'preflightSHA256':'55120973ff22dbb93c42b0c1e27f7e61081c04025636e9d142306ff19e92e36a','selectorSHA256':'c4766605f8fdc83f28f8768c757fd64b1a735b3366abe91d8982e211b2a247d2','sdkManifestSHA256':'32d610a2fba76f609491a4aa2c33cf7431bc6007b96d8f90364e3737ce4797dd'}
message='EXACT_CASE_GO '+' '.join(subject.values())
owner={'turns':[{'role':'user','text':'[From Cuttle session "peer" · '+g.REVIEWER+']\n'+message}]}
peer={'turns':[{'role':'assistant','kind':'tool','tool':{'name':'cuttle sessions send '+g.OWNER+' '+message,'id':'t1'}},{'role':'user','kind':'tool_result','tool':{'id':'t1','ok':True}}]}
def stub(args,**kwargs):return json.dumps(owner if args[-1]==g.OWNER else peer).encode()
with patch.object(g.subprocess,'check_output',side_effect=stub): review=g.current_review(None)
assert review['subject']==subject
bad={'turns':[]}
def missing(args,**kwargs):return json.dumps(owner if args[-1]==g.OWNER else bad).encode()
with patch.object(g.subprocess,'check_output',side_effect=missing):
 try:g.current_review(None)
 except AssertionError:pass
 else:raise AssertionError('forged recipient accepted')
with tempfile.TemporaryDirectory() as d:
 g.ROOT=Path(d); now=datetime.datetime.now(datetime.timezone.utc); r={'action':'SELECT_ONCE','owner':g.OWNER,'source':g.SOURCE,'planSHA256':g.PLAN_SHA,'reviewCaptureSHA256':review['correlatedCaptureSHA256'],'createdUTC':now.isoformat()}; f=g.ROOT/'selection.json';f.write_text(json.dumps(r));f.chmod(0o600)
 cmd='/usr/bin/python3 -E -S -B '+str(g.ROOT/'select.py'); turns={'turns':[{'role':'assistant','kind':'tool','ts':(now-datetime.timedelta(seconds=1)).isoformat(),'tool':{'name':cmd,'id':'s1'}},{'role':'user','kind':'tool_result','tool':{'id':'s1','ok':True},'text':''}]}
 with patch.object(g.subprocess,'check_output',return_value=json.dumps(turns).encode()):assert g.selected(review)['sha256']==g.sha(f.read_bytes())
 with patch.object(g.subprocess,'check_output',return_value=b'{"turns":[]}'):
  try:g.selected(review)
  except AssertionError:pass
  else:raise AssertionError('unobserved selection accepted')
print('synthetic two-sided review and successful selection call accepted; forged/missing sender refused')
