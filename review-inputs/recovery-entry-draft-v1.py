"""UNSELECTED DRAFT: same-job collection only; never submit or create submission.json."""
import hashlib, importlib.metadata, importlib.util, json, os, pathlib, re, shlex, signal, stat, sys, types
from datetime import datetime, timezone
from huggingface_hub import HfApi

ROOT = pathlib.Path('/tmp/zogre-hf-case')
OWNER = '42689c7a-eddd-44ad-b8bb-04bd0f3c70de'
REVIEWER = 'd9664b7d-13e2-464d-81e5-37de82992eb2'
JOB = '6af14d46e6320c66fad7d183'
CLAIM = '15ecdacb4b53366c5d06ddd79adb1f9c480a0461'
REF = 'refs/heads/case-intents/zogre-uglug-baseline-42689c'
REMOTE = 'https://github.com/autismoyah-hub/tsps.git'
BINDING = '722073938c846568ff073098702b72c2fd083b3d55081af050decd4b9b0daab6'
RECORDS = 'b37e74ba6bde114d968dead3a0ad2b4d7f2306976e6c402af98368584826b22c'
FRAMES = 'f53689d4f2389af93ad8f6a43fe0de94caf8dcaabde3eacbd13b1c957b129a76'
ENTRY16 = '9994f0b4e12f527dc10ad012637f1612d9be7d0fb6801446dc879d5990187c47'
WRAPPER = 'fe33c0f548236836256fae770ffb4dedf42c8ac66b71694c7940b5dafcb2f755'
SDK_API = '3871f6516f2bf97b77cf4179daea929b30b4c97d7e87e9158fa742f46cbf4860'
SDK_JOBS = '4f6e280e0f1938d3230625384b25acdb360aac2925fe23040a8525c0bc1306e2'
sha = lambda b: hashlib.sha256(b).hexdigest()

def pin(path, expected):
    data = path.read_bytes()
    if sha(data) != expected: raise RuntimeError('PIN_MISMATCH')
    return data

def snapshot(path, expected, name):
    raw = pin(path, expected); module = types.ModuleType(name); module.__file__ = str(path)
    exec(compile(raw, str(path), 'exec'), module.__dict__)
    return module

def durable(path, value):
    data = (json.dumps(value, sort_keys=True, indent=2) + '\n').encode()
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, 'wb') as f:
        f.write(data); f.flush(); os.fsync(f.fileno())
    fd = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try: os.fsync(fd)
    finally: os.close(fd)

def bounded(seconds, operation):
    def timeout(_sig, _frame): raise TimeoutError('TIMEOUT')
    old = signal.signal(signal.SIGALRM, timeout); signal.alarm(seconds)
    try: return operation()
    finally: signal.alarm(0); signal.signal(signal.SIGALRM, old)

