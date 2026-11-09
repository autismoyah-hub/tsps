"""Same-owner terminal collection; offline-only recovery never refetches logs."""
from pathlib import Path
import argparse,datetime,hashlib,json,os,resource,subprocess,sys
CASE=Path(__file__).resolve().parent; ROOT=CASE.parent
OWNER='3f20dcbd-b1d9-4e4a-bc47-a3b02d6ee6bc'; PLAN='a77dc6fa3f6e3c23b9e17fc6b7f0f2e5ff3726f6caec3205983071f6805c669c'; RETAINER='b399ca13e8fbfee6f7633a9d430d85c8204774c2941a2e4d5f74c642820fa690'
def digest(data):return hashlib.sha256(data).hexdigest()
def req(ok,msg):
 if not ok:raise ValueError(msg)
def exclusive(path,data):
 with path.open('x',encoding='utf8') as f:json.dump(data,f,indent=2,default=str);f.write('\n');f.flush();os.fsync(f.fileno())
 fd=os.open(path.parent,os.O_RDONLY);os.fsync(fd);os.close(fd)
def load_bindings():
 req(os.environ.get('CCC_SESSION_ID')==OWNER,'wrong owner');raw=(CASE/'plan.json').read_bytes();req(digest(raw)==PLAN,'plan pin');plan=json.loads(raw)
 receipt=json.loads((CASE/'submission.json').read_bytes());binding=json.loads((ROOT/'submitted-job-id.json').read_bytes());intent=json.loads((ROOT/'submission-intent.json').read_bytes());request=receipt['request'];job=receipt['job']
 req(intent['owner']==OWNER and intent['collector']==OWNER and intent['planSHA256']==PLAN and binding['planSHA256']==PLAN and receipt['plan']==plan and receipt['namespace']=='Autismo-Yah' and job['id']==binding['jobId'],'same job binding')
 req(digest(json.dumps(request,sort_keys=True,separators=(',',':')).encode())==intent['requestSHA256'],'request pin')
 req(digest((CASE/'payload.cjs').read_bytes())==intent['payloadSHA256'] and digest(request['command'][-1].encode())==receipt['payloadSHA256'],'payload pin')
 req(job.get('command')==request['command'] and job.get('owner')=='Autismo-Yah' and job.get('docker_image')==request['image'] and job.get('flavor')==request['flavor'],'receipt tuple')
 labels=job.get('labels');req(isinstance(labels,dict) and labels.get('name')==request['name'] and all(labels.get(k)==v for k,v in request['labels'].items()),'receipt labels')
 req(digest((CASE/'retain-contiguous-inputs003.py').read_bytes())==RETAINER,'retainer pin')
 return plan,receipt,binding,intent

def retain(stamp,recovery):
 complete=json.loads((ROOT/'iterator-ended.json').read_bytes());status=CASE/f'status-{stamp}.json';log=CASE/f'logs-{stamp}.txt'
 req(complete['stamp']==stamp and digest(status.read_bytes())==complete['statusSHA256'] and digest(log.read_bytes())==complete['rawSHA256'],'iterator-ended binding')
 out=CASE/(('native-recovery-' if recovery else 'native-input-')+stamp)
 tag='offline-retain' if recovery else 'native-retain'
 stdout=ROOT/(tag+'-stdout.txt');stderr=ROOT/(tag+'-stderr.txt')
 def limit():resource.setrlimit(resource.RLIMIT_FSIZE,(4*1024*1024,4*1024*1024))
 with stdout.open('xb') as outlog,stderr.open('xb') as errlog:
  run=subprocess.run([sys.executable,'-E','-S','-B',str(CASE/'retain-contiguous-inputs003.py'),'--raw',str(log),'--manifest',str(CASE/'input-manifest.json'),'--submission',str(CASE/'submission.json'),'--terminal-status',str(status),'--out',str(out)],stdout=outlog,stderr=errlog,timeout=120,preexec_fn=limit)
  outlog.flush();os.fsync(outlog.fileno());errlog.flush();os.fsync(errlog.fileno())
 exclusive(ROOT/(tag+'-result.json'),{'exit':run.returncode,'stdoutSHA256':digest(stdout.read_bytes()),'stderrSHA256':digest(stderr.read_bytes()),'stdoutBytes':stdout.stat().st_size,'stderrBytes':stderr.stat().st_size,'output':out.name,'scope':'offline reconstruction, not provider recollection'})
 if run.returncode:raise ValueError('native retainer failed; retained raw remains authoritative')
 print(json.dumps({'state':'RAW_RETAINED','nativeInput':'PASS','jobId':complete['jobId'],'terminal':complete['stage']}))

