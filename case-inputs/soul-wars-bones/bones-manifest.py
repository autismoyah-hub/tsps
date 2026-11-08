import hashlib,json,subprocess,sys,pathlib
source=sys.argv[1]
fixed={'.yarnrc.yml','package.json','scripts/run-with-env.mjs','yarn.lock'}
raw=subprocess.check_output(['git','ls-tree','-r','-z',source]); rows={}
for line in raw.split(b'\0'):
 if not line: continue
 meta,path=line.split(b'\t',1);mode,kind,oid=meta.decode().split();path=path.decode()
 if path.startswith('server/') or path in fixed:
  if kind!='blob' or mode not in ('100644','100755'): raise ValueError('unexpected tree entry '+path)
  rows[path]=oid
if not fixed<=rows.keys(): raise ValueError('missing required root inputs')
paths=sorted(rows);data=subprocess.check_output(['git','cat-file','--batch'],input=('\n'.join(rows[p] for p in paths)+'\n').encode());offset=0;hashes={}
for path in paths:
 end=data.index(b'\n',offset);oid,kind,size=data[offset:end].decode().split();offset=end+1;size=int(size);value=data[offset:offset+size];offset+=size
 if data[offset:offset+1]!=b'\n' or oid!=rows[path] or kind!='blob':raise ValueError(path)
 offset+=1; hashes[path]=hashlib.sha256(value).hexdigest()
if offset!=len(data):raise ValueError('trailing bytes')
doc={'schema':1,'source':source,'tests':['soul-wars.test.cjs'],'expectedTestCounts':{'soul-wars.test.cjs':52},'expectedInputs':hashes,'sourceBlobs':{p:rows[p] for p in paths}}
out=pathlib.Path('/workspace/tsps-control-239f')/('bones-inputs-'+source+'.json');b=(json.dumps(doc,sort_keys=True,separators=(',',':'))+'\n').encode();out.write_bytes(b)
print(json.dumps({'source':source,'path':str(out),'count':len(paths),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}))
