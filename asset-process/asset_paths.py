"""One place for every machine-specific path the asset process uses.

Every staged tool imports this module (a three-line bootstrap at the top of each script finds it by walking up
from the script's own folder). Paths in the tools are written as placeholders, for example
``_AP.fix(r'<ROOT>/tools/lineup.py')`` or ``_AP.fix('<OSRS_CACHE_239>')``; ``fix`` expands them.

Where a value comes from, first match wins:
  1. an environment variable of the same name (``ROOT``, ``OSRS_CACHE_239`` ...);
  2. ``config.toml`` beside this file (copy ``config.example.toml``), a flat table of ``KEY = "value"``;
  3. the default below (most keys have none: an unset key expands to ``UNSET-KEY`` so that a tool that needs it
     fails with a file-not-found naming the key, while tools that do not need it still start).

Keys (see SETUP.md for where each thing comes from):
  ROOT              the working folder. Default: this package. Tools, author pack and studies are found here;
                    build outputs (builds/NAME, render/, measure/) are written here.
  LEGACY_ROOT       optional: a working folder in the original layout (builds/research-*, night/ ...). A path
                    under ROOT that does not exist is looked up there before failing. Used to prove the staged
                    tools against an existing working copy.
  OSRS_CACHE_239    the revision-239 OSRS cache folder (main_file_cache.dat2, .idx*), the cache the tools draw from.
  OSRS_CACHE_242    the revision-242 cache. Default: TSPS_CHECKOUT/server/caches/osrs-242_2026-11-03.
  TSPS_CHECKOUT     a checkout of the TSPS game server (definitions, plugins, its 242 cache).
  RENDERER_BIN      the scene renderer, authored_scene_snapshot.exe. Default: client/bin/renderer/.
  CLIENT_SRC        the Rust client source. Default: client/rust-client in this package.
  CLIENT_BIN        folder of the client's built examples (asset_figure.exe, asset_npc.exe ...). Default: client/bin.
  NATIVE_ASSETS     the client's Python model codec folder. Default: CLIENT_SRC/tools/native-assets.
  BLENDER           blender executable (5.2 was used). Default: "blender" on PATH.
  PYTHON            Python 3.13 with Pillow and NumPy. Default: the interpreter running this module.
  HD_CACHES         folder holding r561, r601, r614 (the HD-era caches, one folder per revision).
  RS3_DIR           folder of the RS3 work (caches.json); RS3_CACHE: the build-951 cache; RS3_FLAT: its
                    flat-file export.
  REFERENCE_ARCHIVE the reference archive of real assets (six-view sheets and JSON per object, NPC, item); you
                    make it yourself from your cache; nothing of it ships.
  E114_SCENE        a base scene folder (content.txt, package/, floor.bwr) used by scene.py, measure.py,
                    roomprobe.py and seams.py as the default room.
  REVIEW_DIR        where sitting pages are written. Default: ROOT/review.
  VERDICTS_DIR      where collected sitting answers are filed. Default: ROOT/verdicts.
  FONT_DIR          TrueType fonts for boards. Default: %WINDIR%/Fonts.
  NODE, POWERSHELL  executables. Defaults: "node", "powershell".
  CARGO_BIN         folder of cargo/rustc (stable-x86_64-pc-windows-gnu). Default: ~/.cargo/bin.

``python asset_paths.py`` prints every key and its value; ``--sh`` prints them as shell assignments for
``eval`` (the staged shell scripts do this).
"""
import os
import re
import sys

PKG = os.path.dirname(os.path.abspath(__file__)).replace('\\', '/')

# working-folder layout (left) -> this package's layout (right); used to read LEGACY_ROOT
LAYOUT = [
    ('client-build/rust-client', 'client/rust-client'),
    ('builds/barrel', 'examples/barrel'),
    ('night', 'prompts'),
]
STUDY_PREFIXES = ('research-', 'tools-')

KEYS = ['ROOT', 'LEGACY_ROOT', 'OSRS_CACHE_239', 'OSRS_CACHE_242', 'TSPS_CHECKOUT', 'RENDERER_BIN', 'CLIENT_SRC',
        'CLIENT_BIN', 'NATIVE_ASSETS', 'BLENDER', 'PYTHON', 'HD_CACHES', 'RS3_DIR', 'RS3_CACHE', 'RS3_FLAT',
        'REFERENCE_ARCHIVE', 'E114_SCENE', 'REVIEW_DIR', 'VERDICTS_DIR', 'FONT_DIR', 'NODE', 'POWERSHELL',
        'CARGO_BIN']

_config = None


