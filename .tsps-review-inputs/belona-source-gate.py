"""Source/profile guard; no credentials, provider API, selection or game execution."""
import hashlib
import json
import re
import subprocess


def sha(raw): return hashlib.sha256(raw).hexdigest()

def git_source(source, path):
    return subprocess.check_output(['git','show',source+':'+path],stderr=subprocess.PIPE)

def verify_source_profile(plan, manifest_raw, payload_raw, profile, read_source=git_source):
    ref=plan.get('expectedInputsRef',{})
    if len(manifest_raw) != ref.get('bytes') or sha(manifest_raw) != ref.get('sha256'): raise ValueError('manifest identity mismatch')
    manifest=json.loads(manifest_raw)
    if manifest.get('source') != plan.get('source'): raise ValueError('manifest source mismatch')
    if profile.get('image') != plan.get('image') or profile.get('cacheTarget') != plan.get('cacheTarget') or profile.get('cacheId') != plan.get('cacheId'):
        raise ValueError('profile image/cache identity mismatch')
    if not isinstance(plan.get('cacheId'),int) or plan['cacheId'] < 1: raise ValueError('invalid cache id')
    if sha(payload_raw) != profile.get('payloadSHA256'): raise ValueError('payload identity mismatch')
    expected=manifest.get('expectedInputs',{})
    def source(path):
        data=read_source(plan['source'],path)
        if not isinstance(data,bytes) or sha(data) != expected.get(path): raise ValueError('source bytes mismatch: '+path)
        return data
    target=source('server/target.txt').decode('utf-8').strip()
    if not re.fullmatch(r'osrs-\d+_\d{4}-\d{2}-\d{2}',target) or target != plan['cacheTarget']:
        raise ValueError('source target/profile mismatch')
    cache=source('server/scripts/ensure-cache.ts').decode('utf-8')
    for marker in ('const TARGET_FILE = path.join(SERVER_ROOT, "target.txt")',
                   'const target = readTarget()', 'parseTargetName(target)',
                   'c.builds[0].major === revision', 'c.timestamp?.startsWith(date)',
                   'findCacheOnOpenRS2(revision, date)'):
        if marker not in cache: raise ValueError('ensure-cache source contract changed')
    package=json.loads(source('server/package.json'))
    version=package.get('dependencies',{}).get('node-datachannel')
    engine=package.get('engines',{}).get('node')
    if not isinstance(version,str) or not re.fullmatch(r'\d+\.\d+\.\d+',version) or version != profile.get('nativeVersion'):
        raise ValueError('native dependency/profile mismatch')
    if not isinstance(engine,str) or engine != profile.get('nodeEngine'): raise ValueError('node engine/profile mismatch')
    lock=source('server/yarn.lock').decode('utf-8')
    if f'"node-datachannel@npm:{version}":' not in lock or f'  version: {version}' not in lock:
        raise ValueError('native lock mismatch')
    payload=payload_raw.decode('utf-8')
    asset=f'node-datachannel-v{version}-napi-v8-linux-x64.tar.gz'
    native_hash=profile.get('nativeAssetSHA256')
    if not isinstance(native_hash,str) or not re.fullmatch(r'[0-9a-f]{64}',native_hash) or asset not in payload or native_hash not in payload:
        raise ValueError('native payload/profile mismatch')
    for marker in ("scripts/ensure-cache.ts", "target !== PLAN.cacheTarget", "cacheInfo.id !== PLAN.cacheId"):
        if marker not in payload: raise ValueError('payload cache guard missing')
    expected_cache=plan.get('expectedCacheFiles')
    if not isinstance(expected_cache,dict) or not expected_cache: raise ValueError('cache file evidence missing')
    prefix='caches/'+target+'/'
    if 'caches/caches.json' not in expected_cache or any(k!='caches/caches.json' and not k.startswith(prefix) for k in expected_cache):
        raise ValueError('cache file target mismatch')
    if not all(prefix+name in expected_cache for name in ('info.json','keys.json','main_file_cache.dat2','main_file_cache.idx255')):
        raise ValueError('cache file evidence incomplete')
    return {'source':plan['source'],'target':target,'cacheId':plan['cacheId'],'nodeEngine':engine,'nativeVersion':version,'image':plan['image']}

def guard_before_adapter(held_raw,manifest_raw,payload_raw,profile,caller,adapter,read_source=git_source):
    """Only a synthetic adapter may be supplied before independent case review."""
    _,selected=caller.prepare_selected(held_raw)
    plan=json.loads(selected)
    result=verify_source_profile(plan,manifest_raw,payload_raw,profile,read_source)
    adapter()
    return result
