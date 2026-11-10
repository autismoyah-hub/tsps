import hashlib,json,subprocess,urllib.request
from pathlib import Path
from huggingface_hub import HfApi
r=Path(__file__).resolve().parent
sha=lambda b: hashlib.sha256(b).hexdigest()
check=lambda ok,msg: None if ok else (_ for _ in ()).throw(RuntimeError(msg))
plan_bytes=(r/'pilot.json').read_bytes(); selection=(r/'selection.json').read_bytes()
check(sha(plan_bytes)=='f494acd6bfc5eacc8187adea4239a621a580dd3c9adcd0f0e84a0a4257bf716f','plan mismatch')
check(sha(selection)=='c0783cbd8f743030d4e559d21d055d378a26b08e4087877b5bcac417e22b8d1f','selection mismatch')
p=json.loads(plan_bytes)
check(p['source']=='0f017cb140362ee2c46f0a8049a85f2555a30638' and p['attempts']==1 and p['timeout']==1800 and p['flavor']=='cpu-upgrade', 'prestart check failed')
check(not p.get('executionHold') and not (r/'submission.json').exists(), 'prestart check failed')
check(sha((r/'payload.cjs').read_bytes())=='220ef3554ec1bf8951c22dc8e50b849feeeaaa5b0b0b06f664ea7b393cb1dde4', 'prestart check failed')
check(sha((r/'pilot.py').read_bytes())=='2d7c5e44bb37f3043311962f8c05deee90e83ab6fa5ee5e3b499df238ed98200', 'prestart check failed')
check(sha((r/'package.json').read_bytes())=='b8b2b1ff91bc4121bdde7c00a64243108a104d81dcb60ead22b497f9dd2652b6', 'prestart check failed')
spec=p['expectedInputsRef']; url=f"https://raw.githubusercontent.com/{spec['repository']}/{spec['artifactCommit']}/{spec['path']}"; raw=urllib.request.urlopen(url,timeout=30).read(spec['bytes']+1); check(len(raw)==spec['bytes'] and sha(raw)==spec['sha256'], 'manifest mismatch')
ref=subprocess.check_output(['git','ls-remote','https://github.com/autismoyah-hub/tsps.git','refs/heads/fix/4087-fossil-multicombat'],text=True).split()[0]; check(ref==p['source'], 'source ref mismatch')
api=HfApi(); identity=api.whoami()['name']; check(identity=='Autismo-Yah', 'identity mismatch')
jobs=list(api.list_jobs(namespace=identity,labels={'project':'tsps','source':p['source']}))
check(not jobs, 'Matching source job exists; reconcile before selection')
print(json.dumps({'status':'PASS','source':ref,'planSHA256':sha((r/'pilot.json').read_bytes()),'manifestSHA256':sha(raw),'identity':identity,'matchingJobName':'none','scope':'same source/name; read-only prestart'}))
