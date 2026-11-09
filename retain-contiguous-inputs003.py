"""Offline retained-input decoder. Never fetches/runs/fills expected rows.
Integrity is of retained contiguous returned fragments, not an emitter per-frame digest.
"""
import pathlib,json,hashlib,argparse,re
sha=lambda b:hashlib.sha256(b).hexdigest()
def unique(pairs):
 out={}
 for k,v in pairs:
  if k in out:raise ValueError('Duplicate actual JSON key')
  out[k]=v
 return out
def decode(raw,manifest):
 lines=raw.splitlines();marker=b'{"event":"inputs",';starts=[i for i,x in enumerate(lines) if x.startswith(marker)]
 if len(starts)!=1:raise ValueError('Require one actual inputs event, no mixed/repeated execution')
 begin=starts[0];parts=[];parsed=None;size=0
 for index in range(begin,min(len(lines),begin+1024)):
  piece=lines[index]
  if not piece or (index>begin and piece.startswith(b'{"event":')):raise ValueError('Missing/interleaved actual fragments')
  size+=len(piece)
  if size>4*1024*1024:raise ValueError('Actual event bound exceeded')
  parts.append(piece)
  try:parsed=json.loads(b''.join(parts).decode('utf-8'),object_pairs_hook=unique)
  except (json.JSONDecodeError,UnicodeDecodeError):continue
  break
 if not isinstance(parsed,dict):raise ValueError('Truncated actual event')
 if parsed.get('event')!='inputs' or parsed.get('source')!=manifest['source']:raise ValueError('Actual source/event mismatch')
 actual=parsed.get('inputs')
 if not isinstance(actual,dict) or actual!=manifest['expectedInputs']:raise ValueError('Actual complete map/count/SHA mismatch')
 if len(actual)!=4890 and manifest['source']=='78fb0d1eeb5a1add5e932959788a13b15a145753':raise ValueError('Grand count mismatch')
 joined=b''.join(parts)
 report=dict(startLine1=begin+1,endLine1=begin+len(parts),fragmentLengths=[len(x) for x in parts],rawLogSHA256=sha(raw),retainedEventSHA256=sha(joined),retainedEventBytes=len(joined),actualCount=len(actual),source=parsed['source'],expectedRowsInserted=0,chunkDigestsOrigin='COMPUTED_FROM_RETAINED_RETURNED_FRAGMENTS_NOT_RUNTIME_EMITTED',runtimeGitblobTelemetry=False,runtimeGitblobQualification='Reaching input event follows hardfail per-row computed Gitblob guard in pinned successor; distinct from printed SHA telemetry',limitation='No runtime per-frame signature/digest in frozen successor. Missing/interleaved/invalid/duplicate/mismatched event HOLD; no repair from manifest.')
 return joined,parsed,report

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--raw',type=pathlib.Path,required=True);ap.add_argument('--manifest',type=pathlib.Path,required=True);ap.add_argument('--submission',type=pathlib.Path,required=True);ap.add_argument('--terminal-status',type=pathlib.Path,required=True);ap.add_argument('--out',type=pathlib.Path,required=True);a=ap.parse_args()
 submission=json.loads(a.submission.read_bytes());status=json.loads(a.terminal_status.read_bytes());manifest_raw=a.manifest.read_bytes();manifest=json.loads(manifest_raw)
 assert a.raw.resolve().parent==a.terminal_status.resolve().parent==a.submission.resolve().parent
 assert a.raw.name=='logs-'+a.terminal_status.stem.removeprefix('status-')+'.txt'
 spec=submission['plan']['expectedInputsRef'];assert len(manifest_raw)==spec['bytes'] and sha(manifest_raw)==spec['sha256']
 assert len(manifest['expectedInputs'])==spec['count']
 assert status['id']==submission['job']['id'] and status['status']['stage'] in ['COMPLETED','ERROR','CANCELED','CANCELLED']
 assert submission['request']['command']==submission['job']['command']==status['command']
 assert submission['plan']['source']==manifest['source']
 joined,event,report=decode(a.raw.read_bytes(),manifest);report.update(jobId=status['id'],terminal=status['status'],commandBound=True)
 a.out.mkdir(exist_ok=False);(a.out/'actual-input-event.raw').write_bytes(joined);(a.out/'actual-input-event.json').write_text(json.dumps(event,indent=2)+'\n',encoding='utf-8');(a.out/'fragment-receipt.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
 print(json.dumps(report))
if __name__=='__main__':main()
