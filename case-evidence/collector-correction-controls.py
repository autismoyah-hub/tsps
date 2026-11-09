"""Offline exact original receipt and synthetic mutations; no provider/job call."""
import copy,hashlib,importlib.util,json,pathlib
r=pathlib.Path(__file__).resolve().parent;spec=importlib.util.spec_from_file_location('identity',r/'job_identity-corrected.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
raw=(r/'submission.json').read_bytes();assert hashlib.sha256(raw).hexdigest()=='769d4147791049381e2129aa755bc751ce26635537e7a72c45699f84feb4df76';d=json.loads(raw);plan=json.loads((r/'pilot.json').read_bytes());payload=(r/'payload.cjs').read_bytes();assert m.verify_job(d['job'],d,plan,payload)==d['job']['id'];print('original returned receipt matches corrected exact label union PASS')
for scope in ('returned','inspected'):
 for field in ('missing-name','wrong-name','extra-label','wrong-source','wrong-command','wrong-image','wrong-owner'):
  sub=copy.deepcopy(d);inspected=copy.deepcopy(d['job']);j=sub['job'] if scope=='returned' else inspected
  if field=='missing-name':del j['labels']['name']
  elif field=='wrong-name':j['labels']['name']='wrong'
  elif field=='extra-label':j['labels']['extra']='wrong'
  elif field=='wrong-source':j['labels']['source']='wrong'
  elif field=='wrong-command':j['command']=['node','wrong']
  elif field=='wrong-image':j['docker_image']='wrong'
  else:j['owner']['name']='wrong'
  try:m.verify_job(inspected,sub,plan,payload)
  except ValueError:continue
  raise AssertionError(scope+' '+field+' accepted')
print('returned/inspected label, command, image, owner mutations refused PASS')
for field in ('image','command','name'):
 sub=copy.deepcopy(d)
 if field=='image':sub['request']['image']='wrong';sub['job']['docker_image']='wrong'
 elif field=='command':sub['request']['command']=['node','wrong'];sub['job']['command']=['node','wrong']
 else:sub['request']['name']='wrong';sub['job']['labels']['name']='wrong'
 try:m.verify_job(sub['job'],sub,plan,payload)
 except ValueError:continue
 raise AssertionError('mutated request accepted '+field)
print('saved request image, command and name mutations refused PASS')
# Execute corrected peer verifier with synthetic transcript: mismatched reviewed files must stop.
import tempfile,types
from unittest.mock import patch
with tempfile.TemporaryDirectory(prefix='wc-collector-authority-') as td:
 root=pathlib.Path(td);names=('correction-verify.py','job_identity-corrected.py','status-corrected.py','status-invoke-corrected.py','collector-entry-corrected.py','collect-corrected.py','submission.json','prestart-004.json','pilot.json','exclusive-intent.json','authority.py')
 for n in names:(root/n).write_bytes((r/n).read_bytes())
 spec=importlib.util.spec_from_file_location('correction',root/'correction-verify.py');v=importlib.util.module_from_spec(spec);spec.loader.exec_module(v)
 subject={'owner':'a57b09b1-fcf5-4aea-b25e-7f19910af934','jobId':v.JOB,'submissionSHA256':v.RECEIPT,'prestartSHA256':v.PRESTART,'planSHA256':v.PLAN,'files':{n:hashlib.sha256((root/n).read_bytes()).hexdigest() for n in names[:6]}}
 receipt={'reviewer':v.PEER,'verdict':'COLLECTION_DELTA_GO','subject':subject};rp=root/'review.json';rp.write_text(json.dumps(receipt));transcript=json.dumps({'turns':[{'role':'assistant','kind':'text','ts':'2026-11-09T00:00:00Z','text':json.dumps(receipt)}]}).encode()
 with patch.object(v.subprocess,'check_output',return_value=transcript):
  actual,_=v.verify(rp,hashlib.sha256(rp.read_bytes()).hexdigest());assert actual==subject
  (root/'status-corrected.py').write_text('altered')
  try:v.verify(rp,hashlib.sha256(rp.read_bytes()).hexdigest());raise AssertionError('modified file accepted')
  except ValueError:pass
print('correction peer subject authentic-turn matching and altered file refusal PASS (synthetic transcript)')
