"""Synthetic no-provider controls for held live boundary draft."""
import importlib.util
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
api=Mock();rejected(lambda:m.guarded_submit(api,None,None,None,{'subject':{}}),RuntimeError);rejected(lambda:m.guarded_status_collect(api,None,None,{'subject':{}}),RuntimeError)
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
