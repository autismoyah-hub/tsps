"""Fail-closed duplicate reconciliation with an explicitly reviewed closed BEFORE."""
from pathlib import Path
import hashlib,json
TERMINAL={'COMPLETED','ERROR','CANCELED','CANCELLED','DELETED'}
KNOWN=TERMINAL|{'SCHEDULING','RUNNING','PAUSED','PENDING'}
BEFORE='c8e46107c504610a46366bd2ef1315e68c2e2fbd'
AFTER='8b018e0871b41e5758ffba49d6763c05fdb6d92c'
def rows(api, labels):
    result={}
    for job in api.list_jobs(namespace='Autismo-Yah',labels=labels):
        ident=getattr(job,'id',None);actual=getattr(job,'labels',None);status=getattr(job,'status',None)
        stage=getattr(status,'stage',None);stage=getattr(stage,'value',stage);owner=getattr(getattr(job,'owner',None),'name',None)
        if not isinstance(ident,str) or not ident or not isinstance(actual,dict) or owner!='Autismo-Yah' or stage not in KNOWN: raise ValueError('unknown provider job identity/labels/owner/stage')
        if any(not isinstance(k,str) or not isinstance(v,str) for k,v in actual.items()) or any(actual.get(k)!=v for k,v in labels.items()): raise ValueError('malformed labels or query mismatch')
        if ident in result: raise ValueError('duplicate provider result')
        result[ident]=actual
    return result

def accepted_files(root, accepted):
    if accepted is None:return set()
    if accepted.get('issuer')!='95e96f70-1d9c-4850-9f39-1b277b7fe892' or accepted.get('verdict')!='BASELINE_ACTUAL_ACCEPTED' or accepted.get('beforeSource')!=BEFORE or accepted.get('afterSource')!=AFTER: raise ValueError('baseline acceptance identity')
    files=accepted.get('files'); job=accepted.get('jobId')
    if not isinstance(files,dict) or not isinstance(job,str) or not job:raise ValueError('baseline acceptance schema')
    names=set(files)
    if 'before/submission.intent' not in names or 'before/submission.json' not in names or len(names)!=4:raise ValueError('baseline evidence set')
    status_names=[n for n in names if n.startswith('before/status-') and n.endswith('.json')];log_names=[n for n in names if n.startswith('before/logs-') and n.endswith('.txt')]
    if len(status_names)!=1 or len(log_names)!=1 or status_names[0][len('before/status-'):-5]!=log_names[0][len('before/logs-'):-4]:raise ValueError('baseline status/log pairing')
    for name,digest in files.items():
        member=root/name
        if member.is_symlink(): raise ValueError('baseline evidence symlink')
        path=member.resolve(strict=True)
        if not path.is_relative_to(root) or hashlib.sha256(path.read_bytes()).hexdigest()!=digest:raise ValueError('baseline evidence bytes')
    intent=json.loads((root/'before/submission.intent').read_bytes());submission=json.loads((root/'before/submission.json').read_bytes());status=json.loads((root/status_names[0]).read_bytes())
    if intent.get('stage')!='SUBMISSION_PENDING' or intent.get('collector')!='260b5681-1632-4418-adb3-1e0ce7a20952' or intent.get('plan',{}).get('source')!=BEFORE:raise ValueError('baseline intent')
    if submission.get('stage')!='SUBMITTED' or submission.get('namespace')!='Autismo-Yah' or submission.get('job',{}).get('id')!=job or submission.get('plan',{}).get('source')!=BEFORE or submission.get('request')!=intent.get('request'):raise ValueError('baseline submission')
    if status.get('id')!=job or status.get('status',{}).get('stage') not in TERMINAL or status.get('command')!=submission['request'].get('command'):raise ValueError('baseline terminal identity')
    return {str((root/n).resolve()) for n in ('before/submission.intent','before/submission.json')}

def reconcile(api, source, name, case_root, accepted_before=None):
    root=Path(case_root).resolve(strict=True)
    project=rows(api,{'project':'tsps'});source_rows=rows(api,{'source':source});names=rows(api,{'name':name})
    for labels in project.values():
        if not all(isinstance(labels.get(k),str) and labels[k] for k in ('project','source','name')):raise ValueError('project job missing required labels')
    if set(source_rows)!={i for i,l in project.items() if l['source']==source} or set(names)!={i for i,l in project.items() if l['name']==name}:raise ValueError('provider query sets disagree')
    for mapping in (source_rows,names):
        for ident,labels in mapping.items():
            if project[ident]!=labels:raise ValueError('provider labels changed between queries')
    if source_rows or names:raise ValueError('matching provider job requires reconciliation')
    allowed=accepted_files(root,accepted_before) if source==AFTER else set()
    if allowed:
        previous=project.get(accepted_before['jobId'])
        if not previous or previous.get('source')!=BEFORE or previous.get('name')!='shades-logout-before-held': raise ValueError('accepted baseline absent or mismatched in provider project query')
    found=[]
    for pattern in ('**/submission.intent','**/submit.intent','**/submission.json','**/submitted.json'):
        found.extend(root.glob(pattern))
    for f in found:
        if f.is_symlink() or not f.is_file() or str(f.resolve()) not in allowed:raise ValueError('local matching/sibling intent requires reconciliation')
    if allowed and {str(f.resolve()) for f in found}!=allowed:raise ValueError('accepted baseline intent set differs')
    return {'projectCount':len(project),'sourceCount':0,'nameCount':0,'localIntents':len(found),'acceptedBefore':bool(allowed),'scope':'project/source/name and owned case tree only'}
