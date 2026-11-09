"""HELD source candidate. Run only after exact-case review and selection."""
import hashlib, importlib.metadata, json, os, re, sys, time, types, urllib.request
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
SDK = Path('/workspace/shades-evidence/sdk/site-packages')
ACCESS = ROOT / 'hf-project-access-v4.py'
ENTRY = HERE / 'case-executor.py'
PYTHON_SHA = 'c2c20b4745d447551221ec3d4e70f92c270c4609fe3df34fc52ea6dd46e92273'
ACCESS_SHA = 'fab4bc92dc8601143ce61e910f8121774c22a0d19be6d2617bba9449f7131326'
ENTRY_SHA = 'decdc2d3257b8fc11eee9ec9989839c5698cc081f52bd583d683043efcd34f51'
RECEIPT_SHA = '7ef21896e53f1e39c16f83b17c07889d4d570ff3415814cdb6aa1ce2901ae39f'
PROJECTION_SHA = '9cd75e87859ecfa4f0f7ef0fee291e1aaa76fbf4e67a5f68663d56ffc2a92bfd'
def checked(path, digest):
    raw = path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != digest: raise RuntimeError('Bound input mismatch')
    return raw
bundle = json.loads(checked(HERE / 'bindings.json', 'c8ff9392a62d379d03ff37437e299ad6662e396abdd00919a98869ebd06a0a09'))
if set(bundle) != {'payload','resolver','pilot','package','caller','originalPlan','originalRequest','profileBasis','originalD46','submitCaller','inputCollector'}: raise RuntimeError('Binding keys mismatch')
for row in bundle.values():
    if set(row) != {'path','sha256'}: raise RuntimeError('Binding schema mismatch')
    path=(ROOT / row['path']).resolve()
    if not path.is_relative_to(ROOT.resolve()): raise RuntimeError('Binding path escape')
    checked(path,row['sha256'])
if Path(sys.executable).resolve() != Path('/usr/bin/python3.12').resolve() or sys.version_info[:2] != (3, 12): raise RuntimeError('Interpreter mismatch')
checked(Path(sys.executable).resolve(), PYTHON_SHA)
if not sys.flags.isolated or not sys.dont_write_bytecode or sys.flags.optimize: raise RuntimeError('Require isolated Python -I -B without optimization')
checked(Path('/workspace/shades-evidence/sdk/setup-receipt.json'), RECEIPT_SHA)
projection = json.loads(checked(ROOT / 'sdk-projection.json', PROJECTION_SHA))
actual = sorted([{'name': re.sub(r'[-_.]+', '-', d.metadata['Name']).lower(), 'version': d.version} for d in importlib.metadata.distributions(path=[str(SDK)])], key=lambda x:(x['name'],x['version']))
if actual != projection or len(actual) != len({x['name'] for x in actual}): raise RuntimeError('SDK distribution projection mismatch')
if any(x == '--executor-source-sha256' or x.startswith('--executor-source-sha256=') for x in sys.argv[1:]): raise RuntimeError('Executor hash option is launcher-owned')
checked(ROOT / 'review-authority.py', 'f4388755f7a792c27ff0cbadba4ab937af5db12b9735bc3badf5c230a644e946')
access = checked(ACCESS, ACCESS_SHA)
checked(ENTRY, ENTRY_SHA)
sys.path.insert(0, str(SDK))
import huggingface_hub
if Path(huggingface_hub.__file__).resolve() != (SDK / 'huggingface_hub' / '__init__.py').resolve() or importlib.metadata.version('huggingface_hub') != '2.4.0': raise RuntimeError('SDK module origin or version mismatch')
args = sys.argv[1:]
def preauth_plan_check(args):
    def arg(name):
        if args.count(name) != 1: raise RuntimeError('Missing or duplicate selected plan argument')
        i = args.index(name)
        if i + 1 >= len(args): raise RuntimeError('Missing selected plan value')
        return args[i + 1]
    selected_path = Path(arg('--selected-plan')).resolve()
    if selected_path != (HERE / 'selected-plan.json').resolve(): raise RuntimeError('Selected plan path mismatch')
    selected = json.loads(checked(selected_path, arg('--selected-plan-sha256')))
    preview = json.loads(checked(HERE / 'preview.json', '3b816c8f8b702db85ca05f7632add4c8d10653d988cabc41c34c0874401bcdce'))
    expected = dict(preview['plan'], executionHold=False, selection=selected.get('selection'))
    if selected != expected: raise RuntimeError('Selected plan differs from reviewed preview; HOLD')
