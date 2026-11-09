"""HELD case preflight; no provider access or selection."""
import argparse,hashlib,json,subprocess,urllib.request
from pathlib import Path
ROOT=Path(__file__).resolve().parent
REPO=Path('/workspace/repo')
CASES={'before':('345a2fe6e568111aeb662eccef371f39151a7570','ee9377437f3cc1f919c8ee1ace58338d1a09ec6e3f69cfc2870b84f58fd8b584'),'after':('4afdd3b9dcb0187ffc629452929fbcdb918bf4cd','cd544feba243ad66454431fa1fae6d33a7721cd52d5417cbd8fa04d8c51c1212')}
PREFIX='server/plugins/skills/crafting/'
BASE='fb3df2e043b6bc0542af27822486d4d5e4e00455'  # Full SHA is bound below.
SEAMS=['server/src/main/typescript/elvarg/','server/plugins/quests/QuestRuntime.js',
 'server/plugins/quests/quests/ShadesOfMortton.Quest.js','server/tests/shades-sanctity-logout.test.cjs',
 'server/target.txt','server/package.json','server/yarn.lock','server/tsconfig.json','server/scripts/ensure-cache.ts']
def relevant(p):return any(p==x or (x.endswith('/') and p.startswith(x)) for x in SEAMS)
def sha(data):return hashlib.sha256(data).hexdigest()
def command_json(args): return json.loads(subprocess.check_output(args))
def open_overlaps(read=command_json):
 def open_list():
  prs=read(['gh','pr','list','--repo','RSPSApp/tsps','--state','open','--limit','100','--json','number,headRefOid'])
  if not isinstance(prs,list) or len(prs)>=100:raise RuntimeError('incomplete PR listing; HOLD')
  result={}
  for pr in prs:
   number,head=pr.get('number'),pr.get('headRefOid')
   if not isinstance(number,int) or number in result or not isinstance(head,str) or len(head)!=40:raise RuntimeError('invalid PR listing; HOLD')
   result[number]=head
  return result
 initial=open_list(); overlaps=[]
 for number,head in sorted(initial.items()):
  endpoint=f'repos/RSPSApp/tsps/pulls/{number}'
  def metadata():
   value=read(['gh','api',endpoint]);count=value.get('changed_files');actual=value.get('head',{}).get('sha')
   if value.get('state')!='open' or actual!=head or not isinstance(count,int) or isinstance(count,bool) or not 0<=count<=3000:raise RuntimeError('PR metadata changed or unbounded; HOLD')
   return count
  count=metadata();files=[];page=1
  while len(files)<count:
   rows=read(['gh','api',f'{endpoint}/files?per_page=100&page={page}'])
   if not isinstance(rows,list) or not rows or len(rows)>100 or len(files)+len(rows)>count:raise RuntimeError('truncated or inconsistent PR files; HOLD')
   files.extend(rows);page+=1
  if metadata()!=count:raise RuntimeError('PR count changed; HOLD')
  names=set(); old_names=set(); matched=[]
  for row in files:
   name,status=row.get('filename'),row.get('status'); previous=row.get('previous_filename')
   if not isinstance(name,str) or not name or name in names or status not in ('added','removed','modified','renamed','copied','changed','unchanged'):raise RuntimeError('invalid PR file row; HOLD')
   names.add(name)
   if status in ('renamed','copied'):
    if not isinstance(previous,str) or not previous or previous in old_names:raise RuntimeError('rename origin missing or duplicate; HOLD')
    old_names.add(previous)
   elif previous is not None:raise RuntimeError('unexpected rename origin; HOLD')
   if relevant(name) or (previous is not None and relevant(previous)):
    matched.append({'path':name,'status':status,'previous':previous})
  if matched:overlaps.append({'number':number,'head':head,'files':sorted(matched,key=lambda x:(x['path'],x['previous'] or ''))})
 if open_list()!=initial:raise RuntimeError('open PR set/head changed; HOLD')
 return overlaps
def preflight(which):
 source,mhash=CASES[which];folder=ROOT/which; raw=(folder/'manifest.json').read_bytes()
 if sha(raw)!=mhash or len(raw)!=1098685:raise RuntimeError('manifest identity')
 manifest=json.loads(raw);plan=json.loads((folder/'pilot.json').read_bytes())
 subprocess.run(['git','-C',str(REPO),'fetch','origin','main'],check=True,stdout=subprocess.DEVNULL)
 if subprocess.run(['git','-C',str(REPO),'diff','--quiet','fb3df2e043b6bc0542af27822486d4d5e4e00455','origin/main','--',*SEAMS],check=False).returncode != 0:raise RuntimeError('connected current source changed; HOLD')
 overlaps=open_overlaps()
 if manifest['source']!=source or manifest['excludedPrefix']!=PREFIX or manifest['excludedCount']!=11 or plan['source']!=source or plan['executionHold'] is not True or plan['timeout']!=1800 or plan['attempts']!=1:raise RuntimeError('profile mismatch')
 url=f'https://raw.githubusercontent.com/autismoyah-hub/tsps/115002a7f53c877bfa8fc5d67afb9495ba19545b/case-inputs/shades-logout-{which}.json'
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
 if excluded!=11 or rows!=manifest['sourceBlobs'] or set(rows)!=set(manifest['expectedInputs']) or len(rows)!=5067:raise RuntimeError('tree membership mismatch')
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
 ref='refs/heads/cuttle-shades-fb3-'+('test' if which=='before' else 'fix')+'-260b'
 observed=subprocess.check_output(['git','ls-remote','https://github.com/autismoyah-hub/tsps.git',ref],text=True).strip()
 if observed!=source+'\t'+ref:raise RuntimeError('remote ref mismatch')
 return {'source':source,'manifestSHA256':mhash,'excludedPrefix':PREFIX,'excludedCount':excluded,'includedCount':len(rows),'ref':ref,'seams':SEAMS,'openOverlap':overlaps}
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('which',choices=CASES);p.add_argument('--review-subject',action='store_true');a=p.parse_args()
 if not a.review_subject:raise RuntimeError('held preflight only')
 print(json.dumps({'stage':'HELD_PREFLIGHT','subject':preflight(a.which),'providerJobAPI':False,'runtimeSelected':False}))
