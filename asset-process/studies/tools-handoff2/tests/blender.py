import os
import subprocess
import tempfile
from pathlib import Path
base = Path(__file__).resolve().parents[1]
bash = __import__('os').environ.get('GIT_BASH', 'bash')
with tempfile.TemporaryDirectory(dir=base) as tmp:
    folder = Path(tmp)
    binary = folder/'out.bin'
    for script, options in [('good', ['--weapon']), ('bad', []), ('good', ['--collection','missing_collection'])]:
        result = subprocess.run([bash, 'blender_run.sh', f'tests/{script}.py', (folder/'out.blend').as_posix(),
                                 binary.as_posix(), *options], cwd=base, capture_output=True, text=True, errors='replace',
                                env=dict(os.environ, TEMP=tmp, TMP=tmp))
        print(script, options, 'exit', result.returncode, 'bin bytes', binary.stat().st_size if binary.exists() else 'absent')
        if script == 'bad':
            assert 'intentional build failure' in result.stdout + result.stderr
        if script == 'good' and '--weapon' in options:
            assert result.returncode == 0 and binary.stat().st_size > 0
        else:
            assert result.returncode and not binary.exists()
