import hashlib,json,runpy,sys
from pathlib import Path
r=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
check=lambda ok: None if ok else (_ for _ in ()).throw(RuntimeError('binding mismatch'))
check(sha((r/'pilot.json').read_bytes())=='f494acd6bfc5eacc8187adea4239a621a580dd3c9adcd0f0e84a0a4257bf716f')
check(sha((r/'selection.json').read_bytes())=='c0783cbd8f743030d4e559d21d055d378a26b08e4087877b5bcac417e22b8d1f')
check(sha((r/'prestart.py').read_bytes())=='ffbef97dbeb62adc36db105d78fcdf354b0aa9c4b80a38f3d9b4a9a703ffeaed')
check(not (r/'submission.json').exists())
runpy.run_path(str(r/'prestart.py'),run_name='__main__')
check(sha((r/'pilot.json').read_bytes())=='f494acd6bfc5eacc8187adea4239a621a580dd3c9adcd0f0e84a0a4257bf716f')
check(sha((r/'pilot.py').read_bytes())=='2d7c5e44bb37f3043311962f8c05deee90e83ab6fa5ee5e3b499df238ed98200')
sys.argv=[str(r/'pilot.py'),'--plan',str(r/'pilot.json'),'--submit']
runpy.run_path(str(r/'pilot.py'),run_name='__main__')