def classify_stage(stage):
 if stage in ('SCHEDULING','RUNNING'):return 'NONTERMINAL'
 req(stage in ('COMPLETED','ERROR','CANCELED'),'unknown or unsupported provider stage')
 return 'TERMINAL'

def main():
 p=argparse.ArgumentParser();p.add_argument('--recover-offline',action='store_true');a=p.parse_args();plan,receipt,binding,intent=load_bindings()
 if a.recover_offline:
  req((ROOT/'collection-intent.json').exists() and (ROOT/'iterator-ended.json').exists() and not (ROOT/'offline-retain-intent.json').exists(),'offline recovery prerequisites')
  complete=json.loads((ROOT/'iterator-ended.json').read_bytes()); original=json.loads((ROOT/'collection-intent.json').read_bytes());req(complete['jobId']==binding['jobId'] and original.get('jobId')==binding['jobId'] and original.get('collector')==OWNER and original.get('planSHA256')==PLAN and original.get('stamp')==complete.get('stamp') and original.get('state')=='COLLECTION_STARTING_NO_REPLAY','offline intent binding')
  exclusive(ROOT/'offline-retain-intent.json',{'jobId':binding['jobId'],'state':'OFFLINE_ONLY_NO_PROVIDER'})
  return retain(complete['stamp'],True)
 req(not (ROOT/'collection-intent.json').exists(),'collection attempted; reconcile, do not refetch')
 from huggingface_hub import HfApi
 api=HfApi();req(type(api).__name__=='ExplicitTokenApi' and api.whoami()['name']=='Autismo-Yah','authenticated adapter')
 info=api.inspect_job(job_id=binding['jobId'],namespace='Autismo-Yah');request=receipt['request'];labels=info.labels
 req(info.id==binding['jobId'] and info.command==request['command'] and info.owner.name=='Autismo-Yah' and info.docker_image==request['image'] and info.flavor==request['flavor'],'provider tuple')
 req(isinstance(labels,dict) and labels.get('name')==request['name'] and all(labels.get(k)==v for k,v in request['labels'].items()),'provider labels')
 stage=getattr(info.status.stage,'value',info.status.stage)
 if classify_stage(stage)=='NONTERMINAL':
  print(json.dumps({'state':'NONTERMINAL','jobId':info.id,'stage':stage}));return
 stamp=datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ');exclusive(ROOT/'collection-intent.json',{'jobId':info.id,'collector':OWNER,'planSHA256':PLAN,'stamp':stamp,'state':'COLLECTION_STARTING_NO_REPLAY'})
 status=CASE/f'status-{stamp}.json';exclusive(status,{'id':info.id,'status':{'stage':stage,'message':info.status.message},'command':info.command,'owner':info.owner.name,'image':info.docker_image,'flavor':info.flavor,'labels':labels,'retry':info.retry,'started_at':str(info.started_at),'finished_at':str(info.finished_at),'planSHA256':PLAN})
 log=CASE/f'logs-{stamp}.txt';h=hashlib.sha256();total=0
 with log.open('xb') as f:
  for line in api.fetch_job_logs(job_id=info.id,namespace='Autismo-Yah',follow=False):
   req(isinstance(line,str),'log shape'); data=(line+'\n').encode();total+=len(data)
   if total>128*1024*1024:raise ValueError('log limit; partial raw retained, no refetch')
   f.write(data);h.update(data)
  f.flush();os.fsync(f.fileno())
 fd=os.open(CASE,os.O_RDONLY);os.fsync(fd);os.close(fd)
 exclusive(ROOT/'iterator-ended.json',{'jobId':info.id,'stamp':stamp,'stage':stage,'statusSHA256':digest(status.read_bytes()),'rawSHA256':h.hexdigest(),'rawBytes':total,'scope':'SDK iterator ended; not proof of complete provider logs'})
 return retain(stamp,False)
if __name__=='__main__':main()
