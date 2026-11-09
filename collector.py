"""Same-job terminal collection; never submits or retries an ambiguous collection."""
from pathlib import Path
import datetime,hashlib,json,os
from frame_transport import reconstruct_child_outputs
CASE=Path(__file__).resolve().parent
ROOT=CASE.parent
OWNER='3f20dcbd-b1d9-4e4a-bc47-a3b02d6ee6bc'
PLAN='a77dc6fa3f6e3c23b9e17fc6b7f0f2e5ff3726f6caec3205983071f6805c669c'
def digest(data):return hashlib.sha256(data).hexdigest()
def exclusive(path,data):
 with path.open('x',encoding='utf8') as f:
  json.dump(data,f,indent=2,default=str);f.write('\n');f.flush();os.fsync(f.fileno())
 fd=os.open(path.parent,os.O_RDONLY);os.fsync(fd);os.close(fd)
def main():
 if os.environ.get('CCC_SESSION_ID')!=OWNER:raise ValueError('wrong collector')
 planraw=(CASE/'plan.json').read_bytes()
 if digest(planraw)!=PLAN:raise ValueError('plan identity')
 plan=json.loads(planraw); intent=json.loads((ROOT/'submission-intent.json').read_bytes()); binding=json.loads((ROOT/'submitted-job-id.json').read_bytes());receipt=json.loads((CASE/'submission.json').read_bytes())
 if intent['owner']!=OWNER or intent['collector']!=OWNER or intent['planSHA256']!=PLAN or binding['planSHA256']!=PLAN or receipt['plan']!=plan or receipt['namespace']!='Autismo-Yah' or receipt['job']['id']!=binding['jobId']:raise ValueError('same job binding')
 if (ROOT/'collection-intent.json').exists():raise ValueError('collection already attempted; reconcile without replay')
 from huggingface_hub import HfApi
 api=HfApi()
 if type(api).__name__!='ExplicitTokenApi' or api.whoami()['name']!='Autismo-Yah':raise ValueError('authorized adapter/namespace')
 info=api.inspect_job(job_id=binding['jobId'],namespace='Autismo-Yah')
 if info.id!=binding['jobId'] or info.command!=receipt['request']['command'] or info.owner.name!='Autismo-Yah':raise ValueError('provider identity')
 stage=getattr(info.status.stage,'value',info.status.stage)
 if stage not in ('COMPLETED','ERROR','CANCELED','CANCELLED'):
  print(json.dumps({'state':'NONTERMINAL','jobId':info.id,'stage':stage}));return
 stamp=datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ')
 exclusive(ROOT/'collection-intent.json',{'jobId':info.id,'collector':OWNER,'planSHA256':PLAN,'utc':stamp,'state':'COLLECTION_STARTING_NO_REPLAY'})
 exclusive(CASE/f'status-{stamp}.json',{'id':info.id,'stage':stage,'message':info.status.message,'started_at':str(info.started_at),'finished_at':str(info.finished_at),'planSHA256':PLAN})
 logs=list(api.fetch_job_logs(job_id=info.id,namespace='Autismo-Yah',follow=False))
 if not all(isinstance(line,str) for line in logs):raise ValueError('unknown log shape')
 raw=('\n'.join(logs)+'\n').encode(); path=CASE/f'logs-{stamp}.txt'
 with path.open('xb') as f:f.write(raw);f.flush();os.fsync(f.fileno())
 fd=os.open(CASE,os.O_RDONLY);os.fsync(fd);os.close(fd)
 framed=reconstruct_child_outputs(logs,CASE/f'decoded-{stamp}')
 exclusive(CASE/f'collection-{stamp}.json',{'jobId':info.id,'stage':stage,'rawLog':path.name,'rawLogSHA256':digest(raw),'framedComplete':framed['complete'] if framed else None,'collector':OWNER,'scope':'terminal status and process-group output; detached closure unproved'})
 print(json.dumps({'state':'COLLECTED','jobId':info.id,'stage':stage,'framedComplete':framed['complete'] if framed else None}))
if __name__=='__main__':main()
