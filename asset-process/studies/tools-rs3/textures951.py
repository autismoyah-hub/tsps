"""Build 951 texture archives: 52 = DDS (block compressed), 53 = PNG, 54 = PNG with an extra byte (partial export).

Container payloads (after JS5 decompression):
  52, 53: u8 1, u32 BE length, image bytes (DDS / PNG)
  54:     u8 1, u8 k, u32 BE length, PNG bytes
Decoding uses Pillow's DDS and PNG readers on the bytes only; nothing is executed.
"""
import io, sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
sys.dont_write_bytecode = True
from PIL import Image
from rs3flat import Cache

def payload(cache, a, t):
    d = cache.group(a, t)
    if a == 54:
        if d[0] != 1: raise ValueError(f'54 lead byte {d[0]}')
        n = int.from_bytes(d[2:6], 'big'); body = d[6:6+n]; extra = d[1]
    else:
        # u8 image count (1, or 6 for a cube set), then per image: u32 BE length, bytes. First image returned.
        count = d[0]
        if count not in (1, 6): raise ValueError(f'{a} lead byte {d[0]}')
        q = 1; images = []
        for _ in range(count):
            n = int.from_bytes(d[q:q+4], 'big'); images.append(d[q+4:q+4+n]); q += 4+n
            if len(images[-1]) != n: raise ValueError('short image')
        return images[0], count, len(d)-q
    if len(body) != n: raise ValueError('short image')
    rest = len(d) - 6 - n
    return body, extra, rest

def image(cache, a, t):
    body, extra, rest = payload(cache, a, t)
    magic = body[:4]
    if a == 52 and magic != b'DDS ': raise ValueError(f'52 magic {magic!r}')
    if a in (53, 54) and magic != b'\x89PNG': raise ValueError(f'{a} magic {magic!r}')
    im = Image.open(io.BytesIO(body))
    im.load()
    info = dict(archive=a, id=t, size=im.size, mode=im.mode, trailing=rest, extra=extra)
    if a == 52:
        info['fourcc'] = body[84:88].decode('latin1')
        info['mipmaps'] = int.from_bytes(body[28:32], 'little')
    return im, info

if __name__ == '__main__':
    c = Cache()
    a, t, out = int(sys.argv[1]), int(sys.argv[2]), sys.argv[3]
    im, info = image(c, a, t)
    im.save(out)
    print(info)
