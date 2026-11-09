import json
from huggingface_hub import HfApi
sources={'e6ac3bbe7f1c7e8f08c23df7dc63859b9cd079ef','3a224eee211af4a6701956c161f2602a567d3c5c'}
hits=[]
for job in HfApi().list_jobs(namespace='Autismo-Yah', labels={'project':'tsps'}):
 labels=getattr(job,'labels',None) or {}
 if labels.get('source') in sources:
  stage=getattr(job.status.stage,'value',job.status.stage)
  hits.append({'id':job.id,'source':labels.get('source'),'stage':str(stage)})
print(json.dumps({'stage':'DUPLICATE_SCAN','hits':hits}))
