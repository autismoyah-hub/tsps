"""Held Belona provider boundaries; no CLI and no provider work at import time.

REVIEW DRAFT: HARD_RUNTIME_HOLD is deliberately unconditional. A later source
revision and independent review are required before any live execution.
"""
import dataclasses
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess

HARD_RUNTIME_HOLD = True
OWNER = '49676823-be5e-408f-be58-047e80dadbb2'
NAMESPACE = 'Autismo-Yah'

def sha(raw): return hashlib.sha256(raw).hexdigest()
def snapshot(path, digest):
    path = Path(path).resolve(strict=True); raw = path.read_bytes()
    if sha(raw) != digest: raise ValueError('snapshot hash mismatch')
    return path, raw

def module_snapshot(path, digest, name):
    path, raw = snapshot(path,digest)
    spec = importlib.util.spec_from_loader(name,loader=None)
    module = importlib.util.module_from_spec(spec); module.__file__ = str(path)
    exec(compile(raw,str(path),'exec'),module.__dict__)
    return module

def unlocked():
    if HARD_RUNTIME_HOLD: raise RuntimeError('HARD_RUNTIME_HOLD: no provider action')

def object_dict(value):
    if dataclasses.is_dataclass(value): return dataclasses.asdict(value)
    if isinstance(value,dict): return value
    raise ValueError('unknown provider object shape')

def scan_provider(api, source, name, caller):
    """A complete iterator is required; any absent identity is a hold."""
    rows = []
    for job in api.list_jobs(namespace=NAMESPACE, labels={'project':'tsps'}):
        row = object_dict(job); labels = row.get('labels')
        if not isinstance(labels,dict): raise ValueError('provider job labels missing')
        rows.append({'id':row.get('id'), 'source':labels.get('source'), 'name':row.get('name')})
    caller.reject_duplicates(rows,source,name)
    return rows

def local_records(root, caller, source, name):
    rows=[]
    for pattern in ('**/submission.json','**/belona-intent.json'):
        for path in Path(root).glob(pattern):
            record=json.loads(path.read_bytes())
            plan=record.get('plan',{})
            identifier=record.get('job',{}).get('id') or record.get('intentId')
            rows.append({'id':identifier,'source':plan.get('source'),'name':plan.get('name')})
    caller.reject_duplicates(rows,source,name)
    return rows

def validate_choice(choice, selected_raw, caller):
    plan=json.loads(selected_raw)
    expected={'action':'SELECT_ONCE','owner':OWNER,'source':plan['source'],
              'selectedPlanSHA256':sha(selected_raw),'manifestSHA256':plan['expectedInputsRef']['sha256'],
              'attempts':1,'name':plan['name']}
    if choice != expected or sha(selected_raw) not in caller.SELECTED_SHA.values():
        raise ValueError('choice identity mismatch')
    return plan

def literal_and_choice(caller,selected_raw,payload_raw,choice,subject):
    plan=validate_choice(choice,selected_raw,caller)
    request,program_sha=caller.literal_request(selected_raw,payload_raw)
    request_sha=sha(json.dumps(request,sort_keys=True,separators=(',',':')).encode())
    if subject.get('owner')!=OWNER or subject.get('source')!=plan['source'] or subject.get('selectedPlanSHA256')!=sha(selected_raw) or subject.get('manifestSHA256')!=plan['expectedInputsRef']['sha256'] or subject.get('requestSHA256')!=request_sha:
        raise ValueError('request/subject mismatch')
    return plan,request,program_sha,request_sha

def require_bindings(ctx, caller, helper, subject):
    expected = {'caller':'callerSHA256','helper':'childSHA256','payload':'payloadSHA256',
                'pilot':'pilotSHA256','adapter':'adapterSHA256','python':'sdkPythonSHA256'}
    for field,key in expected.items(): snapshot(ctx[field],subject[key])
    if Path(caller.__file__).resolve() != Path(ctx['caller']).resolve() or Path(helper.__file__).resolve() != Path(ctx['helper']).resolve():
        raise ValueError('loaded helper identity mismatch')
    if Path(ctx['plan']).resolve().parent != Path(ctx['caseDir']).resolve(): raise ValueError('plan/case directory mismatch')
    if sha(Path(ctx['plan']).read_bytes()) != subject['selectedPlanSHA256']: raise ValueError('plan hash mismatch')


