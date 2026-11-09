"""Held, nonsecret source/review prestart primitives. No selection or provider access."""
import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import subprocess
import sys
import urllib.request
import urllib.error
from datetime import datetime

OWNER = '49676823-be5e-408f-be58-047e80dadbb2'
REVIEWER = 'ed3a4125-645c-4266-8b86-4e199dd5bc63'
PROJECT = '4cbb8d38-9dae-47c7-9736-c0020aea1c2a'
PREFLIGHT_SHA = 'de911881dfce95a7e95a3f13c2aa8fb0140010b6e696e08b8cfc9f9fcfb8fead'
CALLER_SHA = 'c649a7b0eb3bdaceff8c81a380b9fa1b3fe9ed5bc14e977918f7cb234a870c0c'

def sha(raw): return hashlib.sha256(raw).hexdigest()

def pinned(path, digest):
    raw = Path(path).read_bytes()
    if sha(raw) != digest: raise ValueError('source hash mismatch')
    return raw

def durable_create(path, raw):
    path = Path(path)
    with path.open('xb') as stream:
        stream.write(raw); stream.flush(); os.fsync(stream.fileno())
    fd = os.open(path.parent, os.O_RDONLY | getattr(os, 'O_DIRECTORY', 0))
    try: os.fsync(fd)
    finally: os.close(fd)
    if path.read_bytes() != raw: raise ValueError('capture readback mismatch')

def canonical_subject(subject):
    required = {'owner','reviewer','model','reasoningEffort','source','heldPlanSHA256','selectedPlanSHA256','manifestSHA256',
                'callerSHA256','prestartSHA256','payloadSHA256','pilotSHA256','adapterSHA256',
                'childSHA256','classifierSHA256','liveSHA256','sdkPythonSHA256','sdkManifestSHA256','cliSHA256','sdkVersion','requestSHA256'}
    if not isinstance(subject,dict) or set(subject) != required: raise ValueError('authority subject schema')
    if subject['owner'] != OWNER or subject['reviewer'] != REVIEWER or subject['model'] != 'gpt-6.1-terra' or subject['reasoningEffort'] != 'medium' or subject['sdkVersion'] != '2.4.0':
        raise ValueError('authority identity mismatch')
    for name in required - {'owner','reviewer','model','reasoningEffort','source','sdkVersion'}:
        value=subject[name]
        if not isinstance(value,str) or len(value)!=64 or any(c not in '0123456789abcdef' for c in value):
            raise ValueError('authority hash malformed')
    allowed = {
        '247ed387d6a631a00440904a00aa8e1a2e44a962': ('e50f6ec1e341799fd48bd3e8d39897717f68d8b657afa9297e3b7a968118fabf','6d739505fa4ca4aa04fb3e273b47b40814256edd4af4b14d176e3d15c30a2b16','a44a97f57d259f84a8cc2ad3adc6ce9dbd65ec8013e7def9258702c78a2aa305'),
        '88b5e5793039c1f246c2a4096206664e50c51003': ('7b141df4d1b9966654795c6fcff01efc11c0c153bbf7806519df1ca60560157e','92dcba657ee0599f5e38c7927e1bcb1296301125f435f002f828bcc109b1ec00','f2797ca4c62cd14e4c039f766d17163d49f4a28ed1548e3af87616aafa4e7206'),
    }
    if allowed.get(subject['source']) != (subject['heldPlanSHA256'],subject['selectedPlanSHA256'],subject['manifestSHA256']):
        raise ValueError('source/plan/manifest mismatch')
    return json.dumps(subject,sort_keys=True,separators=(',',':')).encode()

