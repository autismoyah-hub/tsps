"""Tiny helper for the tools-gaps patches: exact, counted string replacement that keeps CRLF line endings."""
import pathlib


class Patch:
    def __init__(self, path):
        self.path = pathlib.Path(path)
        raw = self.path.read_bytes()
        self.crlf = b'\r\n' in raw
        self.s = raw.decode('utf-8').replace('\r\n', '\n')

    def rep(self, a, b, count=1):
        n = self.s.count(a)
        if n != count:
            raise SystemExit(f'{self.path.name}: expected {count} of {a[:70]!r}, found {n}')
        self.s = self.s.replace(a, b)

    def save(self):
        out = self.s.replace('\n', '\r\n') if self.crlf else self.s
        self.path.write_bytes(out.encode('utf-8'))
        print('patched', self.path)
