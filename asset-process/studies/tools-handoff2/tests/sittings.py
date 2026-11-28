import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import json
import subprocess
import sys
from html.parser import HTMLParser
from pathlib import Path
from tempfile import TemporaryDirectory
base = Path(__file__).resolve().parents[1]


class Links(HTMLParser):
    def __init__(self):
        super().__init__()
        self.paths = []
    def handle_starttag(self, tag, attrs):
        if tag == 'a':
            self.paths.append(dict(attrs)['href'])


with TemporaryDirectory(dir=base) as tmp:
    root = Path(tmp)
    for sid, title in [('2026-11-26-b','New & bright'), ('2026-11-25-a','Old sitting')]:
        spec = root / 'spec.json'
        spec.write_text(json.dumps(dict(id=sid, title=title, items=[])))
        subprocess.run([sys.executable,'-I',str(base/'make_sitting.py'),str(spec),'--root',tmp], check=True, capture_output=True)
    subprocess.run([sys.executable,'-I',str(base/'make_sitting.py'),'--root',tmp,'--index-only'],check=True,capture_output=True)
    page = (root/'index.html').read_text()
    links = Links()
    links.feed(page)
    assert links.paths == ['sittings/2026-11-26-b/index.html','sittings/2026-11-25-a/index.html']
    assert all((root / href).is_file() for href in links.paths)
    assert 'New &amp; bright' in page and 'Old sitting' in page and '2026-11-25</time>' in page
    print('Two isolated sittings, both dated titles and targets verified, newest first after rebuilding older sitting and index.')

with TemporaryDirectory(dir=base) as tmp:
    import shutil
    shared = Path(_AP.fix('<REVIEW_DIR>'))
    root = Path(tmp)
    before = (shared/'index.html').read_bytes()
    pages = sorted((shared/'sittings').glob('*/index.html'), reverse=True)
    for src in pages:
        dest = root/src.relative_to(shared)
        dest.parent.mkdir(parents=True)
        shutil.copyfile(src,dest)
    subprocess.run([sys.executable,'-I',str(base/'make_sitting.py'),'--index-only','--root',tmp],check=True,capture_output=True)
    links = Links()
    links.feed((root/'index.html').read_text(encoding='utf-8'))
    assert links.paths == [p.relative_to(shared).as_posix() for p in pages]
    assert before == (shared/'index.html').read_bytes()
    print(f'{len(pages)} existing sitting pages indexed on private copies; first {links.paths[0]}; shared index byte-identical.')