def capture_review(subject, cli, cli_sha, output):
    """Read actual named peer's session, retain only exact GO turn and transcript hash."""
    subject_raw = canonical_subject(subject)
    digest = sha(subject_raw)
    if subject['cliSHA256'] != cli_sha: raise ValueError('CLI subject mismatch')
    pinned(cli, cli_sha)
    command = [str(Path(cli).resolve()), '--json', 'sessions', 'read', '--turns', '100', REVIEWER]
    result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30, check=True)
    transcript = json.loads(result.stdout)
    matches = [turn for turn in transcript.get('turns',[]) if turn.get('role') == 'assistant'
               and turn.get('kind') != 'thinking' and turn.get('text','').strip() == 'BELONA_PROFILE_GO '+digest]
    if len(matches) != 1: raise ValueError('direct exact peer review not found or ambiguous')
    turn = matches[0]
    if not isinstance(turn.get('ts'), str) or not turn['ts']: raise ValueError('review timestamp missing')
    receipt = {'type':'DIRECT_SESSION_CAPTURE','reviewer':REVIEWER,'owner':OWNER,'subject':subject,
               'subjectSHA256':digest,'turn':turn,'rawTranscriptSHA256':sha(result.stdout),'cliSHA256':cli_sha}
    raw = (json.dumps(receipt,sort_keys=True,indent=2)+'\n').encode()
    durable_create(output,raw)
    return sha(raw)

def validate_review(receipt_raw, subject, cli_sha):
    receipt = json.loads(receipt_raw)
    digest = sha(canonical_subject(subject))
    if receipt.get('type') != 'DIRECT_SESSION_CAPTURE' or receipt.get('reviewer') != REVIEWER or receipt.get('owner') != OWNER:
        raise ValueError('review identity mismatch')
    if receipt.get('subject') != subject or receipt.get('subjectSHA256') != digest or receipt.get('cliSHA256') != cli_sha:
        raise ValueError('review subject mismatch')
    turn = receipt.get('turn',{})
    if turn.get('role') != 'assistant' or turn.get('kind') == 'thinking' or turn.get('text','').strip() != 'BELONA_PROFILE_GO '+digest or not turn.get('ts'):
        raise ValueError('review turn mismatch')
    if not isinstance(receipt.get('rawTranscriptSHA256'),str) or len(receipt['rawTranscriptSHA256']) != 64:
        raise ValueError('review transcript hash missing')


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self,*args,**kwargs): raise ValueError('config redirect refused')

def effective_config():
    base=os.environ.get('CCC_BASE_URL','').rstrip('/')
    token=os.environ.get('CCC_TOKEN') or os.environ.get('CCC_PAT')
    if not base or not token: raise ValueError('own ambient config credentials unavailable')
    headers={'Authorization':'Bearer '+token} if token.startswith('ccc_pat_') else {'X-CCC-Control-Token':token}
    request=urllib.request.Request(base+'/api/acp/sessions/'+OWNER+'/config',headers=headers)
    with urllib.request.build_opener(NoRedirect()).open(request,timeout=30) as response:
        if response.status != 200: raise ValueError('effective config unavailable')
        raw=response.read(1024*1024+1)
    if len(raw)>1024*1024: raise ValueError('config response exceeds bound')
    data=json.loads(raw)
    if data.get('ok') is not True: raise ValueError('effective config rejected')
    options=data.get('configOptions')
    if not isinstance(options,list): raise ValueError('config options missing')
    values={}
    for item in options:
        if not isinstance(item,dict): raise ValueError('invalid config option')
        key=item.get('id')
        if key in ('model','reasoning_effort'):
            if key in values: raise ValueError('duplicate effective config')
            values[key]=item.get('currentValue')
    if values != {'model':'gpt-6.1-terra','reasoning_effort':'medium'}:
        raise ValueError('effective model/effort mismatch')
    return {'model':values['model'],'reasoningEffort':values['reasoning_effort'],'configSHA256':sha(raw)}

def selected_choice(subject, selected_raw):
    digest=sha(canonical_subject(subject))
    if sha(selected_raw)!=subject['selectedPlanSHA256']: raise ValueError('selected plan mismatch')
    selected=json.loads(selected_raw)
    if selected.get('source')!=subject['source'] or selected.get('expectedInputsRef',{}).get('sha256')!=subject['manifestSHA256']:
        raise ValueError('choice source/manifest mismatch')
    return {'action':'SELECT_ONCE','owner':OWNER,'source':selected['source'],
            'selectedPlanSHA256':sha(selected_raw),'manifestSHA256':subject['manifestSHA256'],
            'requestSHA256':subject['requestSHA256'],'subjectSHA256':digest,'attempts':1,'name':selected['name']}