def guarded_submit(api, caller, prestart_module, helper, ctx):
    """No retry; prestart, complete duplicate scans, prestart, then durable intent."""
    unlocked()
    require_bindings(ctx,caller,helper,ctx['subject'])
    selected_raw=Path(ctx['plan']).read_bytes();payload_raw=Path(ctx['payload']).read_bytes()
    choice=json.loads(Path(ctx['choice']).read_bytes()); subject=ctx['subject']
    plan,request,program_sha,request_sha=literal_and_choice(caller,selected_raw,payload_raw,choice,subject)
    if api.whoami().get('name') != NAMESPACE: raise ValueError('provider namespace mismatch')
    snapshot(ctx['prestartSource'],subject['prestartSHA256'])
    if Path(prestart_module.__file__).resolve() != Path(ctx['prestartSource']).resolve(): raise ValueError('prestart module mismatch')
    prestart_args=ctx['prestartArgs']
    if prestart_args.get('subject') != subject: raise ValueError('prestart subject mismatch')
    first=prestart_module.nonsecret_prestart(**prestart_args)
    local_records(ctx['localRoot'],caller,plan['source'],plan['name'])
    provider=scan_provider(api,plan['source'],plan['name'],caller)
    second=prestart_module.nonsecret_prestart(**prestart_args)
    if first != second: raise ValueError('prestarts differ')
    command=helper.authenticated_pilot_command(ctx['python'],ctx['adapter'],subject['adapterSHA256'],
              ctx['pilot'],subject['pilotSHA256'],ctx['plan'],ctx['sdkRoot'])
    caller.claim_intent(Path(ctx['caseDir'])/'belona-intent.json',{'intentId':'exclusive-'+sha(selected_raw),
             'owner':OWNER,'plan':plan,'requestSHA256':request_sha,'programSHA256':program_sha,
             'reviewSubject':subject,'prestart':second,'providerScan':provider,'state':'PENDING_NO_RETRY'})
    # A timeout or any error after this point remains ambiguous; the intent is never deleted.
    result=subprocess.run(command+['--submit'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=180,check=True)
    receipt_path=Path(ctx['caseDir'])/'submission.json'
    receipt=json.loads(receipt_path.read_bytes())
    if receipt.get('namespace')!=NAMESPACE or receipt.get('plan')!=plan or receipt.get('request')!=request:
        raise ValueError('submission binding mismatch; HOLD')
    job=receipt.get('job',{})
    if not job.get('id') or job.get('command')!=request['command'] or job.get('labels')!=request['labels']:
        raise ValueError('returned job mismatch; HOLD')
    return {'jobId':job['id'],'requestSHA256':request_sha,'programSHA256':program_sha,
            'pilotStdoutSHA256':sha(result.stdout),'stage':'AWAIT_INSPECTION_SAME_COLLECTOR'}

def guarded_status_collect(api,caller,prestart_module,ctx):
    """Inspect first, persist raw identity, require terminal, then preserve partial logs."""
    unlocked()
    snapshot(ctx['caller'],ctx['subject']['callerSHA256']); snapshot(ctx['payload'],ctx['subject']['payloadSHA256'])
    snapshot(ctx['prestartSource'],ctx['subject']['prestartSHA256'])
    if Path(prestart_module.__file__).resolve() != Path(ctx['prestartSource']).resolve(): raise ValueError('prestart module mismatch')
    if ctx['prestartArgs'].get('subject') != ctx['subject']: raise ValueError('prestart subject mismatch')
    prestart_module.nonsecret_prestart(**ctx['prestartArgs'])
    if Path(caller.__file__).resolve() != Path(ctx['caller']).resolve(): raise ValueError('loaded caller mismatch')
    if Path(ctx['plan']).resolve().parent != Path(ctx['caseDir']).resolve(): raise ValueError('plan/case directory mismatch')
    selected_raw=Path(ctx['plan']).read_bytes();payload_raw=Path(ctx['payload']).read_bytes()
    choice=json.loads(Path(ctx['choice']).read_bytes())
    plan,request,program_sha,request_sha=literal_and_choice(caller,selected_raw,payload_raw,choice,ctx['subject'])
    receipt=json.loads((Path(ctx['caseDir'])/'submission.json').read_bytes())
    if receipt.get('namespace')!=NAMESPACE or receipt.get('plan')!=plan or receipt.get('request')!=request:
        raise ValueError('saved request mismatch before inspection')
    job_id=receipt.get('job',{}).get('id')
    if not isinstance(job_id,str) or not job_id: raise ValueError('missing original job identity')
    if api.whoami().get('name') != NAMESPACE: raise ValueError('provider namespace mismatch')
    info=object_dict(api.inspect_job(job_id=job_id,namespace=NAMESPACE))
    raw=(json.dumps(info,sort_keys=True,default=str)+'\n').encode()
    caller.create_selected(Path(ctx['caseDir'])/('inspection-'+sha(raw)+'.json'),raw)
    caller.require_inspection(info,NAMESPACE,job_id,request)
    caller.retain_log_prefix(Path(ctx['caseDir'])/'terminal-raw.log',api.fetch_job_logs(job_id=job_id,namespace=NAMESPACE,follow=False))
    return {'jobId':job_id,'requestSHA256':request_sha,'programSHA256':program_sha,
            'inspectionSHA256':sha(raw),'stage':'TERMINAL_RAW_RETAINED_UNCLASSIFIED'}
