"""Case-specific prestart candidate. No provider action; output is not a review."""
import argparse, datetime, hashlib, json, os, subprocess, urllib.request
from pathlib import Path
ROOT=Path(__file__).resolve().parent
REPO=Path('/workspace/repo')
SOURCE='daff5ac899dc1bae79ab6fea6d66df15f0ef855f'
REF='refs/heads/woodcutting-inventory-interruption-fixture'
OWNER='a57b09b1-fcf5-4aea-b25e-7f19910af934'
NAME='tsps-woodcutting-eat-before-a57b-001'
def sha(b): return hashlib.sha256(b).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--plan',required=True);p.add_argument('--selection',required=True);p.add_argument('--output',required=True);a=p.parse_args()
 planpath=Path(a.plan).resolve();selpath=Path(a.selection).resolve();output=Path(a.output).resolve()
 assert all(x.parent==ROOT for x in (planpath,selpath,output)) and not output.exists()
 assert os.environ['CCC_SESSION_ID']==OWNER
 planraw=planpath.read_bytes(); selraw=selpath.read_bytes(); plan=json.loads(planraw); sel=json.loads(selraw)
 assert plan['source']==SOURCE and plan['name']==NAME and plan['attempts']==1 and plan['timeout']==1800 and not plan.get('executionHold')
 assert sel=={'owner':OWNER,'action':'SELECT_ONCE','attempts':1,'source':SOURCE,'planSHA256':sha(planraw)}
 req=urllib.request.Request(os.environ['CCC_BASE_URL'].rstrip('/')+'/api/acp/sessions/'+OWNER+'/config',headers={'Authorization':'Bearer '+os.environ['CCC_TOKEN']})
 with urllib.request.urlopen(req,timeout=15) as r: config=json.load(r)
 opts={x['id']:x['currentValue'] for x in config['configOptions']}; assert opts['model']=='gpt-6.1-terra' and opts['reasoning_effort']=='medium'
 ref=subprocess.check_output(['git','ls-remote','https://github.com/autismoyah-hub/tsps.git',REF],text=True).split(); assert ref==[SOURCE,REF]
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPO,text=True).strip()==SOURCE
 spec=plan['expectedInputsRef']; url=f"https://raw.githubusercontent.com/{spec['repository']}/{spec['artifactCommit']}/{spec['path']}"
 with urllib.request.urlopen(url,timeout=45) as r: raw=r.read(spec['bytes']+1)
 assert len(raw)==spec['bytes'] and sha(raw)==spec['sha256']
 doc=json.loads(raw); assert doc['source']==SOURCE and doc['tests']==plan['tests'] and doc['expectedTestCounts']==plan['expectedTestCounts']
 assert len(doc['expectedInputs'])==spec['count'] and doc['expectedInputs'].keys()==doc['sourceBlobs'].keys()
 tree=subprocess.check_output(['git','ls-tree','-r','-z',SOURCE],cwd=REPO).split(b'\0'); objects={}
 for row in tree:
  if row:
   meta,n=row.split(b'\t',1); mode,kind,oid=meta.decode().split();objects[n.decode()]=(kind,oid)
 names=sorted(doc['sourceBlobs']); assert all(objects[n]==('blob',doc['sourceBlobs'][n]) for n in names)
 data=subprocess.check_output(['git','cat-file','--batch'],cwd=REPO,input=('\n'.join(doc['sourceBlobs'][n] for n in names)+'\n').encode());pos=0
 for n in names:
  end=data.index(b'\n',pos); oid,kind,size=data[pos:end].decode().split();pos=end+1; b=data[pos:pos+int(size)];pos+=int(size)+1
  assert kind=='blob' and oid==doc['sourceBlobs'][n] and sha(b)==doc['expectedInputs'][n]
 assert pos==len(data)
 for name in ('submission.json','exclusive-intent.json'): assert not (ROOT/name).exists()
 files={n:sha((ROOT/n).read_bytes()) for n in ('prestart.py','provider_guard.py','pilot.py','payload.cjs','package.json','frame_transport.py')}
 record={'owner':OWNER,'source':SOURCE,'planSHA256':sha(planraw),'selectionSHA256':sha(selraw),'manifestSHA256':sha(raw),'inputCount':len(names),'forkRef':REF,'model':opts['model'],'effort':opts['reasoning_effort'],'files':files,'createdUTC':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':'source and local intent only; provider duplicate check and independent review separate'}
 with output.open('x') as f: json.dump(record,f,indent=2)
 print(json.dumps(record))
if __name__=='__main__':main()
