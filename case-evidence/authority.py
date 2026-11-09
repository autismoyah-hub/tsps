"""Exact distinct peer/owner assistant-turn authority. Pure and side-effect free."""
import hashlib
import json
import re

OWNER = 'a57b09b1-fcf5-4aea-b25e-7f19910af934'
AUTHOR = 'a57b09b1-fcf5-4aea-b25e-7f19910af934'
UUID = re.compile(r'^[0-9a-f]{8}-(?:[0-9a-f]{4}-){3}[0-9a-f]{12}$')


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True).encode()


def exact_turn(transcript_raw, session_id, expected):
    if not UUID.fullmatch(session_id):
        raise ValueError('SESSION_ID')
    turns = json.loads(transcript_raw)['turns']
    if not isinstance(turns, list):
        raise ValueError('TRANSCRIPT_SCHEMA')
    hits = []
    for turn in turns:
        if turn.get('role') != 'assistant' or turn.get('kind') != 'text' or not isinstance(turn.get('ts'), str) or not turn['ts']:
            continue
        try:
            actual = json.loads(turn.get('text', ''))
        except (ValueError, TypeError):
            continue
        if actual == expected:
            hits.append(turn)
    if len(hits) != 1:
        raise ValueError('EXACT_TURN_NOT_UNIQUE')
    turn = hits[0]
    stable = {'session':session_id,'role':'assistant','kind':'text','ts':turn['ts'],
              'text':turn.get('text')}
    return {'session': session_id, 'transcriptSHA256': hashlib.sha256(transcript_raw).hexdigest(),
            'turnUTC': turn['ts'], 'turnSHA256':hashlib.sha256(canonical(stable)).hexdigest(),
            'valueSHA256': hashlib.sha256(canonical(expected)).hexdigest()}


def reviewer_receipt(receipt_raw, transcript_raw, reviewer, subject, verdict):
    if reviewer in (OWNER, AUTHOR) or not UUID.fullmatch(reviewer):
        raise ValueError('REVIEWER_NOT_INDEPENDENT')
    receipt = json.loads(receipt_raw)
    expected = {'reviewer': reviewer, 'verdict': verdict, 'subject': subject}
    if receipt != expected:
        raise ValueError('REVIEW_RECEIPT_SUBJECT')
    proof = exact_turn(transcript_raw, reviewer, expected)
    proof['receiptSHA256'] = hashlib.sha256(receipt_raw).hexdigest()
    return proof