def _load_config():
    global _config
    if _config is None:
        _config = {}
        path = os.path.join(PKG, 'config.toml')
        if os.path.isfile(path):
            try:
                import tomllib
                with open(path, 'rb') as f:
                    _config = {k: str(v) for k, v in tomllib.load(f).items()}
            except Exception:  # a minimal reader for KEY = "value" lines
                for line in open(path, encoding='utf-8'):
                    m = re.match(r'\s*([A-Z0-9_]+)\s*=\s*"(.*)"\s*$', line)
                    if m:
                        _config[m.group(1)] = m.group(2)
        for k, v in list(_config.items()):  # a relative path in config.toml is relative to this package
            if v and ('/' in v or '\\' in v) and not os.path.isabs(v):
                _config[k] = os.path.join(PKG, v)
    return _config


def _default(key):
    if key == 'ROOT':
        return PKG
    if key == 'CLIENT_SRC':
        return PKG + '/client/rust-client'
    if key == 'CLIENT_BIN':
        return PKG + '/client/bin'
    if key == 'RENDERER_BIN':
        return PKG + '/client/bin/renderer/authored_scene_snapshot.exe'
    if key == 'NATIVE_ASSETS':
        return get('CLIENT_SRC') + '/tools/native-assets'
    if key == 'OSRS_CACHE_242':
        t = get('TSPS_CHECKOUT')
        return t + '/server/caches/osrs-242_2026-11-03'
    if key == 'PYTHON':
        return sys.executable.replace('\\', '/')
    if key == 'BLENDER':
        return 'blender'
    if key == 'REVIEW_DIR':
        return get('ROOT') + '/review'
    if key == 'VERDICTS_DIR':
        return get('ROOT') + '/verdicts'
    if key == 'FONT_DIR':
        return os.environ.get('WINDIR', os.environ.get('SystemRoot', '')).replace('\\', '/') + '/Fonts'
    if key == 'NODE':
        return 'node'
    if key == 'POWERSHELL':
        return 'powershell'
    if key == 'CARGO_BIN':
        return os.path.expanduser('~/.cargo/bin').replace('\\', '/')
    if key == 'LEGACY_ROOT':
        return ''
    return 'UNSET-' + key


def get(key):
    """The value of one key (environment, then config.toml, then default)."""
    v = os.environ.get(key)
    if v is None:
        v = _load_config().get(key)
    if v is None:
        v = _default(key)
    return v.replace('\\', '/').rstrip('/') if v else v


def _legacy_rel(rel):
    """This package's layout -> the original working-folder layout (for LEGACY_ROOT)."""
    for old, new in LAYOUT:
        if rel == new or rel.startswith(new + '/'):
            return old + rel[len(new):]
    if rel.startswith('studies/'):
        return 'builds/' + rel[len('studies/'):]
    return rel


def root_path(rel):
    """ROOT/rel; if that does not exist and LEGACY_ROOT is set and has it in the old layout, that one."""
    rel = rel.replace('\\', '/').lstrip('/')
    base = get('ROOT')
    here = base + ('/' + rel if rel else '')
    legacy = get('LEGACY_ROOT')
    if legacy and rel and not os.path.exists(here.rstrip('/')):
        old = legacy + '/' + _legacy_rel(rel)
        if os.path.exists(old.rstrip('/')):
            return old
    return here


_PLACEHOLDER = re.compile(r'<(%s)>([^\s\'"<>|*?,;@+]*)' % '|'.join(KEYS))


def fix(s):
    """Expand every <KEY> placeholder in s. <ROOT>/rest goes through root_path; other keys are replaced by value."""
    if not isinstance(s, str):
        return s

    def rep(m):
        key, rest = m.group(1), m.group(2)
        if key == 'ROOT':
            out = root_path(rest)
            if rest.endswith(('/', '\\')) and not out.endswith('/'):
                out += '/'
            return out
        return get(key) + rest
    return _PLACEHOLDER.sub(rep, s)


def exe(name, *folders):
    """An executable by file name: the first of the given folders that has it, then CLIENT_BIN."""
    for d in list(folders) + [get('CLIENT_BIN')]:
        p = os.path.join(str(d), name)
        if os.path.isfile(p):
            return p.replace('\\', '/')
    return (get('CLIENT_BIN') + '/' + name)


if __name__ == '__main__':
    sh = '--sh' in sys.argv
    for k in KEYS:
        v = get(k)
        if sh:
            print("%s='%s'; export %s" % (k, v.replace("'", "'\\''"), k))
        else:
            print('%-18s %s' % (k, v))
