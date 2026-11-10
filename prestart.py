import hashlib,json,subprocess,urllib.request
from pathlib import Path
from huggingface_hub import HfApi
r=Path(__file__).resolve().parent
sha=lambda b: hashlib.sha256(b).hexdigest()
p=json.loads((r/'pilot.json').read_text())
assert p['source']=='0f017cb140362ee2c46f0a8049a85f2555a30638' and p['attempts']==1 and p['timeout']==1800 and p['flavor']=='cpu-upgrade'
assert not p.get('executionHold') and not (r/'submission.json').exists()
assert sha((r/'payload.cjs').read_bytes())=='220ef3554ec1bf8951c22dc8e50b849feeeaaa5b0b0b06f664ea7b393cb1dde4'
assert sha((r/'pilot.py').read_bytes())=='2d7c5e44bb37f3043311962f8c05deee90e83ab6fa5ee5e3b499df238ed98200'
assert sha((r/'package.json').read_bytes())=='b8b2b1ff91bc4121bdde7c00a64243108a104d81dcb60ead22b497f9dd2652b6'
spec=p['expectedInputsRef']; url=f"https://raw.githubusercontent.com/{spec['repository']}/{spec['artifactCommit']}/{spec['path']}"; raw=urllib.request.urlopen(url,timeout=30).read(); assert len(raw)==spec['bytes'] and sha(raw)==spec['sha256']
ref=subprocess.check_output(['git','ls-remote','https://github.com/autismoyah-hub/tsps.git','refs/heads/fix/4087-fossil-multicombat'],text=True).split()[0]; assert ref==p['source']
api=HfApi(); identity=api.whoami()['name']; assert identity=='Autismo-Yah'
jobs=list(api.list_jobs(namespace=identity,labels={'project':'tsps','source':p['source']}))
assert not jobs, 'Matching source job exists; reconcile before selection'
print(json.dumps({'status':'PASS','source':ref,'planSHA256':sha((r/'pilot.json').read_bytes()),'manifestSHA256':sha(raw),'identity':identity,'matchingJobName':'none','scope':'same source/name; read-only prestart'}))
