"""rmtree_long.py DIR...: remove folders that hold read-only files or paths past Windows' 260-character limit
(uses the \\\\?\\ long-path prefix and clears the read-only flag). Prints what is left."""
import os
import shutil
import stat
import sys


def fix(func, path, exc):
    try:
        os.chmod(path, stat.S_IWRITE)
        func(path)
    except OSError:
        pass


for d in sys.argv[1:]:
    p = os.path.abspath(d)
    long = '\\\\?\\' + p
    shutil.rmtree(long, onexc=fix)
    print(d, 'gone' if not os.path.exists(long) else 'STILL THERE')
