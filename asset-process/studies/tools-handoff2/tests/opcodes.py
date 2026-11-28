"""Walk every data-bearing client opcode before recolours, in cache and local mode."""
import sys
from pathlib import Path
sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from objlight import decode

def record(op, n=0):
    return bytes([op]) + bytes(n)

for local in (False, True):
    raw = bytearray()
    if local:
        raw.extend(bytes.fromhex('01 01 1234 0a 05 01 2345 03') + b'description\0' + record(200,2))
    else:
        raw.extend(bytes([1,5,3,200]))  # client treats local-only opcodes as no-ops
    raw.extend(bytes.fromhex('06 01 00003456 0a 07 01 00004567'))
    for op in (14,15,19,28,69,75,81,91,95,96): raw.extend(record(op,1))
    for op in (17,18,21,22,23,27,62,64,73,74,89,90,250): raw.extend(record(op))
    for op in (24,61,68,70,71,72,82): raw.extend(record(op,2))
    for op in (2,30,31,32,33,34): raw.extend(bytes([op])+b'Remove\0')
    for op, size in [(100,2),(101,13),(102,15)]:
        raw.extend(record(op,size)+b'condition\0')
    for op in (77,92):
        raw.extend(record(op,6 if op == 92 else 4) + bytes.fromhex('01 abcd dcba'))
    raw.extend(record(78,4) + record(79,6) + bytes.fromhex('02 aabb ccdd') + record(93,6))
    raw.extend(bytes.fromhex('f9 03 01 010203') + b'string\0' + bytes.fromhex('02 030405') +
               bytes(8) + bytes.fromhex('00 060708') + bytes(4))
    raw.extend(bytes.fromhex('29 01 0011 0022 41 0081 42 0082 43 0083 1d f6 27 f1 28 01 1234 5678 00'))
    d = decode(raw, local=local)
    assert d['scale'] == {0:129,1:130,2:131}
    assert d['recolor'] == [(0x1234,0x5678)] and (d['ambient'],d['contrast']) == (-10,-15)
    print('local' if local else 'cache', 'walk reached final recolour, scales and signed lighting')
