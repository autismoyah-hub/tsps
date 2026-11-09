"""Read-only SDK tree metadata/content inventory; never imports SDK or calls HF."""
import hashlib,json,os,pathlib,stat,time
ROOT=pathlib.Path('/tmp/zogre-hf-sdk/site-packages')
MAX_FILES=10000

def main():
 rows=[]
 for directory,dirs,files in os.walk(ROOT,topdown=True,followlinks=False):
  dirs.sort();files.sort()
  for name in list(dirs):
   p=pathlib.Path(directory)/name;st=p.lstat();rel=p.relative_to(ROOT).as_posix()
   if stat.S_ISLNK(st.st_mode):rows.append({'path':rel,'kind':'symlink','target':os.readlink(p)});dirs.remove(name)
   elif stat.S_ISDIR(st.st_mode):rows.append({'path':rel,'kind':'directory','mode':stat.S_IMODE(st.st_mode)})
   else:raise RuntimeError('UNEXPECTED_DIRECTORY_ENTRY')
  for name in files:
   p=pathlib.Path(directory)/name;before=p.lstat();rel=p.relative_to(ROOT).as_posix()
   if stat.S_ISLNK(before.st_mode):rows.append({'path':rel,'kind':'symlink','target':os.readlink(p)});continue
   if not stat.S_ISREG(before.st_mode):raise RuntimeError('NONREGULAR_FILE')
   h=hashlib.sha256();size=0
   with p.open('rb') as f:
    while True:
     chunk=f.read(65536)
     if not chunk:break
     h.update(chunk);size+=len(chunk)
    after=os.fstat(f.fileno())
   named=p.lstat()
   if (before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns)!=(after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns,after.st_ctime_ns) or (named.st_dev,named.st_ino)!=(after.st_dev,after.st_ino) or size!=before.st_size:raise RuntimeError('FILE_CHANGED')
   rows.append({'path':rel,'kind':'file','mode':stat.S_IMODE(before.st_mode),'bytes':size,'sha256':h.hexdigest()})
   if len(rows)>MAX_FILES:raise RuntimeError('TREE_LIMIT')
 rows.sort(key=lambda x:x['path'])
 print(json.dumps({'schema':1,'root':str(ROOT),'utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'entries':rows,'scope':'point-in-time read only; no import/execution, snapshot or immutability proof'},sort_keys=True,separators=(',',':')))
if __name__=='__main__':main()
