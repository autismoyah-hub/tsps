"""Bounded read-only HF SDK metadata inventory; no SDK import/provider I/O."""
import hashlib,json,os,pathlib,stat,time
ROOT=pathlib.Path('/tmp/zogre-hf-sdk/site-packages')
MAX_ENTRIES=10000;MAX_DEPTH=32;MAX_BYTES=128*1024*1024

def scan(rootfd):
 rows=[];seen=0;total=0
 def walk(dfd,prefix,depth):
  nonlocal seen,total
  if depth>MAX_DEPTH:raise RuntimeError('DEPTH_LIMIT')
  with os.scandir(dfd) as it:
   for entry in it:
    seen+=1
    if seen>MAX_ENTRIES:raise RuntimeError('ENTRY_LIMIT')
    name=entry.name
    if name in ('.','..') or '/' in name:raise RuntimeError('NAME_INVALID')
    rel=prefix+'/'+name if prefix else name;named=os.stat(name,dir_fd=dfd,follow_symlinks=False)
    if stat.S_ISDIR(named.st_mode):
     fd=os.open(name,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=dfd)
     try:
      before=os.fstat(fd)
      if not stat.S_ISDIR(before.st_mode) or (before.st_dev,before.st_ino)!=(named.st_dev,named.st_ino):raise RuntimeError('DIR_IDENTITY')
      rows.append({'path':rel,'kind':'directory','mode':stat.S_IMODE(before.st_mode)});walk(fd,rel,depth+1)
      after=os.fstat(fd);named_after=os.stat(name,dir_fd=dfd,follow_symlinks=False)
      if (before.st_dev,before.st_ino,before.st_mtime_ns,before.st_ctime_ns)!=(after.st_dev,after.st_ino,after.st_mtime_ns,after.st_ctime_ns) or (after.st_dev,after.st_ino)!=(named_after.st_dev,named_after.st_ino):raise RuntimeError('DIR_CHANGED')
     finally:os.close(fd)
    else:
     if not stat.S_ISREG(named.st_mode) or named.st_nlink!=1 or named.st_size>MAX_BYTES:raise RuntimeError('FILE_TYPE_OR_BOUND')
     fd=os.open(name,os.O_RDONLY|os.O_NONBLOCK|os.O_NOFOLLOW,dir_fd=dfd)
     try:
      before=os.fstat(fd)
      if not stat.S_ISREG(before.st_mode) or before.st_nlink!=1 or (before.st_dev,before.st_ino)!=(named.st_dev,named.st_ino):raise RuntimeError('FILE_IDENTITY')
      digest=hashlib.sha256();size=0
      while True:
       chunk=os.read(fd,65536)
       if not chunk:break
       size+=len(chunk);total+=len(chunk)
       if total>MAX_BYTES:raise RuntimeError('BYTE_LIMIT')
       digest.update(chunk)
      after=os.fstat(fd);named_after=os.stat(name,dir_fd=dfd,follow_symlinks=False)
      if (before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns)!=(after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns,after.st_ctime_ns) or (after.st_dev,after.st_ino)!=(named_after.st_dev,named_after.st_ino) or size!=before.st_size:raise RuntimeError('FILE_CHANGED')
      rows.append({'path':rel,'kind':'file','mode':stat.S_IMODE(before.st_mode),'bytes':size,'sha256':digest.hexdigest()})
     finally:os.close(fd)
 before=os.fstat(rootfd)
 if not stat.S_ISDIR(before.st_mode):raise RuntimeError('ROOT_TYPE')
 walk(rootfd,'',0);after=os.fstat(rootfd)
 if (before.st_dev,before.st_ino,before.st_mtime_ns,before.st_ctime_ns)!=(after.st_dev,after.st_ino,after.st_mtime_ns,after.st_ctime_ns):raise RuntimeError('ROOT_CHANGED')
 return sorted(rows,key=lambda row:row['path'])

def main():
 fd=os.open(ROOT,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
 try:
  before=os.fstat(fd);named=os.stat(ROOT,follow_symlinks=False)
  if (before.st_dev,before.st_ino)!=(named.st_dev,named.st_ino):raise RuntimeError('ROOT_IDENTITY')
  rows=scan(fd);after=os.fstat(fd);named_after=os.stat(ROOT,follow_symlinks=False)
  if (before.st_dev,before.st_ino)!=(after.st_dev,after.st_ino) or (named_after.st_dev,named_after.st_ino)!=(after.st_dev,after.st_ino):raise RuntimeError('ROOT_CHANGED')
 finally:os.close(fd)
 print(json.dumps({'schema':1,'root':str(ROOT),'utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'entries':rows,'scope':'bounded point-in-time named tree; no import, execution, immutability or ancestor proof'},sort_keys=True,separators=(',',':')))
if __name__=='__main__':main()
