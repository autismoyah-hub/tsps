#!/usr/bin/env python3
"""Classify the retained test stream only; terminal/custody evidence is separate."""
import argparse
import hashlib
import json
import re
from pathlib import Path

SOURCES = {
    'before': '247ed387d6a631a00440904a00aa8e1a2e44a962',
    'after': '88b5e5793039c1f246c2a4096206664e50c51003',
}
FIRST = "Belona's actual exchange charges and returns the Wiki mineral prices"
SECOND = 'Belona refuses other stock and preserves locked items and a full currency stack'
THIRD = "another reward shop still refuses Belona's stock"

def classify(label, raw):
    text = raw.decode('utf-8', errors='strict')
    events = []
    for line in text.splitlines():
        try:
            value = json.loads(line)
            if isinstance(value, dict) and 'event' in value:
                events.append(value)
        except json.JSONDecodeError:
            pass
    def one(event):
        matches = [v for v in events if v.get('event') == event]
        if len(matches) != 1:
            raise ValueError(f'{event}: expected one event, got {len(matches)}')
        return matches[0]
    inputs = one('inputs')
    if inputs.get('source') != SOURCES[label]:
        raise ValueError('source mismatch')
    cache = one('cache-equivalence')
    if cache.get('status') != 'PASS':
        raise ValueError('cache equivalence not established')
    stage = [v for v in events if v.get('event') == 'stage-result' and v.get('name') == 'test:mineral-exchange.test.cjs']
    if len(stage) != 1:
        raise ValueError('test stage missing or duplicated')
    stage = stage[0]
    if stage.get('signal') is not None or stage.get('timedOut') or stage.get('leftoverGroup'):
        raise ValueError('test process did not close normally')
    seal = one('source-closure')
    if not seal.get('complete') or not seal.get('sealsUnchanged'):
        raise ValueError('source/dist/cache seals incomplete or changed')
    summary = {}
    for key in ('tests', 'pass', 'fail', 'skipped', 'cancelled'):
        found = re.findall(r'^# ' + key + r' (\d+)\s*$', text, flags=re.M)
        if len(found) != 1:
            raise ValueError(f'missing or duplicate TAP summary: {key}')
        summary[key] = int(found[0])
    names = re.findall(r'^(not ok|ok) (\d+) - (.+)$', text, flags=re.M)
    wanted = [('not ok' if label == 'before' else 'ok', '1', FIRST), ('ok', '2', SECOND), ('ok', '3', THIRD)]
    if names != wanted:
        raise ValueError('TAP test names/results do not match the frozen fixture')
    expected = {'tests': 3, 'pass': 2 if label == 'before' else 3,
                'fail': 1 if label == 'before' else 0, 'skipped': 0, 'cancelled': 0}
    if summary != expected:
        raise ValueError(f'TAP summary mismatch: {summary}')
    result = one('native-qualification-result')
    if result.get('source') != SOURCES[label]:
        raise ValueError('result source mismatch')
    if label == 'before':
        if stage.get('code') != 1 or result.get('status') != 'PAYLOAD_NONZERO_OR_TAP_UNCLASSIFIED':
            raise ValueError('before was not a normal failing test process')
        if not re.search(r'mineral-exchange\.test\.cjs:107:\d+', text):
            raise ValueError('expected refund value assertion was not reached')
        if 'shop will buy for' not in text or 'AssertionError' not in text:
            raise ValueError('refund value assertion diagnostic missing')
    elif stage.get('code') != 0 or not stage.get('tapValid') or result.get('status') != 'TESTS_PASS_AWAIT_SANDBOX_TERMINAL':
        raise ValueError('after did not report a valid passing test')
    return {'classification': 'TARGET_ASSERTION_REACHED' if label == 'before' else 'TESTS_PASS',
            'source': SOURCES[label], 'rawSHA256': hashlib.sha256(raw).hexdigest(),
            'summary': summary, 'terminalAndCustody': 'NOT_ASSESSED'}

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('label', choices=SOURCES)
    parser.add_argument('raw', type=Path)
    args = parser.parse_args()
    try:
        print(json.dumps(classify(args.label, args.raw.read_bytes()), sort_keys=True))
    except (ValueError, UnicodeError, OSError) as error:
        print(json.dumps({'classification': 'UNCLASSIFIED', 'reason': str(error), 'terminalAndCustody': 'NOT_ASSESSED'}, sort_keys=True))
        raise SystemExit(1)
