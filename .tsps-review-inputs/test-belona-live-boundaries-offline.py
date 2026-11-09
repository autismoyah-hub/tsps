"""Synthetic no-provider controls for held live boundary draft."""
import importlib.util
import os
from pathlib import Path
from unittest.mock import Mock
root=Path(__file__).resolve().parent

def load(name,file):
 spec=importlib.util.spec_from_file_location(name,root/file);mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod);return mod
m=load('live','belona-live-boundaries.py');caller=load('caller','belona-caller.py')
def rejected(fn,exception=Exception):
 try: fn()
 except exception: return
 raise AssertionError('expected refusal')
api=Mock();rejected(lambda:m.guarded_submit(api,{'subject':{}}),RuntimeError);rejected(lambda:m.guarded_status_collect(api,{'subject':{}}),RuntimeError)
api.assert_not_called();api.list_jobs.assert_not_called();api.inspect_job.assert_not_called();api.fetch_job_logs.assert_not_called()
print('hard hold blocks all live boundaries before provider methods')
source='247ed387d6a631a00440904a00aa8e1a2e44a962';name='belona-before-held'
for records in ([{'id':'1','labels':{'source':source,'name':'other'}}], [{'id':'1','labels':{'source':'other','name':name}}], [{'id':'1','labels':{'source':'other'}}]):
 fake=Mock();fake.list_jobs.return_value=iter(records)
 rejected(lambda:m.scan_provider(fake,source,name,caller),ValueError)
print('synthetic source OR name duplicate and incomplete provider row refused')
def partial():
 yield {'id':'1','labels':{'source':'other','name':'other'}}
 raise RuntimeError('incomplete iterator')
fake=Mock();fake.list_jobs.return_value=partial();rejected(lambda:m.scan_provider(fake,source,name,caller),RuntimeError)
print('synthetic provider iterator failure cannot become a clear scan')
print('no network, provider, selection, submission or game execution')

import hashlib
import tempfile
with tempfile.TemporaryDirectory() as temp:
    file=Path(temp)/'live.py'; file.write_bytes((root/'belona-live-boundaries.py').read_bytes())
    digest=hashlib.sha256(file.read_bytes()).hexdigest()
    loaded=m.module_snapshot(file,digest,'verified_live')
    loaded.verify_live_snapshot({'liveSHA256':digest})
    rejected(lambda:loaded.verify_live_snapshot({}),ValueError)
    loaded.__dict__['__BELONA_SNAPSHOT_SHA__']='0'*64
    rejected(lambda:loaded.verify_live_snapshot({'liveSHA256':digest}),ValueError)
    loaded.__dict__['__BELONA_SNAPSHOT_SHA__']=digest
    file.write_bytes(file.read_bytes()+b'\n# mutated\n')
    rejected(lambda:loaded.verify_live_snapshot({'liveSHA256':digest}),ValueError)
    rejected(lambda:m.module_snapshot(file,digest,'changed_live'),ValueError)
print('verified snapshot accepted; missing subject, wrong provenance and mutable source refused')

# module_snapshot compiles the checked source bytes, not a pre-existing module
# that happens to claim the same __file__.
with tempfile.TemporaryDirectory() as temp:
    file=Path(temp)/'module.py'; file.write_text('VALUE = "reviewed"\n')
    digest=hashlib.sha256(file.read_bytes()).hexdigest()
    old=m.module_snapshot(file,digest,'old_module')
    old.VALUE='stale-or-modified'
    fresh=m.module_snapshot(file,digest,'fresh_module')
    assert fresh.VALUE=='reviewed' and fresh is not old
    file.write_text('VALUE = "changed"\n')
    rejected(lambda:m.module_snapshot(file,digest,'changed_module'),ValueError)
print('fresh module comes from pinned bytes; altered in-memory module not reused')
for name, file in [('caller',root/'belona-caller.py'),('prestart',root/'belona-prestart.py'),
                   ('helper',Path(os.environ['BELONA_CHILD_HELPER']))]:
    raw=file.read_bytes();digest=hashlib.sha256(raw).hexdigest()
    stale=m.module_snapshot(file,digest,'stale_'+name);stale.injected_unreviewed_marker=True
    fresh=m.module_snapshot(file,digest,'fresh_'+name)
    assert fresh is not stale and not hasattr(fresh,'injected_unreviewed_marker')
    assert fresh.__BELONA_SNAPSHOT_SHA__ == digest
print('caller, prestart and helper are independently reloaded from verified snapshots')
# Synthetic original-intent and sole-collector lineage; no provider is called.
with tempfile.TemporaryDirectory() as temp:
    root2=Path(temp); plan={'source':'source1','name':'test'}
    subject={'selectedPlanSHA256':'a'*64,'owner':m.OWNER}
    intent={'intentId':'exclusive-'+'a'*64,'owner':m.OWNER,'plan':plan,'requestSHA256':'b'*64,'programSHA256':'c'*64,
            'reviewSubject':subject,'state':'PENDING_NO_RETRY','prestart':{'stage':'test'},'providerScan':[]}
    m.validate_original_intent(intent,plan,'b'*64,'c'*64,subject)
    for key in ('owner','requestSHA256','programSHA256','reviewSubject','state'):
        bad=dict(intent);bad[key]='changed';rejected(lambda:m.validate_original_intent(bad,plan,'b'*64,'c'*64,subject),ValueError)
    marker=root2/'collector.json'
    first=m.validate_collector_marker(marker,caller,plan,'b'*64,'c'*64,'job1',subject)
    assert m.validate_collector_marker(marker,caller,plan,'b'*64,'c'*64,'job1',subject)==first
    rejected(lambda:m.validate_collector_marker(marker,caller,plan,'b'*64,'c'*64,'job2',subject),ValueError)
