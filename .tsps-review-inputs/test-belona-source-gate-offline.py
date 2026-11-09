"""Source-only gate controls; adapter is a fake and no selection is written."""
import copy
import importlib.util
import json
from pathlib import Path
root=Path(__file__).resolve().parent
def load(name,file):
 spec=importlib.util.spec_from_file_location(name,root/file);mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod);return mod
g=load('gate','belona-source-gate.py');caller=load('caller','belona-caller.py')
payload=(root/'belona-native-payload.cjs').read_bytes();profile=json.loads((root/'belona-source-profile.json').read_bytes());inventory=(root/'belona-cache-inventory.json').read_bytes()
def rejected(fn):
 try:fn()
 except ValueError:return
 raise AssertionError('expected refusal')
for side in ('before','after'):
 held=(root/('belona-'+side+'-held-plan.json')).read_bytes();manifest=(root/('belona-'+side+'-manifest.json')).read_bytes();calls=[]
 result=g.guard_before_adapter(held,manifest,payload,profile,inventory,caller,lambda:calls.append('fake-adapter'))
 assert calls==['fake-adapter'] and result['cacheId']==profile['cacheId']
 for field,value in [('cacheTarget','osrs-999_2099-01-01'),('cacheId',9999),('nativeVersion','9.9.9'),('nodeEngine','>=99'),('image','other')]:
  bad=dict(profile);bad[field]=value;calls=[]
  rejected(lambda:g.guard_before_adapter(held,manifest,payload,bad,inventory,caller,lambda:calls.append('adapter')))
  assert calls==[]
 # Even a mutually changed plan/profile must match the independent retained inventory.
 plan=json.loads(held); altered=copy.deepcopy(plan); altered_profile=dict(profile)
 altered['cacheId']=9999; altered_profile['cacheId']=9999
 rejected(lambda:g.verify_source_profile(altered,manifest,payload,altered_profile,inventory))
 altered=copy.deepcopy(plan); cache_key='caches/'+plan['cacheTarget']+'/info.json'
 altered['expectedCacheFiles'][cache_key]='0'*64
 rejected(lambda:g.verify_source_profile(altered,manifest,payload,profile,inventory))
 calls=[]
 def modified(source,path):
  raw=g.git_source(source,path)
  return raw+b'\nchanged' if path=='server/scripts/ensure-cache.ts' else raw
 rejected(lambda:g.guard_before_adapter(held,manifest,payload,profile,inventory,caller,lambda:calls.append('adapter'),modified))
 assert calls==[]
 rejected(lambda:g.guard_before_adapter(held,manifest+b' ',payload,profile,inventory,caller,lambda:calls.append('adapter')))
 assert calls==[]
 print(side,'matching source/profile reaches fake adapter; mismatches and changed inputs leave adapter untouched')
print('synthetic fake adapter only; no auth, provider, selected file or game')
