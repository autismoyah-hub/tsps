"""Case-specific prestart candidate. No provider action; output is not a review."""
import argparse, datetime, glob, hashlib, importlib.metadata, json, os, subprocess, sys, urllib.request
from pathlib import Path
ROOT=Path(__file__).resolve().parent
REPO=Path('/workspace/repo')
SOURCE='daff5ac899dc1bae79ab6fea6d66df15f0ef855f'
REF='refs/heads/woodcutting-inventory-interruption-fixture'
OWNER='a57b09b1-fcf5-4aea-b25e-7f19910af934'
NAME='tsps-woodcutting-eat-before-a57b-001'
def sha(b): return hashlib.sha256(b).hexdigest()
def main():
 if not __debug__: raise RuntimeError('Optimized interpreter forbidden')
 p=argparse.ArgumentParser();p.add_argument('--plan',required=True);p.add_argument('--selection',required=True);p.add_argument('--output',required=True);p.add_argument('--review',required=True);p.add_argument('--review-sha256',required=True);p.add_argument('--identity',required=True);p.add_argument('--identity-sha256',required=True);a=p.parse_args()
 planpath=Path(a.plan).resolve();selpath=Path(a.selection).resolve();output=Path(a.output).resolve()
 assert all(x.parent==ROOT for x in (planpath,selpath,output)) and not output.exists()
 assert os.environ['CCC_SESSION_ID']==OWNER
 assert sys.executable=='/usr/bin/python3' and sys.version_info[:2]==(3,12)
 sdk=ROOT.parents[1]/'sdk'; receipt=json.loads((sdk/'setup-receipt.json').read_bytes())
 assert receipt['SDK']=='2.4.0' and any(d.metadata['Name']=='huggingface_hub' and d.version=='2.4.0' for d in importlib.metadata.distributions(path=[str(sdk/'site-packages')]))
 reviewraw=Path(a.review).read_bytes(); identityraw=Path(a.identity).read_bytes()
 assert sha(reviewraw)==a.review_sha256 and sha(identityraw)==a.identity_sha256
 review=json.loads(reviewraw); identity=json.loads(identityraw)
 # These records must be authentic direct peer and MC captures, not authored assertions.
 assert review['reviewer']=='ed3a4125-645c-4266-8b86-4e199dd5bc63' and review['verdict']=='EXACT_INPUT_PROFILE_GO'
 assert review['source']==SOURCE and review['planSHA256']==sha(planpath.read_bytes())
 assert identity['session']==OWNER and identity['node']=='40fb0804-05b5-4dd8-a3ba-249574adcfa0' and identity['runningReady'] is True
 assert identity['podUID'] and identity['containerID'] and identity['pvcBound'] is True and identity['pvcUID'] and identity['cpuMilli']==500 and identity['memoryBytes']==2147483648
 planraw=planpath.read_bytes(); selraw=selpath.read_bytes(); plan=json.loads(planraw); sel=json.loads(selraw)
 assert plan['source']==SOURCE and plan['name']==NAME and plan['attempts']==1 and plan['timeout']==1800 and not plan.get('executionHold')
 assert sel=={'owner':OWNER,'action':'SELECT_ONCE','attempts':1,'source':SOURCE,'planSHA256':sha(planraw)}
 token=os.environ['CCC_TOKEN']; headers={'Authorization':'Bearer '+token} if token.startswith('ccc_pat_') else {'X-CCC-Control-Token':token}
 req=urllib.request.Request(os.environ['CCC_BASE_URL'].rstrip('/')+'/api/acp/sessions/'+OWNER+'/config',headers=headers)
 with urllib.request.urlopen(req,timeout=15) as r: config=json.load(r)
 opts={x['id']:x['currentValue'] for x in config['configOptions']}; assert opts['model']=='gpt-6.1-terra' and opts['reasoning_effort']=='medium'
 sessionid=os.environ['CODEX_SESSION_ID']; matches=glob.glob('/run/agent-state/codex/sessions/**/*'+sessionid+'.jsonl',recursive=True); assert len(matches)==1
 records=[json.loads(line) for line in Path(matches[0]).read_text().splitlines()]
 metas=[r['payload'] for r in records if r.get('type')=='session_meta']; turns=[r['payload'] for r in records if r.get('type')=='turn_context']
 assert metas and metas[-1]['session_id']==sessionid and turns and turns[-1]['model']=='gpt-6.1-terra' and turns[-1]['effort']=='medium'
 assert Path('/sys/fs/cgroup/cpu.max').read_text().strip()=='50000 100000' and Path('/sys/fs/cgroup/memory.max').read_text().strip()=='2147483648'
 ref=subprocess.check_output(['git','ls-remote','https://github.com/autismoyah-hub/tsps.git',REF],text=True).split(); assert ref==[SOURCE,REF]
 assert subprocess.check_output(['git','cat-file','-t',SOURCE],cwd=REPO,text=True).strip()=='commit'
 subprocess.run(['git','fetch','--quiet','origin','main'],cwd=REPO,check=True)
 upstream=subprocess.check_output(['git','rev-parse','origin/main'],cwd=REPO,text=True).strip()
 connected=['server/plugins/skills/Woodcutting.plugin.js','server/tests/woodcutting-emote-interruption.test.cjs','server/src/main/typescript/elvarg/game/entity/impl/player/Player.ts','server/src/main/typescript/elvarg/net/packet/impl/ItemActionPacketListener.ts','server/src/main/typescript/elvarg/net/NetworkBuilder.ts','server/plugins/items/Food.plugin.js']
 # A changed connected path requires a fresh finite review, never silent acceptance.
 differences=subprocess.check_output(['git','diff','--name-only','587d6a8923cf7ce1f329e7e96d754463534498a3',upstream,'--',*connected],cwd=REPO,text=True).splitlines()
 assert not differences, differences
 spec=plan['expectedInputsRef']; url=f"https://raw.githubusercontent.com/{spec['repository']}/{spec['artifactCommit']}/{spec['path']}"
 with urllib.request.urlopen(url,timeout=45) as r: raw=r.read(spec['bytes']+1)
 assert len(raw)==spec['bytes'] and sha(raw)==spec['sha256']
 doc=json.loads(raw); assert doc['source']==SOURCE and doc['tests']==plan['tests'] and doc['expectedTestCounts']==plan['expectedTestCounts']
 assert len(doc['expectedInputs'])==spec['count'] and doc['expectedInputs'].keys()==doc['sourceBlobs'].keys()
 tree=subprocess.check_output(['git','ls-tree','-r','-z',SOURCE],cwd=REPO).split(b'\0'); objects={}
 for row in tree:
  if row:
   meta,n=row.split(b'\t',1); mode,kind,oid=meta.decode().split();objects[n.decode()]=(mode,kind,oid)
 names=sorted(doc['sourceBlobs']); roots={'.yarnrc.yml','package.json','yarn.lock','scripts/run-with-env.mjs'}
 allowed={n for n in objects if n.startswith('server/') or n in roots}
 assert set(names)==allowed and all(objects[n][0] in ('100644','100755') and objects[n][1:]==('blob',doc['sourceBlobs'][n]) for n in names)
 data=subprocess.check_output(['git','cat-file','--batch'],cwd=REPO,input=('\n'.join(doc['sourceBlobs'][n] for n in names)+'\n').encode());pos=0
 for n in names:
  end=data.index(b'\n',pos); oid,kind,size=data[pos:end].decode().split();pos=end+1; b=data[pos:pos+int(size)];pos+=int(size)+1
  assert kind=='blob' and oid==doc['sourceBlobs'][n] and len(b)==int(size) and data[pos-1:pos]==b'\n' and sha(b)==doc['expectedInputs'][n]
 assert pos==len(data)
 for name in ('submission.json','exclusive-intent.json'): assert not (ROOT/name).exists()
 pins={'pilot.py':'2d7c5e44bb37f3043311962f8c05deee90e83ab6fa5ee5e3b499df238ed98200','payload.cjs':'220ef3554ec1bf8951c22dc8e50b849feeeaaa5b0b0b06f664ea7b393cb1dde4','package.json':'ac9cb75167ec66ac1c1f4345d65bda679a20ac1b9a85aba45dd1919dcd18ac78','frame_transport.py':'f53689d4f2389af93ad8f6a43fe0de94caf8dcaabde3eacbd13b1c957b129a76','retain-job-result.py':'bdcb7f104ebfa8c26d2aa68c1dd31cf7780f12174eb92d3e98ffb928797a9071','retain-contiguous-inputs003.py':'b399ca13e8fbfee6f7633a9d430d85c8204774c2941a2e4d5f74c642820fa690'}
 assert all(sha((ROOT/n).read_bytes())==h for n,h in pins.items())
 files={n:sha((ROOT/n).read_bytes()) for n in ('prestart.py','provider_guard.py','invoke.py','collect.py','status.py','status_invoke.py',*pins)}
 assert review['authAdapterSHA256']=='fab4bc92dc8601143ce61e910f8121774c22a0d19be6d2617bba9449f7131326' and review['authHelperSHA256']=='00ff45d75be1eae3bc8f04995cf8b8bcbe2bc5adc28c1162b843cfe88c87024f9'
 assert review['files']==files and review['manifestSHA256']==sha(raw) and review['owner']==OWNER
 record={'owner':OWNER,'source':SOURCE,'planSHA256':sha(planraw),'selectionSHA256':sha(selraw),'manifestSHA256':sha(raw),'reviewSHA256':sha(reviewraw),'identitySHA256':sha(identityraw),'inputCount':len(names),'forkRef':REF,'upstream':upstream,'connectedDifferences':differences,'sdkReceiptSHA256':sha((sdk/'setup-receipt.json').read_bytes()),'pythonSHA256':sha(Path(sys.executable).read_bytes()),'model':opts['model'],'effort':opts['reasoning_effort'],'files':files,'createdUTC':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':'source and local intent only; provider duplicate check and independent review separate'}
 with output.open('x') as f: json.dump(record,f,indent=2)
 print(json.dumps(record))
if __name__=='__main__':main()
