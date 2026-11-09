"""Synthetic launcher controls: no SDK, credentials, network, or GAME."""
import os,runpy,shutil,socket,tempfile
from pathlib import Path
from unittest.mock import patch
source=Path(__file__).resolve().parent
with tempfile.TemporaryDirectory() as d:
 root=Path(d); (root/'candidate').mkdir()
 for name in ['offline-recovery.py']:
  shutil.copyfile(source/name,root/name)
 shutil.copyfile(source/'collector.py',root/'candidate'/'collector.py')
 calls=[]
 def no_network(*args,**kwargs):raise AssertionError('network attempted')
 def capture(executable,argv,env):calls.append((executable,argv,env))
 with patch.object(socket,'socket',no_network),patch.object(os,'execve',capture),patch('sys.argv',[str(root/'offline-recovery.py')]),patch.dict(os.environ,{'CCC_SESSION_ID':'fixture','CCC_TOKEN':'secret-fixture'}):
  runpy.run_path(str(root/'offline-recovery.py'),run_name='__main__')
 assert len(calls)==1 and calls[0][1][-1]=='--recover-offline' and calls[0][2]=={'CCC_SESSION_ID':'fixture','PATH':'/usr/bin:/bin'}
 (root/'candidate'/'collector.py').write_text('tampered')
 with patch.object(socket,'socket',no_network),patch.object(os,'execve',capture),patch('sys.argv',[str(root/'offline-recovery.py')]):
  try:runpy.run_path(str(root/'offline-recovery.py'),run_name='__main__')
  except ValueError:pass
  else:raise AssertionError('tampered collector accepted')
 assert len(calls)==1
print('offline launcher no-network, no-auth environment, exact argv and tamper refusal passed')
