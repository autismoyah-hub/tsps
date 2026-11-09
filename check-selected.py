#!/usr/bin/env python3
"""Inspect public 2211 tar against an externally extracted trimmed 3027 master.
Only archives 2 and 3 are compared. Not a full cache validator.
"""
import argparse,bz2,gzip,hashlib,re,struct,tarfile,zlib
from collections import Counter
p=argparse.ArgumentParser(); p.add_argument('tar'); p.add_argument('master'); a=p.parse_args()

master=open(a.master,'rb').read()
if hashlib.sha256(master).hexdigest()!='72c3ad092dcea8aa8d2494b208270b9c5054ef8e6de991c84fd0e026d95d7d57' or len(master)!=205 or master[:5]!=bytes.fromhex('00000000c8'): raise ValueError('master hash/format')
found={}; allowed={2,3,255}
with open(a.tar,'rb') as source:
 h=hashlib.sha256()
 for chunk in iter(lambda:source.read(1048576),b''): h.update(chunk)
 if h.hexdigest()!='ec27efb9575484c249cb53351e367e747344c3def7285b0570dca1a5d1d45677': raise ValueError('tar hash')
 source.seek(0)
 with tarfile.open(fileobj=source,mode='r:gz') as t:
  for m in t:
   match=re.fullmatch(r'cache/(2|3|255)/(\d+)\.dat',m.name)
   if not match: continue
   if not m.isfile() or m.size<1 or m.size>16*1024*1024: raise ValueError('invalid selected member')
   key=(int(match[1]),int(match[2]))
   if key in found: raise ValueError('duplicate member')
   found[key]=t.extractfile(m).read()
 source.seek(0); endhash=hashlib.sha256()
 for chunk in iter(lambda:source.read(1048576),b''): endhash.update(chunk)
 if endhash.hexdigest()!=h.hexdigest(): raise ValueError('tar changed during parse')
def crc(b):return zlib.crc32(b)&0xffffffff
for i in range(25):
 want,version=struct.unpack_from('>II',master,5+i*8); row=found.get((255,i))
 if want==0:
  if row is not None: raise ValueError('zero slot unexpectedly present')
 elif row is None or crc(row)!=want: raise ValueError(f'reference mismatch {i}')
def unpack(b):
 if len(b)<9: raise ValueError('short container')
 kind=b[0]; n=int.from_bytes(b[1:5],'big'); outlen=int.from_bytes(b[5:9],'big')
 if kind not in (1,2) or n>16*1024*1024 or outlen>16*1024*1024 or len(b)!=n+9: raise ValueError('container bounds')
 decoder=bz2.BZ2Decompressor() if kind==1 else zlib.decompressobj(16+zlib.MAX_WBITS)
 data=decoder.decompress((b'BZh1' if kind==1 else b'')+b[9:],max_length=outlen+1)
 if len(data)!=outlen or not decoder.eof or decoder.unused_data: raise ValueError('container output mismatch')
 return data
for archive in (2,3):
 b=unpack(found[(255,archive)]); pos=0
 def read(n):
  global pos
  if pos+n>len(b): raise ValueError('truncated index')
  v=int.from_bytes(b[pos:pos+n],'big'); pos+=n; return v
 def smart():return read(4)&0x7fffffff if b[pos]&128 else read(2)
 if read(1)!=7: raise ValueError('expected protocol 7')
 read(4); flags=read(1)
 if flags!=4: raise ValueError('unexpected flags')
 n=smart(); ids=[]; cur=0
 for _ in range(n):
  cur+=smart()
  if ids and cur<=ids[-1]: raise ValueError('duplicate/nonmonotone group')
  ids.append(cur)
 crcs=[read(4) for _ in ids]; pos+=8*n
 for _ in ids: read(4)  # group revisions
 counts=[smart() for _ in ids]
 for count in counts:
  prior=-1; current=0
  for _ in range(count):
   current+=smart()
   if current<=prior: raise ValueError('duplicate/nonmonotone file id')
   prior=current
 if pos!=len(b): raise ValueError('index trailing data')
 actual={key[1] for key in found if key[0]==archive}
 if actual!=set(ids): raise ValueError(f'group membership mismatch {archive}')
 for group,want in zip(ids,crcs):
  data=found[(archive,group)]
  if crc(data)!=want and (len(data)<2 or crc(data[:-2])!=want): raise ValueError(f'group CRC mismatch {archive}:{group}')
 print(f'archive {archive}: {n} groups match CRCs and membership')
print('reference tables: 23 nonzero master slots match; zero slots absent')
