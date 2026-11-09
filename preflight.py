"""Verify own runtime bytes and run read-only case guards before authenticated entry."""
import argparse,hashlib,json,os,signal,stat,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent
SDK=Path('/tmp/frem-hf-sdk/site-packages')
WRAPPER=Path('/tmp/frem-hf-sdk/python-with-hf.py')
ACCESS=Path('/tmp/frem-control/hf-author-control/004/files/hf-project-access-v4.py')
MANIFEST='32d610a2fba76f609491a4aa2c33cf7431bc6007b96d8f90364e3737ce4797dd'
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def check(ok,msg):
 if not ok:raise ValueError(msg)
def inventory():
 rows=[]
 for base,dirs,files in os.walk(SDK,followlinks=False):
  for name in sorted(dirs+files):
   p=Path(base)/name; st=p.lstat(); rel=p.relative_to(SDK).as_posix(); row={'path':rel,'mode':stat.S_IMODE(st.st_mode)}
   if stat.S_ISLNK(st.st_mode):raise ValueError('sdk symlink')
   if stat.S_ISDIR(st.st_mode):row['type']='dir'
   elif stat.S_ISREG(st.st_mode):row.update(type='file',bytes=st.st_size,sha256=digest(p))
   else:raise ValueError('sdk nonfile')
   rows.append(row)
 return sorted(rows,key=lambda x:x['path'])
def main():
 p=argparse.ArgumentParser();p.add_argument('--guard-sha256',required=True);p.add_argument('--mode',choices=['submit','collect'],required=True);args=p.parse_args()
 check(sys.flags.optimize==0 and sys.flags.dont_write_bytecode and sys.flags.ignore_environment and sys.flags.no_site and Path(sys.executable).resolve()==Path('/usr/bin/python3.12').resolve(),'interpreter flags')
 for item in ['/','/usr','/usr/bin','/usr/lib','/usr/lib/python3.12',str(Path(sys.executable).resolve())]:
  st=Path(item).stat();check(st.st_uid==0 and not (st.st_mode & 0o022),'system chain ownership/mode')
 for item in [SDK,SDK.parent,ROOT]:
  st=item.stat();check(st.st_uid==os.getuid() and not (st.st_mode & 0o022),'owned tree permissions')
 check(digest(Path(sys.executable).resolve())=='c2c20b4745d447551221ec3d4e70f92c270c4609fe3df34fc52ea6dd46e92273','python')
 check(digest(WRAPPER)=='02a733f9f03eea6e748cb82c6fb8205e93596cada3d77010634df515c56fb64b','wrapper')
 check(digest(ACCESS)=='fab4bc92dc8601143ce61e910f8121774c22a0d19be6d2617bba9449f7131326','adapter')
 manifest=ROOT/'sdk-manifest.json';check(digest(manifest)==MANIFEST,'sdk manifest');check(inventory()==json.loads(manifest.read_bytes())['entries'],'sdk tree')
 target=ROOT/'guard-v2.py' if args.mode=='submit' else ROOT/'candidate/collector.py'
 check(digest(target)==args.guard_sha256,'entry pin')
 if args.mode=='submit':
  child=subprocess.Popen([sys.executable,'-E','-S','-B',str(target),'--precheck'],start_new_session=True)
  try:code=child.wait(timeout=300)
  except subprocess.TimeoutExpired:
   os.killpg(child.pid,signal.SIGKILL);child.wait();raise ValueError('precheck timed out')
  check(code==0,'precheck failed')
 command=[sys.executable,'-E','-S','-B',str(WRAPPER),str(ACCESS),'--entry',str(target),'--entry-sha256',args.guard_sha256]
 if args.mode=='submit':command+=['--','--submit-once']
 os.execve(sys.executable,command,os.environ.copy())
if __name__=='__main__':main()
