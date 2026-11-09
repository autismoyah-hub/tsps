# REVIEW DRAFT: this exact source is to be passed inline to /usr/bin/python3 -c,
# not run from this path. Dedicated foreground, no threads or competing reapers.
import base64,hashlib,importlib.util,json,os,pathlib,sys
ROOT=pathlib.Path('/tmp/zogre-hf-case')
HELPER=pathlib.Path('/tmp/zogre-control/hf-author-control/004/files/authenticated-child-snapshot-command.py')
HELPER_SHA='00ff45d75be1eae3bc8f04995cfb8bcbe2bc5adc28c1162b843cfe88c87024f9'
sha=lambda b:hashlib.sha256(b).hexdigest()
if sys.version_info[:3]!=(3,12,3) or len(sys.argv)!=2 or sys.argv[1] not in ('--submit-once','--collect-once'):raise RuntimeError('EXACT_ACTION_REQUIRED')
selection=json.loads((ROOT/'selection.json').read_bytes());submit=sys.argv[1]=='--submit-once'
launcher=ROOT/('launch-case-v8.py' if submit else 'launch-collect-v4.py');expected=selection['launcherSHA256' if submit else 'collectorLauncherSHA256']
raw=launcher.read_bytes();helper=HELPER.read_bytes()
if sha(raw)!=expected or sha(helper)!=HELPER_SHA:raise RuntimeError('SNAPSHOT_MISMATCH')
mod=importlib.util.module_from_spec(importlib.util.spec_from_loader('accepted_snapshot_helper',loader=None));exec(compile(helper,str(HELPER),'exec'),mod.__dict__)
cmd=[sys.executable,'-c',mod.CHILD_BOOTSTRAP,base64.b64encode(raw).decode('ascii'),expected,str(launcher),'',sys.argv[1]]
if any('\0' in a or len(a.encode())+1>=131072 for a in cmd):raise RuntimeError('ARGV_BOUND')
os.execv(sys.executable,cmd)