def recovery_authority(authority, recovery, recovery_sha):
    selected = authority.parse_utc(recovery['selectedUTC'])
    age = (datetime.now(timezone.utc) - selected).total_seconds()
    if age < 0 or age > 900: raise RuntimeError('RECOVERY_SELECTION_STALE')
    own = authority.read_turns(OWNER, selected)
    parent = authority.read_turns(authority.PARENT, selected)
    peer = authority.read_turns(REVIEWER, selected)
    for turns in (own, parent):
        for turn in turns:
            text = turn.get('text') or ''
            if turn.get('role') == 'user' and (authority.known_hold(text) or authority.revoke_seen(turn) or re.search(r"\b(do not|don't|must not|never)\b.{0,80}\b(collect|collection)\b", text, re.I | re.S) or 'UGLUG_RECOVERY_REVOKE' in text):
                raise RuntimeError('KNOWN_HOLD_OR_REVOKE')
    marker = ('UGLUG_RECOVERY_GO owner=' + OWNER + ' reviewer=' + REVIEWER +
              ' job=' + JOB + ' selection=' + recovery_sha)
    delivered = [t for t in own if t.get('role') == 'user' and REVIEWER in (t.get('text') or '') and (t.get('text') or '').split('\n', 1)[-1] == marker]
    matches = []
    for i, turn in enumerate(peer):
        if turn.get('role') != 'assistant' or turn.get('kind') != 'tool': continue
        tool = turn.get('tool') or {}
        try: args = shlex.split(tool.get('name') or '')
        except ValueError: continue
        if args != ['cuttle', 'sessions', 'send', OWNER, marker]: continue
        result = any(q.get('kind') == 'tool_result' and (q.get('tool') or {}).get('id') == tool.get('id') for q in peer[i+1:])
        if result and any(authority.parse_utc(q['ts']) >= authority.parse_utc(turn['ts']) for q in delivered): matches.append(turn)
    if not matches: raise RuntimeError('RECOVERY_REVIEW_ABSENT')
    review_time = authority.parse_utc(matches[-1]['ts'])
    for turn in peer:
        if authority.parse_utc(turn['ts']) > review_time and 'UGLUG_RECOVERY_REVOKE' in ((turn.get('text') or '') + ' ' + ((turn.get('tool') or {}).get('name') or '')): raise RuntimeError('RECOVERY_REVIEW_REVOKED')
    if sha((ROOT/'recovery-selection.json').read_bytes()) != recovery_sha: raise RuntimeError('RECOVERY_SELECTION_CHANGED')