def capture_choice(subject, selected_raw, review_raw, cli, cli_sha, output):
    """A distinct exact owner assistant turn is required; never creates the turn."""
    if os.environ.get('CCC_SESSION_ID') != OWNER: raise ValueError('session environment mismatch')
    validate_review(review_raw,subject,cli_sha)
    if subject['cliSHA256']!=cli_sha: raise ValueError('CLI subject mismatch')
    pinned(cli,cli_sha)
    choice=selected_choice(subject,selected_raw)
    result=subprocess.run([str(Path(cli).resolve()),'--json','sessions','read','--turns','100',OWNER],stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=30,check=True)
    turns=json.loads(result.stdout).get('turns',[])
    text='BELONA_SELECT_ONCE '+choice['subjectSHA256']+' '+choice['selectedPlanSHA256']+' '+choice['requestSHA256']
    matches=[t for t in turns if t.get('role')=='assistant' and t.get('kind')!='thinking' and t.get('text','').strip()==text]
    if len(matches)!=1: raise ValueError('exact owner choice missing or ambiguous')
    review_turn=json.loads(review_raw)['turn']; turn=matches[0]
    try:
        selected_at=datetime.fromisoformat(turn['ts'].replace('Z','+00:00'))
        reviewed_at=datetime.fromisoformat(review_turn['ts'].replace('Z','+00:00'))
    except (ValueError,KeyError,TypeError): raise ValueError('choice/review timestamps invalid')
    if selected_at<=reviewed_at: raise ValueError('choice not later than review')
    receipt={'type':'DIRECT_OWNER_CHOICE_CAPTURE','choice':choice,'turn':turn,'rawTranscriptSHA256':sha(result.stdout),'cliSHA256':cli_sha}
    raw=(json.dumps(receipt,sort_keys=True,indent=2)+'\n').encode();durable_create(output,raw);return sha(raw)

def validate_choice_live(subject, selected_raw, choice_raw, review_raw, cli, cli_sha):
    if os.environ.get('CCC_SESSION_ID') != OWNER: raise ValueError('session environment mismatch')
    validate_review(review_raw,subject,cli_sha)
    if subject['cliSHA256'] != cli_sha: raise ValueError('CLI subject mismatch')
    pinned(cli,cli_sha)
    receipt=json.loads(choice_raw);choice=selected_choice(subject,selected_raw)
    if receipt.get('type')!='DIRECT_OWNER_CHOICE_CAPTURE' or receipt.get('choice')!=choice or receipt.get('cliSHA256')!=cli_sha:
        raise ValueError('choice receipt mismatch')
    result=subprocess.run([str(Path(cli).resolve()),'--json','sessions','read','--turns','100',OWNER],stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=30,check=True)
    text='BELONA_SELECT_ONCE '+choice['subjectSHA256']+' '+choice['selectedPlanSHA256']+' '+choice['requestSHA256']
    matches=[t for t in json.loads(result.stdout).get('turns',[]) if t.get('role')=='assistant' and t.get('kind')!='thinking' and t.get('text','').strip()==text]
    if len(matches)!=1 or matches[0]!=receipt.get('turn'): raise ValueError('live owner choice mismatch')
    try:
        selected_at=datetime.fromisoformat(matches[0]['ts'].replace('Z','+00:00'))
        reviewed_at=datetime.fromisoformat(json.loads(review_raw)['turn']['ts'].replace('Z','+00:00'))
    except (ValueError,KeyError,TypeError): raise ValueError('choice/review timestamps invalid')
    if selected_at<=reviewed_at: raise ValueError('choice not later than review')
    return choice

