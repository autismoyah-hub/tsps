"""Draft read-only same-job inspection. Do not run until independently reviewed."""
import hashlib
import json
import signal
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path('/tmp/zogre-hf-case')
PINS = {
    'provider-return.json': 'caeb899c756da00e736647b9ac415fc131bdbd68eeb06359dd705350db1bd3ee',
    'submission-unknown.json': '3520fdbd3b91f7f665a523093131c3eb92124295d21addc9921f3c78a428a949',
    'case-intent-v16.json': '0407632ba392942b1caa4570e847daaa0c44d634b584fbfa7393c6253d1512b7',
    'selection.json': 'cadc7fa79eba914b2fc46fb444dc35f9cee5239e2a07dcd8dde480eb7aff35f8',
}
JOB = '6af14d46e6320c66fad7d183'
OWNER = '42689c7a-eddd-44ad-b8bb-04bd0f3c70de'
NAMESPACE = 'Autismo-Yah'
COMMIT = '15ecdacb4b53366c5d06ddd79adb1f9c480a0461'
REQUEST_SHA = 'dc70c6973733afa4e1463b1acb1c8a5a10805e2f4408b8258cb83b4ee393dfb4'

def main():
    files = {}
    for name, digest in PINS.items():
        raw = (ROOT / name).read_bytes()
        if hashlib.sha256(raw).hexdigest() != digest:
            raise RuntimeError('INPUT_MISMATCH')
        files[name] = json.loads(raw)
    returned = files['provider-return.json']
    intent = files['case-intent-v16.json']
    unknown = files['submission-unknown.json']
    request = intent['request']
    req_sha = hashlib.sha256(json.dumps(request, sort_keys=True).encode()).hexdigest()
    if not (returned['jobId'] == JOB and returned['namespace'] == NAMESPACE
            and returned['collector'] == OWNER and returned['claimCommit'] == COMMIT
            and returned['selectionSHA256'] == PINS['selection.json']
            and returned['requestSHA256'] == req_sha == REQUEST_SHA
            and intent['collector'] == OWNER and intent['claimCommit'] == COMMIT
            and intent['selectionSHA256'] == PINS['selection.json']
            and unknown['claimCommit'] == COMMIT and unknown['collector'] == OWNER
            and unknown['stage'] == 'UNKNOWN_RECONCILE_NO_RETRY'):
        raise RuntimeError('BINDING_MISMATCH')
    from huggingface_hub import HfApi
    def timeout(_signum, _frame):
        raise TimeoutError('INSPECTION_TIMEOUT')
    old = signal.signal(signal.SIGALRM, timeout)
    signal.alarm(60)
    try:
        api = HfApi()
        if api.whoami().get('name') != NAMESPACE:
            raise RuntimeError('IDENTITY_MISMATCH')
        info = api.inspect_job(job_id=JOB, namespace=NAMESPACE)
        status = getattr(getattr(info, 'status', None), 'stage', None)
        status = getattr(status, 'value', status)
        if status not in ('SCHEDULING', 'RUNNING', 'COMPLETED', 'ERROR', 'CANCELED', 'CANCELLED', 'PAUSED', 'RETRYING'):
            status = 'OTHER_OR_UNKNOWN'
        comparison = {
            'id': info.id == JOB,
            'owner': getattr(info.owner, 'name', None) == NAMESPACE,
            'image': info.docker_image == request['image'],
            'command': info.command == request['command'],
            'flavor': info.flavor == request['flavor'],
            'labels': info.labels == request['labels'],
            'retry': info.retry == 0,
        }
        print(json.dumps({'stage': 'READ_ONLY_INSPECTION', 'utc': datetime.now(timezone.utc).isoformat(),
                          'jobId': JOB, 'namespace': NAMESPACE, 'status': status,
                          'currentComparisons': comparison, 'historicalFailure': 'UNKNOWN',
                          'collection': 'NOT_STARTED'}))
    finally:
        signal.alarm(0)
        signal.signal(signal.SIGALRM, old)

if __name__ == '__main__':
    main()
