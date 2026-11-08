"""Stew case guard design, source-only. No CLI, provider access, selection or submission.

This is not an authorized executor. It deliberately cannot clear the hold. Review
and future authority must bind actual snapshots and define the trusted capture source.
"""
from pathlib import Path
import hashlib, json, re

SOURCES = frozenset(('2821d72135c78030aa7dce359b33162c7aeef457', 'c9606d5a082d6d9fb9722f49beec03d194f28e3c'))
TESTS = ['cooking.test.cjs']
COUNTS = {'cooking.test.cjs': 53}
FIXTURE_BLOB = 'a6bbc0b278ca37a46ac3bfa7f9b8b2cc7ac36c55'

def sha(raw): return hashlib.sha256(raw).hexdigest()

def snapshot(path, digest):
    raw = Path(path).read_bytes()
    if not re.fullmatch(r'[a-f0-9]{64}', digest) or sha(raw) != digest:
        raise ValueError('snapshot identity mismatch')
    return raw

def manifest(raw, digest, source):
    if source not in SOURCES or sha(raw) != digest: raise ValueError('manifest identity')
    doc = json.loads(raw)
    if set(doc) != {'schema','source','tests','expectedTestCounts','expectedInputs','sourceBlobs'}: raise ValueError('manifest keys')
    if (doc['schema'],doc['source'],doc['tests'],doc['expectedTestCounts']) != (1,source,TESTS,COUNTS): raise ValueError('manifest subject')
    a,b=doc['expectedInputs'],doc['sourceBlobs']
    if not isinstance(a,dict) or not isinstance(b,dict) or len(a)!=4918 or set(a)!=set(b): raise ValueError('map keyset')
    for path in a:
        if not isinstance(path,str) or path.startswith('/') or '\\' in path or ':' in path or any(x in ('','.','..') for x in path.split('/')): raise ValueError('path')
        if not re.fullmatch(r'[a-f0-9]{64}',a[path]) or not re.fullmatch(r'[a-f0-9]{40}',b[path]): raise ValueError('digest')
    if b.get('server/tests/cooking.test.cjs') != FIXTURE_BLOB: raise ValueError('fixture')
    return doc

def scan_custody(roots):
    """Nested local custody files are evidence, not provider clearance."""
    found=[]
    for root in roots:
        root=Path(root)
        if not root.is_dir(): raise ValueError('custody root missing')
        for path in root.rglob('*'):
            if path.is_symlink(): raise ValueError('symlink in custody tree')
            if path.is_file() and (path.name in ('submission.json','mapref-runtime-custody.json') or 'receipt' in path.name.lower()):
                try: obj=json.loads(path.read_bytes())
                except (ValueError,UnicodeError,OSError): raise ValueError('unreadable custody evidence') from None
                if not isinstance(obj,dict): raise ValueError('unknown custody evidence')
                found.append((str(path),obj))
    return found

def validate_fresh_capture(capture, subject, now):
    """Contract only: authentication and completeness must come from trusted capture plumbing."""
    if set(capture)!={'origin','capturedUTC','complete','project','jobs','evidenceSHA256'}: raise ValueError('capture schema')
    if capture['origin']!='AUTHENTICATED_DIRECT_COMPLETE_CAPTURE' or capture['complete'] is not True: raise ValueError('capture provenance')
    if capture['project']!=subject['project'] or not isinstance(capture['jobs'],list): raise ValueError('capture project')
    if type(capture['capturedUTC']) not in (int,float) or not 0 <= now-capture['capturedUTC'] <= 300: raise ValueError('capture freshness')
    if not re.fullmatch(r'[a-f0-9]{64}',capture['evidenceSHA256']): raise ValueError('capture identity')
    for job in capture['jobs']:
        if not isinstance(job,dict) or set(job)!={'source','name','project','status'} or any(not isinstance(job[k],str) or not job[k] for k in job): raise ValueError('unknown job fields')
        if job['project']!=subject['project']: raise ValueError('unexpected project')
        if job['source']==subject['source'] or job['name']==subject['name']: raise ValueError('matching source or name: HOLD')
    return True

def execution_gate(*_args, **_kwargs):
    raise RuntimeError('HELD: executor unresolved, no selection or submission authority')
