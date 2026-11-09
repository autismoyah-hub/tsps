#!/usr/bin/env python3
"""Extract only master bytes from a private RSProx binary; never print account fields."""
import argparse,hashlib,struct
p=argparse.ArgumentParser(); p.add_argument('binary'); p.add_argument('out'); a=p.parse_args(); b=open(a.binary,'rb').read()
if hashlib.sha256(b).hexdigest()!='97351d78ee2a2c49fc8de1a0b6460abb6825630b3175c55c4f4f89299a6154cc': raise ValueError('recording hash')
o=0
def read(fmt):
 global o
 n=struct.calcsize(fmt); x=struct.unpack_from('>'+fmt,b,o)[0]; o+=n; return x
def skipstr():
 global o
 o=b.index(0,o)+1
hv=read('i'); rev=read('i'); sub=read('i'); read('B'); read('B'); read('q'); read('H'); read('i'); read('B'); skipstr(); skipstr(); read('H'); n=read('H'); o+=n; skipstr(); n=read('i')
if (hv,rev,sub,n)!=(1,231,1,208) or o+n>len(b): raise ValueError('unexpected header')
master=b[o+3:o+n]
if hashlib.sha256(master).hexdigest()!='72c3ad092dcea8aa8d2494b208270b9c5054ef8e6de991c84fd0e026d95d7d57': raise ValueError('master hash')
with open(a.out,'xb') as f: f.write(master)
print('extracted pinned trimmed master')
