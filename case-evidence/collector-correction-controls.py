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
