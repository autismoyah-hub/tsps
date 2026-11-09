"""Unselected pure recovery binding predicate; does not call providers."""
import hashlib
import json

JOB = '6af14d46e6320c66fad7d183'
NAMESPACE = 'Autismo-Yah'
OWNER = '42689c7a-eddd-44ad-b8bb-04bd0f3c70de'
SOURCE = '4619ba5838569a68b0f56bb46ab6b2b77c930586'
CLAIM = '15ecdacb4b53366c5d06ddd79adb1f9c480a0461'
SELECT = 'cadc7fa79eba914b2fc46fb444dc35f9cee5239e2a07dcd8dde480eb7aff35f8'
REQUEST = 'dc70c6973733afa4e1463b1acb1c8a5a10805e2f4408b8258cb83b4ee393dfb4'
PINS = {
    'return': 'caeb899c756da00e736647b9ac415fc131bdbd68eeb06359dd705350db1bd3ee',
    'unknown': '3520fdbd3b91f7f665a523093131c3eb92124295d21addc9921f3c78a428a949',
    'intent': '0407632ba392942b1caa4570e847daaa0c44d634b584fbfa7393c6253d1512b7',
    'selection': SELECT,
}


def validate(retained, observed):
    """Return a fixed code. Caller must pin this executed snapshot and inputs.

    `retained` contains exact original byte strings; `observed` contains only
    SDK-projected fields from a fresh exact-job inspect. Claim and authorization
    checks, collection intent, network, and storage are separate obligations.
    """
    try:
        if set(retained) != set(PINS):
            return 'ORIGINAL_SET_MISMATCH'
        if any(hashlib.sha256(retained[k]).hexdigest() != v for k, v in PINS.items()):
            return 'ORIGINAL_HASH_MISMATCH'
        docs = {k: json.loads(v) for k, v in retained.items()}
        ret, unknown, intent, selected = (docs[k] for k in ('return', 'unknown', 'intent', 'selection'))
        request = intent['request']
        if hashlib.sha256(json.dumps(request, sort_keys=True).encode()).hexdigest() != REQUEST:
            return 'REQUEST_HASH_MISMATCH'
        if (ret.get('stage') != 'RETURNED_UNINSPECTED' or ret.get('jobId') != JOB
            or ret.get('namespace') != NAMESPACE or ret.get('requestSHA256') != REQUEST
            or unknown.get('stage') != 'UNKNOWN_RECONCILE_NO_RETRY'
            or intent.get('source') != SOURCE or intent.get('namespace') != NAMESPACE
            or selected.get('owner') != OWNER or selected.get('collector') != OWNER
            or any(d.get('claimCommit') != CLAIM or d.get('collector') != OWNER
                   or d.get('selectionSHA256') != SELECT for d in (ret, unknown, intent))):
            return 'ORIGINAL_BINDING_MISMATCH'
        if not isinstance(observed, dict):
            return 'OBSERVATION_INVALID'
        if observed.get('id') != JOB or observed.get('owner') != NAMESPACE:
            return 'JOB_IDENTITY_MISMATCH'
        if (observed.get('image') != request['image'] or observed.get('command') != request['command']
            or observed.get('flavor') != request['flavor'] or type(observed.get('retry')) is not int
            or observed.get('retry') != 0):
            return 'JOB_REQUEST_MISMATCH'
        labels = request['labels']
        if not isinstance(labels, dict) or set(labels) != {'project', 'source', 'case', 'purpose'}:
            return 'REQUEST_LABEL_SCHEMA'
        if observed.get('labels') != dict(labels, name=request['name']):
            return 'JOB_LABEL_MISMATCH'
        if observed.get('stage') != 'ERROR':
            return 'STATUS_NOT_EXPECTED'
        return 'BOUND_TERMINAL'
    except Exception:
        return 'INVALID_OR_INCOMPLETE'
