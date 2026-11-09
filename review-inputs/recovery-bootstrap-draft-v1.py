"""Unselected bootstrap; execute only as a verified literal snapshot."""
import base64, hashlib, importlib.util, json, os, pathlib, sys
ROOT = pathlib.Path('/tmp/zogre-hf-case')
HELPER = ROOT/'authenticated-child-v2.py'
HELPER_SHA = '4663c9f62db6556261be389ab9830db755c711e1f7182f69f187df2635d97853'
sha = lambda b: hashlib.sha256(b).hexdigest()
if sys.version_info[:3] != (3,12,3) or sys.argv[1:] != ['--collect-recovery-once']:
    raise RuntimeError('ACTION_MISMATCH')
selection = json.loads((ROOT/'recovery-selection.json').read_bytes())
bootstrap_sha = globals().get('__verified_bootstrap_sha256__')
if not bootstrap_sha or bootstrap_sha != selection.get('bootstrapSHA256'):
    raise RuntimeError('BOOTSTRAP_SNAPSHOT_MISMATCH')
launcher = ROOT/'recovery-launcher-draft-v1.py'
raw = launcher.read_bytes(); helper = HELPER.read_bytes()
if sha(raw) != selection.get('launcherSHA256') or sha(helper) != HELPER_SHA:
    raise RuntimeError('SNAPSHOT_MISMATCH')
mod = importlib.util.module_from_spec(importlib.util.spec_from_loader('helper',loader=None))
exec(compile(helper,str(HELPER),'exec'),mod.__dict__)
cmd = [sys.executable,'-c',mod.CHILD_BOOTSTRAP,base64.b64encode(raw).decode('ascii'),sha(raw),str(launcher),'','--collect-recovery-once']
if any('\0' in a or len(a.encode())+1 >= 131072 for a in cmd): raise RuntimeError('ARGV_BOUND')
os.environ['TSPS_RECOVERY_BOOTSTRAP_SHA256'] = bootstrap_sha
os.execv(sys.executable,cmd)