preauth_plan_check(args)
if os.environ.get('CCC_SESSION_ID') != '260b5681-1632-4418-adb3-1e0ce7a20952': raise RuntimeError('Invoking session mismatch')
if args and args[0] == 'submit':
    with (HERE / 'prestart-clock.json').open('xb') as f:
        f.write(json.dumps({'startedMonotonic':time.monotonic()}).encode()); f.flush(); os.fsync(f.fileno())
if args and args[0] == 'collect':
    record = json.loads((HERE / 'submission.json').read_bytes())
    job = record.get('job', {}).get('id')
    if not isinstance(job, str) or not job or record.get('stage') != 'SUBMITTED' or record.get('namespace') != 'Autismo-Yah': raise RuntimeError('Missing collection identity')
    claim_dir = HERE / 'collection-journal'
    claim_dir.mkdir(mode=0o700)  # Exclusive claim before auth004 can query whoami.
    parent_fd=os.open(HERE,os.O_RDONLY|os.O_DIRECTORY)
    try: os.fsync(parent_fd)
    finally: os.close(parent_fd)
    with (claim_dir / '00-claim.json').open('xb') as f:
        f.write(json.dumps({'collector':'260b5681-1632-4418-adb3-1e0ce7a20952','jobId':job,'planSHA256':args[args.index('--selected-plan-sha256')+1]},sort_keys=True).encode()); f.flush(); os.fsync(f.fileno())
    with (claim_dir / '01-auth-start.json').open('xb') as f:
        f.write(b'{"query":"auth-whoami","stage":"START"}'); f.flush(); os.fsync(f.fileno())
    claim_fd=os.open(claim_dir,os.O_RDONLY|os.O_DIRECTORY)
    try: os.fsync(claim_fd)
    finally: os.close(claim_fd)
# Cuttle ACP is not the provider. Use this session's ambient credentials only; no redirects/fallback.
class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self,*args,**kwargs): raise RuntimeError('ACP redirect refused')
token=os.environ.get('CCC_TOKEN') or os.environ.get('CCC_PAT')
if not token and os.environ.get('CCC_TOKEN_FILE'): token=Path(os.environ['CCC_TOKEN_FILE']).read_text().strip()
if not token: raise RuntimeError('ACP ambient auth missing')
headers={'Authorization':'Bearer '+token} if token.startswith('ccc_pat_') else {'X-CCC-Control-Token':token}
url=os.environ['CCC_BASE_URL'].rstrip('/')+'/api/acp/sessions/260b5681-1632-4418-adb3-1e0ce7a20952/config'
with urllib.request.build_opener(NoRedirect()).open(urllib.request.Request(url,headers=headers),timeout=15) as response:
    raw=response.read(1024*1024+1)
    if response.status!=200 or len(raw)>1024*1024: raise RuntimeError('ACP config status/size')
config=json.loads(raw); options=config.get('configOptions')
if not isinstance(options,list): raise RuntimeError('ACP options missing')
values={}
for option in options:
    key=option.get('id')
    if key in values: raise RuntimeError('ACP duplicate option')
    values[key]=option.get('currentValue')
if values.get('model')!='gpt-6.1-terra' or values.get('reasoning_effort')!='medium' or values.get('mode')!='agent-full-access' or values.get('collaboration_mode')!='default' or config.get('modes',{}).get('currentModeId')!='agent-full-access': raise RuntimeError('ACP effective configuration mismatch')
config_receipt={'session':'260b5681-1632-4418-adb3-1e0ce7a20952','model':values['model'],'effort':values['reasoning_effort'],'mode':values['mode'],'collaboration':values['collaboration_mode'],'rawSHA256':hashlib.sha256(raw).hexdigest()}
config_file=(HERE/'collection-journal'/'00b-acp-config.json') if args[0]=='collect' else (HERE/'prestart-config.json')
with config_file.open('xb') as f:
    f.write(json.dumps(config_receipt,sort_keys=True).encode());f.flush();os.fsync(f.fileno())
config_dir=os.open(config_file.parent,os.O_RDONLY|os.O_DIRECTORY)
try: os.fsync(config_dir)
finally: os.close(config_dir)
del token,headers,raw,config
sys.argv = [str(ACCESS), '--entry', str(ENTRY), '--entry-sha256', ENTRY_SHA, '--'] + args + ['--executor-source-sha256', ENTRY_SHA]
module = types.ModuleType('__main__')
module.__dict__.update(__file__=str(ACCESS), __name__='__main__', __package__='', __spec__=None, __cached__=None)
sys.modules['__main__'] = module
exec(compile(access, str(ACCESS), 'exec'), module.__dict__)
