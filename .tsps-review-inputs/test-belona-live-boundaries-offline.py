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
api=Mock();rejected(lambda:m.guarded_submit(api,None,None,None,{}),RuntimeError);rejected(lambda:m.guarded_status_collect(api,None,None,{}),RuntimeError)
api.assert_not_called();api.list_jobs.assert_not_called();api.inspect_job.assert_not_called();api.fetch_job_logs.assert_not_called()
print('hard hold blocks all live boundaries before provider methods')
source='247ed387d6a631a00440904a00aa8e1a2e44a962';name='belona-before-held'
for records in ([{'id':'1','name':'other','labels':{'source':source}}], [{'id':'1','name':name,'labels':{'source':'other'}}], [{'id':'1','name':'other'}]):
 fake=Mock();fake.list_jobs.return_value=iter(records)
 rejected(lambda:m.scan_provider(fake,source,name,caller),ValueError)
print('synthetic source OR name duplicate and incomplete provider row refused')
def partial():
 yield {'id':'1','name':'other','labels':{'source':'other'}}
 raise RuntimeError('incomplete iterator')
fake=Mock();fake.list_jobs.return_value=partial();rejected(lambda:m.scan_provider(fake,source,name,caller),RuntimeError)
print('synthetic provider iterator failure cannot become a clear scan')
print('no network, provider, selection, submission or game execution')