def main():
    if sys.argv[1:] != ['--collect-recovery-once'] or os.environ.get('CCC_SESSION_ID') != OWNER or sys.version_info[:3] != (3, 12, 3): raise RuntimeError('OWNER_OR_ACTION')
    import huggingface_hub
    if importlib.metadata.version('huggingface_hub') != '2.4.0' or pathlib.Path(huggingface_hub.__file__).resolve() != pathlib.Path('/tmp/zogre-hf-sdk/site-packages/huggingface_hub/__init__.py'): raise RuntimeError('SDK_IDENTITY')
    pin(pathlib.Path('/tmp/zogre-hf-sdk/site-packages/huggingface_hub/hf_api.py'), SDK_API)
    pin(pathlib.Path('/tmp/zogre-hf-sdk/site-packages/huggingface_hub/_jobs_api.py'), SDK_JOBS)
    recovery_raw = (ROOT / 'recovery-selection.json').read_bytes(); recovery = json.loads(recovery_raw)
    if recovery.get('owner') != OWNER or recovery.get('collector') != OWNER or recovery.get('reviewer') != REVIEWER or recovery.get('jobId') != JOB or recovery.get('claimCommit') != CLAIM or recovery.get('entrySHA256') != globals().get('__verified_entry_sha256__') or recovery.get('launcherSHA256') != globals().get('__verified_launcher_sha256__') or globals().get('__verified_wrapper_sha256__') != WRAPPER or recovery.get('wrapperSHA256') != WRAPPER or recovery.get('bindingSHA256') != BINDING or recovery.get('recordsSHA256') != RECORDS or recovery.get('framesSHA256') != FRAMES or recovery.get('action') != 'COLLECT_EXISTING_ONCE': raise RuntimeError('RECOVERY_SELECTION_MISMATCH')
    binder = snapshot(ROOT / 'recovery-binding-draft-v1.py', BINDING, 'case_binding')
    records = snapshot(ROOT / 'recovery-records-draft-v5.py', RECORDS, 'case_records')
    frames = snapshot(ROOT / 'frame_transport.py', FRAMES, 'case_frames')
    authority = snapshot(ROOT / 'case-entry-v16.py', ENTRY16, 'case_authority')
    selected_raw = (ROOT / 'selection.json').read_bytes(); selected = json.loads(selected_raw)
    retained = {'return': (ROOT/'provider-return.json').read_bytes(), 'unknown': (ROOT/'submission-unknown.json').read_bytes(), 'intent': (ROOT/'case-intent-v16.json').read_bytes(), 'selection': selected_raw}
    def checks():
        authority.check_config()
        authority.current_state(selected, sha(selected_raw), ENTRY16, selected['programSHA256'], collection=True)
        recovery_authority(authority, recovery, sha(recovery_raw))
        if authority.run(['git', 'ls-remote', REMOTE, REF]).split() != [CLAIM, REF]: raise RuntimeError('CLAIM_MISMATCH')
    checks()
    api = HfApi()
    if bounded(30, lambda: api.whoami()).get('name') != 'Autismo-Yah': raise RuntimeError('NAMESPACE')
    info = bounded(30, lambda: api.inspect_job(namespace='Autismo-Yah', job_id=JOB))
    stage = getattr(info.status.stage, 'value', info.status.stage)
    observed = {'id': info.id, 'owner': getattr(info.owner, 'name', None), 'image': info.docker_image,
                'command': info.command, 'flavor': info.flavor, 'retry': info.retry,
                'labels': info.labels, 'stage': stage}
    if binder.validate(retained, observed) != 'BOUND_TERMINAL': raise RuntimeError('RECOVERY_BINDING_MISMATCH')
    checks()
    # An exclusive directory is a cooperating-process once guard. A crash after
    # mkdir is UNKNOWN, never delete or retry; it is not a global provider lock.
    attempt = ROOT / 'recovery-collection-v1'
    os.mkdir(attempt, 0o700)
    d = os.open(ROOT, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try: os.fsync(d)
    finally: os.close(d)
    durable(attempt/'collection-intent.json', {'kind':'COLLECTION_ONLY_RECOVERY', 'jobId':JOB,
        'collector':OWNER, 'claimCommit':CLAIM, 'originalSelectionSHA256':sha(selected_raw),
        'recoverySelectionSHA256':sha(recovery_raw), 'entrySHA256':globals().get('__verified_entry_sha256__'),
        'stage':'PENDING_NO_RETRY', 'providerStage':stage})
    try:
        # Persist only fixed, allowlisted facts; never serialize JobInfo wholesale.
        durable(attempt/'status.json', {'jobId':JOB,'namespace':'Autismo-Yah','providerStage':stage,
            'binding':'BOUND_TERMINAL','observedUTC':datetime.now(timezone.utc).isoformat(),
            'historicalSubmission':'UNKNOWN','originalLabelsEquality':False,'sdkExpandedLabels':'MATCH'})
        def fetch():
            return records.retain_records(api.fetch_job_logs(namespace='Autismo-Yah', job_id=JOB, follow=False), attempt/'sdk-records.jsonl')
        result = bounded(120, fetch)
        durable(attempt/'records-result.json', result)
        if result['outcome'] != 'COMPLETE' or result['cleanupErrors']:
            raise RuntimeError('SDK_RECORDS_INCOMPLETE')
        lines = [json.loads(line) for line in (attempt/'sdk-records.jsonl').read_text('ascii').splitlines()]
        if len(lines) != result['records']: raise RuntimeError('SDK_RECORD_COUNT')
        frame_result = bounded(60, lambda: frames.reconstruct_child_outputs(lines, attempt/'decoded'))
        durable(attempt/'frame-result.json', {'kind':'FRAME_RECONSTRUCTION_UNREVIEWED','result':frame_result})
        durable(attempt/'collection-readback.json', {'stage':'COLLECTED_UNREVIEWED','jobId':JOB,
            'providerStage':stage,'sdkRecordsSHA256':result['fileSHA256'],'sdkRecords':result['records'],
            'historicalSubmission':'UNKNOWN','scope':'SDK-decoded records; frame integrity and process closure require independent review'})
    except BaseException:
        try: durable(attempt/'collection-unknown.json', {'stage':'UNKNOWN_RECONCILE_SAME_JOB','jobId':JOB,
                'partialDetails':'SEE_RETAINED_FILES_OR_UNAVAILABLE','noAutomaticRecollection':True})
        except BaseException: pass  # controller must record UNKNOWN independently
        raise

if __name__ == '__main__': main()
