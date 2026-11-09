"""READ-ONLY DRAFT: list visible project jobs, no provider mutation."""
import json,signal
from datetime import datetime,timezone
from huggingface_hub import HfApi
SOURCE='4619ba5838569a68b0f56bb46ab6b2b77c930586'
CASE='zogre-uglug-baseline-42689c'
def main():
 def alarm(_sig,_frame):raise TimeoutError('BOUNDED_LIST_TIMEOUT')
 old=signal.signal(signal.SIGALRM,alarm);signal.alarm(60)
 try:
  api=HfApi();identity=api.whoami().get('name')
  if identity!='Autismo-Yah':raise RuntimeError('NAMESPACE_MISMATCH')
  matches=[];count=0
  for job in api.list_jobs(namespace=identity,labels={'project':'tsps'},timeout=20):
   count+=1;labels=job.labels or {}
   if labels.get('source')==SOURCE or labels.get('case')==CASE:
    stage=getattr(job.status.stage,'value',job.status.stage)
    matches.append({'id':job.id,'stage':stage,'sourceMatch':labels.get('source')==SOURCE,'caseMatch':labels.get('case')==CASE})
  print(json.dumps({'stage':'READ_ONLY_LIST_COMPLETE','utc':datetime.now(timezone.utc).isoformat(),'namespace':identity,'projectJobsVisible':count,'matches':matches,'scope':'all returned states and project-labelled jobs; unlabelled, unindexed and eventual-consistency gaps unknown'}))
 finally:signal.alarm(0);signal.signal(signal.SIGALRM,old)
if __name__=='__main__':main()