def nonsecret_prestart(subject, review_raw, cli, cli_sha, preflight, caller, runtime_files, sdk_manifest):
    """Recheck connected identity, authenticated peer turn and local source preflight; no provider."""
    if os.environ.get('CCC_SESSION_ID') != OWNER: raise ValueError('session environment mismatch')
    if subject['cliSHA256'] != cli_sha or subject['callerSHA256'] != CALLER_SHA: raise ValueError('CLI/caller subject mismatch')
    pinned(cli,cli_sha); pinned(preflight,PREFLIGHT_SHA); pinned(caller,CALLER_SHA)
    keys = {'payloadSHA256','pilotSHA256','adapterSHA256','childSHA256','classifierSHA256','liveSHA256','sdkPythonSHA256','prestartSHA256'}
    if set(runtime_files) != keys: raise ValueError('runtime file keyset mismatch')
    for key in keys: pinned(runtime_files[key],subject[key])
    if Path(runtime_files['prestartSHA256']).resolve() != Path(__file__).resolve(): raise ValueError('prestart source mismatch')
    if Path(runtime_files['sdkPythonSHA256']).resolve() != Path(sys.executable).resolve(): raise ValueError('active Python mismatch')
    pinned(sdk_manifest,subject['sdkManifestSHA256'])
    manifest = json.loads(Path(sdk_manifest).read_bytes())
    if manifest.get('version') != '2.4.0' or not isinstance(manifest.get('files'),dict) or not manifest['files']:
        raise ValueError('SDK manifest invalid')
    sdk_root = Path(manifest['root']).resolve(strict=True)
    actual = set()
    for entry in sdk_root.rglob('*'):
        if entry.is_symlink(): raise ValueError('SDK symlink refused')
        if entry.is_file(): actual.add(entry.relative_to(sdk_root).as_posix())
    if actual != set(manifest['files']): raise ValueError('SDK file membership mismatch')
    distributions = [d for d in importlib.metadata.distributions(path=[str(sdk_root)])
                     if d.metadata.get('Name','').lower().replace('_','-') == 'huggingface-hub']
    if len(distributions) != 1 or distributions[0].version != '2.4.0': raise ValueError('SDK package version mismatch')
    for name, digest in manifest['files'].items():
        rel = Path(name)
        if rel.is_absolute() or '..' in rel.parts: raise ValueError('SDK path escapes root')
        target = sdk_root / rel
        if target.is_symlink() or not target.resolve(strict=True).is_relative_to(sdk_root): raise ValueError('SDK link/path invalid')
        pinned(target,digest)
    validate_review(review_raw,subject,cli_sha)
    config = effective_config()
    if config['model'] != subject['model'] or config['reasoningEffort'] != subject['reasoningEffort']: raise ValueError('subject/config mismatch')
    info = json.loads(subprocess.check_output([str(Path(cli).resolve()),'--json','sessions','get',OWNER],timeout=30))
    if info.get('id') != OWNER or info.get('projectId') != PROJECT or info.get('model') != 'gpt-6.1-terra' or info.get('status') != 'running':
        raise ValueError('connected owner/model/project mismatch')
    # Re-read the review from the peer rather than trusting an author-written receipt.
    result = subprocess.run([str(Path(cli).resolve()),'--json','sessions','read','--turns','100',REVIEWER],stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=30,check=True)
    turns=json.loads(result.stdout).get('turns',[])
    expected='BELONA_PROFILE_GO '+sha(canonical_subject(subject))
    matches=[turn for turn in turns if turn.get('role')=='assistant' and turn.get('kind')!='thinking' and turn.get('text','').strip()==expected]
    if len(matches)!=1 or matches[0] != json.loads(review_raw)['turn']: raise ValueError('live peer review mismatch')
    run = subprocess.run([sys.executable,str(Path(preflight).resolve())],stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=120,check=True)
    source = json.loads(run.stdout)
    if source.get('stage') != 'SOURCE_PREFLIGHT_ONLY' or source.get('selection') is not False or source.get('providerContact') is not False:
        raise ValueError('source preflight mismatch')
    return {'stage':'NONSECRET_PRESTART_ONLY','owner':OWNER,'model':info['model'],'subjectSHA256':sha(canonical_subject(subject)),
            'sourcePreflightSHA256':sha(run.stdout),'reviewReceiptSHA256':sha(review_raw),'config':config,'providerContact':False,'selection':False}
