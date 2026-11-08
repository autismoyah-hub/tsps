"""HELD case preflight; no provider access or selection."""
import argparse,hashlib,json,subprocess,urllib.request
from pathlib import Path
ROOT=Path(__file__).resolve().parent
REPO=Path('/workspace/repo')
CASES={'before':('2deadd8ce6d0bb2d7163f5f14d1fa62ea19456cc','f5a7f863edc6107d658afcff607b4fc72b0908ed22ec7419b24434c1c7336a4f'),'after':('f97459c4185b0f0c3772b8ee014c0ea3abd63440','3b6e656078604a8130a2d6f5e9c0e10f075ee79ca5e767eae66b5dac89658cce')}
PREFIX='server/plugins/skills/crafting/'
def sha(data):return hashlib.sha256(data).hexdigest()
def preflight(which):
 source,mhash=CASES[which];folder=ROOT/which; raw=(folder/'manifest.json').read_bytes()
 subprocess.run(['git','-C',str(REPO),'fetch','origin','main'],check=True,stdout=subprocess.DEVNULL)
 if subprocess.run(['git','-C',str(REPO),'diff','--quiet','f44efb070e7f044ba1c4c67608f1260daab12127','origin/main','--','server/plugins/quests/quests/ShadesOfMortton.Quest.js'],check=False).returncode != 0:raise RuntimeError('current source changed; HOLD')
 prs=json.loads(subprocess.check_output(['gh','pr','list','--repo','RSPSApp/tsps','--state','open','--limit','100','--json','files']))
 if len(prs)>=100 or any(any(f.get('path') in ('server/plugins/quests/quests/ShadesOfMortton.Quest.js','server/tests/shades-sanctity-logout.test.cjs') for f in pr.get('files',[])) for pr in prs):raise RuntimeError('open PR overlap or incomplete listing; HOLD')
 if sha(raw)!=mhash or len(raw)!=1087345:raise RuntimeError('manifest identity')
 manifest=json.loads(raw); plan=json.loads((folder/'pilot.json').read_bytes())
 if manifest['source']!=source or manifest['excludedPrefix']!=PREFIX or manifest['excludedCount']!=11 or plan['source']!=source or plan['executionHold'] is not True or plan['timeout']!=1800 or plan['attempts']!=1:raise RuntimeError('profile mismatch')
 url=f'https://raw.githubusercontent.com/autismoyah-hub/tsps/e27054b944c8d4998715f4170b309ab64003becd/case-inputs/shades-logout-{which}.json'
 with urllib.request.urlopen(url,timeout=60) as response:
  if response.status!=200 or response.geturl()!=url:raise RuntimeError('manifest HTTP mismatch')
  remote=response.read(len(raw)+1)
 if remote!=raw:raise RuntimeError('remote manifest mismatch')
 listing=subprocess.check_output(['git','-C',str(REPO),'ls-tree','-rz','--full-tree',source,'--','server','.yarnrc.yml','package.json','scripts/run-with-env.mjs','yarn.lock'])
 rows={};excluded=0
 for record in listing.split(b'\0'):
  if not record:continue
  meta,name=record.split(b'\t',1);mode,kind,oid=meta.decode().split();name=name.decode()
  if mode not in ('100644','100755') or kind!='blob' or name in rows:raise RuntimeError('tree row')
  if name.startswith(PREFIX):excluded+=1;continue
  rows[name]=oid
 if excluded!=11 or rows!=manifest['sourceBlobs'] or set(rows)!=set(manifest['expectedInputs']) or len(rows)!=5013:raise RuntimeError('tree membership mismatch')
 proc=subprocess.Popen(['git','-C',str(REPO),'cat-file','--batch'],stdin=subprocess.PIPE,stdout=subprocess.PIPE)
 for name,oid in sorted(rows.items()):
  proc.stdin.write((oid+'\n').encode());proc.stdin.flush();header=proc.stdout.readline().decode().split()
  if len(header)!=3 or header[:2]!=[oid,'blob']:raise RuntimeError('blob header')
  remaining=int(header[2]);h=hashlib.sha256()
  while remaining:
   data=proc.stdout.read(min(65536,remaining))
   if not data:raise RuntimeError('short blob')
   h.update(data);remaining-=len(data)
  if proc.stdout.read(1)!=b'\n' or h.hexdigest()!=manifest['expectedInputs'][name]:raise RuntimeError('blob hash')
 proc.stdin.close()
 if proc.wait()!=0:raise RuntimeError('cat-file failure')
 ref='refs/heads/cuttle-shades-guarded-'+('test' if which=='before' else 'fix')+'-260b'
 observed=subprocess.check_output(['git','ls-remote','https://github.com/autismoyah-hub/tsps.git',ref],text=True).strip()
 if observed!=source+'\t'+ref:raise RuntimeError('remote ref mismatch')
 return {'source':source,'manifestSHA256':mhash,'excludedPrefix':PREFIX,'excludedCount':excluded,'includedCount':len(rows),'ref':ref}
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('which',choices=CASES);p.add_argument('--review-subject',action='store_true');a=p.parse_args()
 if not a.review_subject:raise RuntimeError('held preflight only')
 print(json.dumps({'stage':'HELD_PREFLIGHT','subject':preflight(a.which),'providerJobAPI':False,'runtimeSelected':False}))
