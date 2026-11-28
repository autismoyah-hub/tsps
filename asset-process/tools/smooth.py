"""Test: the E114 room with every face switched from flat (render type 1) to smooth (0). Nothing else changes."""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import scene


def smooth_bytes(data):
    assert data[-2:] not in (b'\xff\xfd', b'\xff\xfe', b'\xff\xff'), 'legacy format expected'
    end = len(data) - 18
    f = data[end:]
    nv = int.from_bytes(f[0:2], 'big'); nf = int.from_bytes(f[2:4], 'big')
    render, priority, alpha, fskin = f[5], f[6], f[7], f[8]
    assert render == 1
    off = nv + nf + (nf if priority == 255 else 0) + (nf if fskin else 0)
    b = bytearray(data)
    for i in range(off, off + nf):
        assert b[i] & 2 == 0
        b[i] &= 0xFE
    return bytes(b)


if __name__ == '__main__':
    out, base = sys.argv[1], sys.argv[2]
    real = [tuple(map(int, r.split(':'))) for r in sys.argv[3].split(',') if r]
    orig = pathlib.Path.read_bytes
    def patched(self):
        d = orig(self)
        return smooth_bytes(d) if self.name.startswith('m') and self.suffix == '.bin' and self.parent.name == 'package' else d
    pathlib.Path.read_bytes = patched
    content = scene.build(out, base, real)
    pathlib.Path.read_bytes = orig
    for k, cam in enumerate(sys.argv[4:]):
        print(scene.capture(content, f'cam{k}', cam.split()))
