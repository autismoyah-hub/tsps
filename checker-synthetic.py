#!/usr/bin/env python3
"""Synthetic adaptation: replace only public-input SHA pins, test same checker logic."""
import bz2,hashlib,io,os,struct,subprocess,tarfile,tempfile
from pathlib import Path
checker=Path(__file__).with_name('check-selected.py').read_text()
def pack(data):
 raw=bz2.compress(data,compresslevel=1)[4:]; return b'\1'+struct.pack('>II',len(raw),len(data))+raw
def index(group,badcrc=False,extra=b''):
 crc=__import__('zlib').crc32(group)&0xffffffff
 if badcrc: crc^=1
 data=b'\7'+struct.pack('>I',1)+b'\4'+struct.pack('>HHI',1,0,crc)+b'\0'*8+struct.pack('>IHH',1,1,0)+extra
 return pack(data)
def scenario(kind):
 group2=b'group2'; group3=b'group3'; refs={i:b'ref'+bytes([i]) for i in range(25) if i not in (16,23)}
 refs[2]=index(group2,kind=='badcrc',b'x' if kind=='index_trailing' else b''); refs[3]=index(group3)
 if kind=='compressed_invalid': refs[2]=refs[2][:15]+bytes([refs[2][15]^255])+refs[2][16:]
 master=b'\0'+struct.pack('>I',200)+b''.join(struct.pack('>II',__import__('zlib').crc32(refs[i])&0xffffffff,1) if i in refs else b'\0'*8 for i in range(25))
 members=[(f'cache/255/{i}.dat',b) for i,b in refs.items()]+[('cache/2/0.dat',group2),('cache/3/0.dat',group3)]
 if kind=='missing_group': members=[x for x in members if x[0]!='cache/2/0.dat']
 if kind=='duplicate_group': members.append(('cache/2/0.dat',group2))
 with tempfile.TemporaryDirectory() as d:
  tar=Path(d)/'case.tar.gz'; masterfile=Path(d)/'master'; source=Path(d)/'checker.py'; masterfile.write_bytes(master)
  with tarfile.open(tar,'w:gz') as t:
   for name,data in members:
    info=tarfile.TarInfo(name); info.size=len(data); t.addfile(info,io.BytesIO(data))
  code=checker.replace('72c3ad092dcea8aa8d2494b208270b9c5054ef8e6de991c84fd0e026d95d7d57',hashlib.sha256(master).hexdigest()).replace('ec27efb9575484c249cb53351e367e747344c3def7285b0570dca1a5d1d45677',hashlib.sha256(tar.read_bytes()).hexdigest())
  source.write_text(code); run=subprocess.run(['python3','-O',str(source),str(tar),str(masterfile)],capture_output=True,text=True)
  return run.returncode,run.stderr
for case in ('valid','missing_group','duplicate_group','badcrc','index_trailing','compressed_invalid'):
 code,err=scenario(case); expected=0 if case=='valid' else 1
 if code!=expected: raise RuntimeError(f'{case} expected {expected} got {code}: {err}')
 print(case,'pass' if case=='valid' else 'refused')