print('synthetic original-intent mutation and different-job collector custody refused')
# Exercise the orchestration order with inert fakes, without opening provider APIs.
import json
from types import SimpleNamespace
from unittest.mock import patch
with tempfile.TemporaryDirectory() as temp:
    d=Path(temp); (d/'plan').write_bytes(b'{}'); (d/'payload').write_bytes(b'p');(d/'choice').write_text('{"choice":{}}');(d/'prestart.py').write_text('')
    subject={'callerSHA256':'f'*64,'childSHA256':'1'*64,'prestartSHA256':hashlib.sha256(b'').hexdigest(),'selectedPlanSHA256':'a'*64,'adapterSHA256':'d'*64,'pilotSHA256':'e'*64}
    ctx={'caller':'fake-caller','helper':'fake-helper','subject':subject,'plan':str(d/'plan'),'payload':str(d/'payload'),'choice':str(d/'choice'),'caseDir':str(d),'localRoot':str(d),'prestartSource':str(d/'prestart.py'),'prestartArgs':{'subject':subject,'review_raw':b'r','cli':'c','cli_sha':'s'}}
    fakecaller=Mock();fakecaller.reject_duplicates=lambda *args:None
    fakehelper=Mock();fakepre=SimpleNamespace(__file__=str(d/'prestart.py'),__BELONA_SNAPSHOT_SHA__=subject['prestartSHA256'],nonsecret_prestart=lambda **kw:{'stage':'ok'},validate_choice_live=lambda *args:{})
    fakeapi=Mock();fakeapi.whoami.return_value={'name':'Autismo-Yah'};fakeapi.list_jobs.return_value=iter([])
    plan={'source':'source1','name':'name1'};request={'labels':{}};checks=[]
    def modules(path,digest,name):
        if name=='belona_verified_caller':return fakecaller
        if name=='belona_verified_helper':return fakehelper
        return fakepre
    def recheck(*args):
        checks.append('check')
        if len(checks)==2:raise ValueError('changed before intent')
        return SimpleNamespace(),{}
    with patch.object(m,'unlocked'),patch.object(m,'module_snapshot',side_effect=modules),patch.object(m,'require_bindings'),patch.object(m,'literal_and_choice',return_value=(plan,request,'p','r')),patch.object(m,'recheck_launcher',side_effect=recheck),patch.object(m.subprocess,'run') as process:
        rejected(lambda:m.guarded_submit(fakeapi,ctx),ValueError)
        fakecaller.claim_intent.assert_not_called();process.assert_not_called()
print('synthetic changed launcher recheck after scan prevents intent and pilot call')
# Exact saved prestart/scan and receipt bytes must remain the ones captured on submit.
subject={'selectedPlanSHA256':'a'*64,'owner':m.OWNER}; plan={'source':'source1','name':'test'}
intent={'intentId':'exclusive-'+'a'*64,'owner':m.OWNER,'plan':plan,'requestSHA256':'b'*64,'programSHA256':'c'*64,
        'reviewSubject':subject,'state':'PENDING_NO_RETRY','prestart':{'stage':'original'},'providerScan':[]}
encode=lambda obj:(json.dumps(obj,sort_keys=True,separators=(',',':'))+'\n').encode()
raw=encode(intent);receipt=b'{"original":true}\n'
binding={'stage':'SUBMITTED_SAME_OWNER_BINDING','owner':m.OWNER,'jobId':'job1','source':'source1','selectedPlanSHA256':'a'*64,
         'requestSHA256':'b'*64,'programSHA256':'c'*64,'intentSHA256':m.sha(raw),'submissionSHA256':m.sha(receipt),
         'prestartSHA256':m.canonical_sha(intent['prestart']),'providerScanSHA256':m.canonical_sha(intent['providerScan'])}
m.validate_submission_binding(binding,raw,receipt,plan,'b'*64,'c'*64,'job1',subject)
for key,value in [('prestart',{'stage':'foreign'}),('providerScan',[{'id':'foreign','source':'x','name':'y'}])]:
    changed=dict(intent);changed[key]=value
    rejected(lambda:m.validate_submission_binding(binding,encode(changed),receipt,plan,'b'*64,'c'*64,'job1',subject),ValueError)
rejected(lambda:m.validate_submission_binding(binding,raw,receipt+b' ',plan,'b'*64,'c'*64,'job1',subject),ValueError)
print('synthetic changed original prestart/scan/receipt bytes rejected by submit binding')

with tempfile.TemporaryDirectory() as temp:
    file=Path(temp)/'receipt';original=b'{"original":true}\n';file.write_bytes(original)
    m.require_unchanged(file,original,'receipt')
    file.write_bytes(b'{"mutated":true}\n')
    rejected(lambda:m.require_unchanged(file,original,'receipt'),ValueError)
print('synthetic post-log receipt mutation rejected before terminal binding')
