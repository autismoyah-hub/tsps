"""Same-job terminal-only collection; one owner, no submit or replay."""
from pathlib import Path
import datetime, hashlib, json, os
from frame_transport import reconstruct_child_outputs
ROOT=Path(__file__).resolve().parent
CONTROL=ROOT.parent
OWNER='d427048d-86cc-4ad2-b3cc-bebade89e457'
PLAN_SHA='3355f4d839fcfff9d8de9020901bfca45a69a5c745eb3ee59d943cb308e79e44'
def digest(raw):return hashlib.sha256(raw).hexdigest()
def exclusive(path,data):
 with path.open('x',encoding='utf8') as f:
  json.dump(data,f,indent=2,default=str);f.write('\n');f.flush();os.fsync(f.fileno())
def main():
 if os.environ.get('CCC_SESSION_ID')!=OWNER:raise ValueError('wrong collector')
 raw=(ROOT/'plan.json').read_bytes()
 if digest(raw)!=PLAN_SHA:raise ValueError('plan identity')
 plan=json.loads(raw);receipt=json.loads((ROOT/'submission.json').read_bytes());binding=json.loads((CONTROL/'submitted-job-id.json').read_bytes());intent=json.loads((CONTROL/'submission-intent.json').read_bytes())
 if receipt['plan']!=plan or receipt['namespace']!='Autismo-Yah' or receipt['job']['id']!=binding['jobId'] or intent['owner']!=OWNER or intent['collector']!=OWNER or intent['planSHA256']!=PLAN_SHA or binding['planSHA256']!=PLAN_SHA:raise ValueError('same-job binding')
 if (CONTROL/'collection-intent.json').exists():raise ValueError('collection already attempted; reconcile without replay')
 from huggingface_hub import HfApi
 api=HfApi();namespace=api.whoami()['name']
 if namespace!='Autismo-Yah':raise ValueError('namespace')
 info=api.inspect_job(job_id=binding['jobId'],namespace=namespace)
 if info.id!=binding['jobId'] or info.command!=receipt['request']['command']:raise ValueError('provider job identity')
 stage=getattr(info.status.stage,'value',info.status.stage)
 if stage not in ('COMPLETED','ERROR','CANCELED','CANCELLED'):
  print(json.dumps({'state':'NONTERMINAL','jobId':info.id,'stage':stage}));return
 stamp=datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ')
 exclusive(CONTROL/'collection-intent.json',{'jobId':info.id,'collector':OWNER,'planSHA256':PLAN_SHA,'utc':stamp,'state':'COLLECTION_STARTING_NO_REPLAY'})
 status={'id':info.id,'status':{'stage':stage,'message':info.status.message},'started_at':str(info.started_at),'finished_at':str(info.finished_at),'commandMatches':True,'planSHA256':PLAN_SHA}
 exclusive(ROOT/f'status-{stamp}.json',status)
 logs=list(api.fetch_job_logs(job_id=info.id,namespace=namespace,follow=False))
 if not all(isinstance(line,str) for line in logs):raise ValueError('unknown log shape')
 log=ROOT/f'logs-{stamp}.txt'
 with log.open('x',encoding='utf8') as f:f.write('\n'.join(logs)+'\n');f.flush();os.fsync(f.fileno())
 decoded=ROOT/f'decoded-{stamp}'
 framed=reconstruct_child_outputs(logs,decoded)
 exclusive(ROOT/f'collection-{stamp}.json',{'jobId':info.id,'stage':stage,'rawLog':log.name,'rawLogSHA256':digest(log.read_bytes()),'framedComplete':framed['complete'] if framed else None,'collector':OWNER,'scope':'HF terminal and process-group output only; no sandbox-wide closure claim'})
 print(json.dumps({'state':'COLLECTED','jobId':info.id,'stage':stage,'framedComplete':framed['complete'] if framed else None}))
if __name__=='__main__':main()
