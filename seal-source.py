#!/usr/bin/env python3
"""Check tracked regular files and modes; extra/generated output is reported separately."""
import hashlib,json,os,pathlib,stat,sys
p=pathlib.Path(__file__).resolve().parent
m=(p/'composed-source-manifest.json').read_bytes()
if hashlib.sha256(m).hexdigest()!='3c75cf3a085bbe5ea4e5c2c955e22cd92de4e52e2db037c5a6d909d6857f56ac':raise ValueError('manifest pin')
rows=json.loads(m)['files']; root=pathlib.Path(sys.argv[1]); errors=[]
if not root.is_absolute() or root.is_symlink() or not root.is_dir():raise ValueError('invalid root')
for name,row in rows.items():
 path=root
 for part in name.split('/'):
  path=path/part
  try: st=path.lstat()
  except FileNotFoundError:errors.append('missing '+name);break
  if stat.S_ISLNK(st.st_mode):errors.append('symlink '+name);break
 else:
  if not stat.S_ISREG(st.st_mode) or st.st_nlink!=1:errors.append('type/link '+name);continue
  mode=0o755 if row['mode']=='100755' else 0o644
  if stat.S_IMODE(st.st_mode)!=mode or st.st_size!=row['bytes'] or hashlib.sha256(path.read_bytes()).hexdigest()!=row['sha256']:errors.append('mismatch '+name)
print(json.dumps({'event':'tracked-source-seal','checked':len(rows),'errors':errors,'pass':not errors,'limitation':'trusted exclusive stable namespace required; tracked files only; generated outputs, extras and process closure require separate inventory'}))
if errors:sys.exit(1)
