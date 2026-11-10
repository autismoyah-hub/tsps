"""Retain parsed actuals from existing terminal receipts; never launches jobs."""
from pathlib import Path
import argparse
import hashlib
import json
import re

root=Path(__file__).resolve().parent
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('case')
args=parser.parse_args()
case=root/args.case
status_file=sorted(case.glob('status-*.json'))[-1]
stamp=status_file.stem[7:]
log=case/f'logs-{stamp}.txt'
status=json.loads(status_file.read_text(encoding='utf-8'))
assert status['status']['stage'] in ('COMPLETED','ERROR','CANCELED','CANCELLED'), 'Platform terminal pending'
raw=log.read_text(encoding='utf-8')
events=[]
for line in raw.splitlines():
    try:
        row=json.loads(line)
        if isinstance(row,dict) and 'event' in row and row['event']!='child-output-frame': events.append(row)
    except ValueError: pass
decoded=case/f'decoded-{stamp}'
manifest=decoded/'manifest.json'
result={'jobId':status['id'],'source':json.loads((case/'pilot.json').read_text())['source'],
        'platformStatus':status['status'],'startedAt':status['started_at'],'finishedAt':status['finished_at'],
        'stageResults':[x for x in events if x['event']=='stage-result'],
        'integrityEvents':[x for x in events if x['event']!='stage-result'],
        'rawLog':log.name,'rawLogSHA256':hashlib.sha256(log.read_bytes()).hexdigest(),
        'statusReceipt':status_file.name,
        'failureExcerpts':[raw[max(0,m.start()-100):m.start()+2200] for m in re.finditer(r'^not ok ',raw,re.M)],
        'decodedManifest':str(manifest.relative_to(case)) if manifest.exists() else None,
        'outputTransport':json.loads(manifest.read_text()) if manifest.exists() else None,
        'scope':'Actual immutable result only; per-stage exits retain failures, setup vs intended assertions require owner/reviewer classification. No automatic rerun.',
        'closureLimit':'Process-group-only evidence plus HF terminal, not independent sandbox descendant census or Windows equivalence.'}
target=case/'result.json'
with target.open('x',encoding='utf-8') as out: out.write(json.dumps(result,indent=2)+'\n')
print(json.dumps({'path':str(target),'jobId':result['jobId'],'platform':status['status']['stage'],
                  'tests':[x for x in result['stageResults'] if x['name'].startswith('test:')],
                  'framedComplete':result['outputTransport']['complete'] if result['outputTransport'] else None}))
