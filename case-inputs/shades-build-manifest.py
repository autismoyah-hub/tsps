import hashlib,json,subprocess,sys
from pathlib import Path
repo=Path('/workspace/repo'); source=sys.argv[1]; out=Path(sys.argv[2]); excluded='server/plugins/skills/crafting/'
raw=subprocess.check_output(['git','-C',str(repo),'ls-tree','-rz','--full-tree',source,'--','server','.yarnrc.yml','package.json','scripts/run-with-env.mjs','yarn.lock'])
rows={}; excluded_count=0
for record in raw.split(b'\0'):
 if not record:continue
 meta,name=record.split(b'\t',1); mode,kind,oid=meta.decode().split(); name=name.decode('utf-8')
 if name in rows:raise RuntimeError('duplicate path')
 if kind!='blob' or mode not in ('100644','100755'):raise RuntimeError('nonregular entry')
 if name.startswith(excluded):excluded_count+=1;continue
 if name.startswith('/') or any(p in ('','.','..') for p in name.split('/')):raise RuntimeError('unsafe path')
 rows[name]=(mode,oid)
if excluded_count==0 or not rows:raise RuntimeError('empty exclusion or selection')
proc=subprocess.Popen(['git','-C',str(repo),'cat-file','--batch'],stdin=subprocess.PIPE,stdout=subprocess.PIPE)
expected={}; blobs={}
for name,(mode,oid) in sorted(rows.items()):
 proc.stdin.write((oid+'\n').encode());proc.stdin.flush()
 hdr=proc.stdout.readline().decode().split()
 if len(hdr)!=3 or hdr[0]!=oid or hdr[1]!='blob':raise RuntimeError('blob header mismatch')
 size=int(hdr[2]);h=hashlib.sha256()
 while size:
  b=proc.stdout.read(min(size,65536))
  if not b:raise RuntimeError('short blob')
  h.update(b);size-=len(b)
 if proc.stdout.read(1)!=b'\n':raise RuntimeError('blob framing')
 expected[name]=h.hexdigest();blobs[name]=oid
proc.stdin.close()
if proc.wait()!=0:raise RuntimeError('cat-file failed')
doc={'schema':1,'source':source,'tests':['shades-sanctity-logout.test.cjs'],'expectedTestCounts':{'shades-sanctity-logout.test.cjs':1},'excludedPrefix':excluded,'excludedCount':excluded_count,'expectedInputs':expected,'sourceBlobs':blobs}
encoded=(json.dumps(doc,sort_keys=True,separators=(',',':'))+'\n').encode()
with out.open('xb') as f:f.write(encoded)
print(json.dumps({'source':source,'files':len(rows),'excluded':excluded_count,'bytes':len(encoded),'sha256':hashlib.sha256(encoded).hexdigest()}))
