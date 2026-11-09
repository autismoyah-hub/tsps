"""Unreviewed HELD draft: only execute a selected case via reviewed 004 entry."""
import argparse, base64, dataclasses, hashlib, json, os, re, struct, subprocess, sys, time
from datetime import datetime, timezone
from pathlib import Path

OWNER='260b5681-1632-4418-adb3-1e0ce7a20952'
REVIEWER='08acbe8b-e148-4484-8af8-3d7f03acd073'
BASE='345a2fe6e568111aeb662eccef371f39151a7570'
HERE=Path(__file__).resolve().parent
REQUEST_SHA='b51d3c4d3b6a8233455e44422e6a950ce52b3c68ab79d57784b34ecb1f780a30'
PREVIEW_SHA='dba461f6540178fda8c734d8762b74908d02c48a3abf05c705e7944ade1ab72e'
PAYLOAD_SHA='0d52043babcd4543a3df4a1c29062c77d4d17fbab1db451b8282e13a1e6bde4a'
PROGRAM_SHA='98bb0eef48d34aa5fec6c54850923af6a05d9d2b3b4cc8a0c7a4b8bfdd02b544'
MANIFEST_SHA='ee9377437f3cc1f919c8ee1ace58338d1a09ec6e3f69cfc2870b84f58fd8b584'
FRAME_SHA='f53689d4f2389af93ad8f6a43fe0de94caf8dcaabde3eacbd13b1c957b129a76'
RETAIN_SHA='bdcb7f104ebfa8c26d2aa68c1dd31cf7780f12174eb92d3e98ffb928797a9071'
ENCODING="json.dumps(request,sort_keys=True,separators=(',',':'),ensure_ascii=True).encode('utf-8')"
def sha(b): return hashlib.sha256(b).hexdigest()
def canon(x): return json.dumps(x,sort_keys=True,separators=(',',':'),ensure_ascii=True).encode('utf-8')
def exact(path,digest):
    data=Path(path).read_bytes()
    if sha(data)!=digest: raise RuntimeError('Input identity mismatch')
    return data
def syncdir(path):
    fd=os.open(path,os.O_RDONLY|os.O_DIRECTORY)
    try: os.fsync(fd)
    finally: os.close(fd)
def exclusive(path,data):
    with path.open('xb') as f: f.write(data); f.flush(); os.fsync(f.fileno())
    syncdir(path.parent)
def utc(): return datetime.now(timezone.utc).isoformat()
SNAPSHOT_BOOTSTRAP = """import base64,hashlib,sys,types
snapshot=base64.b64decode(sys.argv.pop(1),validate=True)
digest=sys.argv.pop(1); filename=sys.argv.pop(1)
if hashlib.sha256(snapshot).hexdigest()!=digest:raise SystemExit(1)
sys.argv[0]=filename
module=types.ModuleType('__main__');module.__dict__.update(__file__=filename,__name__='__main__',__package__='',__spec__=None,__cached__=None)
sys.modules['__main__']=module
exec(compile(snapshot,filename,'exec'),module.__dict__)
"""
def snapshot_command(path,digest,*args):
    raw=exact(path,digest)
    return [sys.executable,'-I','-B','-c',SNAPSHOT_BOOTSTRAP,base64.b64encode(raw).decode('ascii'),digest,str(path),*map(str,args)]

def compare_job(info,request,job_id):
    d=dataclasses.asdict(info)
    if d.get('id')!=job_id or d.get('command')!=request['command'] or d.get('docker_image')!=request['image']:
        raise RuntimeError('Provider command/image/job mismatch; HOLD')
    labels=d.get('labels') or {}
    if any(labels.get(k)!=v for k,v in request['labels'].items()) or labels.get('name')!=request['name']:
        raise RuntimeError('Provider labels mismatch; HOLD')
    return d

def main():
    p=argparse.ArgumentParser()
    p.add_argument('action',choices=['submit','collect'])
    p.add_argument('--selected-plan',type=Path,required=True)
    p.add_argument('--selected-plan-sha256',required=True)
    p.add_argument('--owner-capture',type=Path,required=True)
    p.add_argument('--owner-capture-sha256',required=True)
    p.add_argument('--review-receipt',type=Path,required=True)
    p.add_argument('--review-receipt-sha256',required=True)
    p.add_argument('--executor-source-sha256',required=True)
    p.add_argument('--fit-receipt',type=Path,required=True)
    p.add_argument('--fit-receipt-sha256',required=True)
    a=p.parse_args()
    if a.selected_plan.resolve() != (HERE/'selected-plan.json').resolve(): raise RuntimeError('Selected plan must be the owned collector plan')
    selected=json.loads(exact(a.selected_plan,a.selected_plan_sha256))
    selection=selected.get('selection',{})
    if selected.get('executionHold') is not False or any(selection.get(k)!=v for k,v in {'action':'SELECT_ONCE','owner':OWNER,'executor':OWNER,'collector':OWNER,'attempts':1,'noAutomaticRetry':True,'exclusiveSameUidRoot':'/tmp/tsps-shades-260b-materialize','ownerCaptureSHA256':a.owner_capture_sha256,'requestSHA256':REQUEST_SHA}.items()):
        raise RuntimeError('Held or incorrect selected tuple; HOLD')
    exact(HERE/'shades-native-payload.cjs',PAYLOAD_SHA)
    preview=json.loads(exact(HERE/'preview.json',PREVIEW_SHA)); request=preview['request']
    expected_plan=dict(preview['plan'], executionHold=False, selection=selection)
    if selected != expected_plan: raise RuntimeError('Selected plan differs from reviewed preview; HOLD')
    if sha(canon(request))!=REQUEST_SHA or sha(request['command'][-1].encode())!=PROGRAM_SHA or len(request['command'])!=3 or any('\0' in x or len(x.encode())+1>=131072 for x in request['command']): raise RuntimeError('Request mismatch')
    if selected.get('image')!=request['image'] or selected.get('flavor')!=request['flavor'] or selected.get('timeout')!=1800 or selected.get('attempts')!=1 or selected.get('source')!=request['labels']['source'] or selected.get('name')!=request['name']:
        raise RuntimeError('Selected plan/request mismatch')
    owner_path=HERE/'owner-choice-capture.json' if a.action=='collect' else a.owner_capture
    review_path=HERE/'review-receipt.json' if a.action=='collect' else a.review_receipt
    owner_raw=exact(owner_path,a.owner_capture_sha256)
    authority_path=HERE.parent/'review-authority.py'
    authority_raw=exact(authority_path,'f4388755f7a792c27ff0cbadba4ab937af5db12b9735bc3badf5c230a644e946')
    authority={'__name__':'case_authority','__file__':str(authority_path)}
    exec(compile(authority_raw,str(authority_path),'exec'),authority)
    choice={k:v for k,v in selection.items() if k!='ownerCaptureSHA256'}
    choice_proof=authority['exact_turn'](owner_raw,OWNER,choice)
    stable=lambda proof:{k:proof[k] for k in ('session','turnUTC','turnSHA256','valueSHA256')}
    if a.action=='submit':
        live_owner=subprocess.run(['cuttle','--json','sessions','read',OWNER,'--turns','200'],capture_output=True,check=True,timeout=45).stdout
        if stable(authority['exact_turn'](live_owner,OWNER,choice))!=stable(choice_proof): raise RuntimeError('Owner turn changed; HOLD')
    review_raw=exact(review_path,a.review_receipt_sha256)
    subject={'owner':OWNER,'executor':OWNER,'collector':OWNER,'planSHA256':a.selected_plan_sha256,'selectionSHA256':sha(canon(selection)),
      'payloadSHA256':PAYLOAD_SHA,'programSHA256':sha(request['command'][-1].encode()),'argvSHA256':sha(json.dumps(request['command']).encode()),
      'requestSHA256':REQUEST_SHA,'exclusiveSameUidRoot':'/tmp/tsps-shades-260b-materialize','requestHashEncoding':ENCODING,
      'executorSourceSHA256':a.executor_source_sha256,'ownerCaptureSHA256':a.owner_capture_sha256,'ownerChoiceTurn':stable(choice_proof)}
    transcript_raw=(HERE/'direct-review-capture.json').read_bytes() if a.action=='collect' else subprocess.run(['cuttle','--json','sessions','read',REVIEWER,'--turns','200'],capture_output=True,check=True,timeout=45).stdout
    review_proof=authority['reviewer_receipt'](review_raw,transcript_raw,REVIEWER,subject,'SELECTED_DELTA_AND_LITERAL_GO')
    if a.action=='submit':
        again=subprocess.run(['cuttle','--json','sessions','read',REVIEWER,'--turns','200'],capture_output=True,check=True,timeout=45).stdout
        if stable(authority['reviewer_receipt'](review_raw,again,REVIEWER,subject,'SELECTED_DELTA_AND_LITERAL_GO'))!=stable(review_proof): raise RuntimeError('Reviewer turn changed; HOLD')
    manifest=json.loads(exact(HERE/'manifest.json',MANIFEST_SHA))
    from huggingface_hub import HfApi
    api=HfApi()
    # auth004 verified the namespace using its own authenticated whoami.
    if a.action=='collect':
        claim=json.loads((HERE/'collection-journal'/'00-claim.json').read_bytes())
        if claim.get('collector')!=OWNER or claim.get('planSHA256')!=a.selected_plan_sha256: raise RuntimeError('Collector claim mismatch')
        exclusive(HERE/'collection-journal'/'02-auth-result.json',canon({'query':'auth-whoami','stage':'PASS'}))
    submission=HERE/'submission.json'; pending=HERE/'submission.intent'
    if a.action=='collect':
        if not submission.is_file() or not pending.is_file(): raise RuntimeError('No complete owned receipt; HOLD')
        intent=json.loads(pending.read_bytes()); record=json.loads(submission.read_bytes())
        if intent.get('requestSHA256')!=REQUEST_SHA or intent.get('collector')!=OWNER or intent.get('directCaptureSHA256')!=sha(transcript_raw) or intent.get('reviewReceiptSHA256')!=sha(review_raw) or intent.get('ownerCaptureSHA256')!=sha(owner_raw) or record.get('namespace')!='Autismo-Yah' or record.get('request')!=request or record.get('plan')!=selected or record.get('directCaptureSHA256')!=sha(transcript_raw) or record.get('reviewReceiptSHA256')!=sha(review_raw) or record.get('ownerCaptureSHA256')!=sha(owner_raw):
            raise RuntimeError('Foreign or changed receipt; HOLD')
        job_id=record.get('job',{}).get('id')
        if not job_id: raise RuntimeError('Ambiguous job identity; HOLD')
        if claim.get('jobId')!=job_id: raise RuntimeError('Claim job mismatch')
        exclusive(HERE/'collection-journal'/'03-inspect-start.json',canon({'query':'inspect','jobId':job_id,'stage':'START'}))
        info=api.inspect_job(job_id=job_id,namespace='Autismo-Yah'); stamp=datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
        rawstatus=dataclasses.asdict(info); exclusive(HERE/f'status-{stamp}.json',json.dumps(rawstatus,default=str,indent=2).encode())
        exclusive(HERE/'collection-journal'/'04-inspect-result.json',canon({'query':'inspect','jobId':job_id,'statusFile':f'status-{stamp}.json'}))
        stage=getattr(info.status.stage,'value',info.status.stage)
        if str(stage).upper() not in {'COMPLETED','ERROR','CANCELED','CANCELLED'}:
            print(json.dumps({'stage':'NOT_TERMINAL','jobId':job_id})); return
        exclusive(HERE/'collection-journal'/'05-logs-start.json',canon({'query':'logs','jobId':job_id,'stage':'START'}))
        frame_path=HERE/f'sdk-fragments-{stamp}.bin'; log_path=HERE/f'logs-{stamp}.txt'
        count=0; total=0
        with frame_path.open('xb') as stream:
            stream.flush(); os.fsync(stream.fileno()); syncdir(frame_path.parent)
            for fragment in api.fetch_job_logs(job_id=job_id,namespace='Autismo-Yah',follow=False):
                if not isinstance(fragment,str): raise RuntimeError('Unexpected SDK fragment type')
                raw_fragment=fragment.encode('utf-8'); total+=len(raw_fragment)
                if len(raw_fragment)>16*1024*1024 or total>128*1024*1024: raise RuntimeError('SDK fragment bound')
                frame=struct.pack('>Q',len(raw_fragment))+raw_fragment
                if stream.write(frame)!=len(frame): raise RuntimeError('Short SDK frame write')
                stream.flush();os.fsync(stream.fileno());count+=1
        exclusive(HERE/'collection-journal'/'06-logs-result.json',canon({'query':'logs','jobId':job_id,'frameFile':frame_path.name,'fragmentCount':count,'bytes':total,'sha256':sha(frame_path.read_bytes()),'format':'u64be length followed by exact SDK string encoded UTF-8'}))
        # SDK yields SSE event data strings, not wire bytes. This explicitly derived newline-joined view is for the legacy retainer only.
        with frame_path.open('rb') as frames, log_path.open('xb') as derived:
            derived.flush();os.fsync(derived.fileno());syncdir(log_path.parent)
            for _ in range(count):
                header=frames.read(8)
                if len(header)!=8: raise RuntimeError('Truncated SDK frame header')
                length=struct.unpack('>Q',header)[0]
                if length>16*1024*1024: raise RuntimeError('SDK frame length')
                data=frames.read(length)
                if len(data)!=length: raise RuntimeError('Truncated SDK frame')
                derived.write(data);derived.write(b'\n')
            if frames.read(1): raise RuntimeError('Unexpected SDK frame tail')
            derived.flush();os.fsync(derived.fileno())
        exclusive(HERE/'collection-journal'/'07-derived-view.json',canon({'format':'SDK event UTF-8 plus newline per event; derived, not raw wire','logFile':log_path.name,'sha256':sha(log_path.read_bytes()),'sourceFrameSHA256':sha(frame_path.read_bytes())}))
        returned=record['job']
        if returned.get('command')!=request['command'] or returned.get('docker_image')!=request['image'] or (returned.get('labels') or {}).get('source')!=request['labels']['source'] or (returned.get('labels') or {}).get('name')!=request['name']:
            raise RuntimeError('Submission returned command/source mismatch; HOLD')
        compare_job(info,request,job_id)
        # Native output is retained directly, not as CLIENT frames. The exact
        # same-job offline retainer validates contiguous actual input fragments.
        retainer=HERE.parent/'retain-contiguous-inputs003.py'
        retain=subprocess.run(snapshot_command(retainer,'b399ca13e8fbfee6f7633a9d430d85c8204774c2941a2e4d5f74c642820fa690','--raw',HERE/f'logs-{stamp}.txt','--manifest',HERE/'manifest.json','--submission',submission,'--terminal-status',HERE/f'status-{stamp}.json','--out',HERE/f'retained-{stamp}'),capture_output=True,timeout=120)
        exclusive(HERE/f'retainer-{stamp}.json',canon({'code':retain.returncode,'stdoutSHA256':sha(retain.stdout),'stderrSHA256':sha(retain.stderr)}))
        if retain.returncode!=0: raise RuntimeError('Native input retention failed; raw preserved; HOLD')
        print(json.dumps({'stage':'TERMINAL_RAW_RETAINED','jobId':job_id,'statusFile':f'status-{stamp}.json','logFile':f'logs-{stamp}.txt','closure':'process-group and provider evidence only; raw requires independent classification'})); return
    # Exact case preflight checks remote refs, schema inputs and all local Git rows.
    preflight=subprocess.run(snapshot_command(HERE.parent/'case-caller.py','101232e78ab3020b7963bdd26dbeb193adfa2a96a004eb7a51e7ada8845cdac9','before','--review-subject'),capture_output=True,check=True,timeout=120)
    preflight_json=json.loads(preflight.stdout)
    if preflight_json.get('stage')!='HELD_PREFLIGHT' or preflight_json['subject']['source']!=BASE: raise RuntimeError('Preflight mismatch')
    # Accepted auth004 wrapper provides HfApi; reconcile all project/source/name queries.
    path=HERE.parent/'reconcile.py'; raw=exact(path,'b2e8d9593222dd823b2f4df3d84e43494fbda0d80b73686015d479825a78df77')
    namespace={'__name__':'case_reconcile','__file__':str(path)};exec(compile(raw,str(path),'exec'),namespace)
    reconciliation=namespace['reconcile'](api,BASE,request['name'],HERE.parent)
    exclusive(HERE/'reconciliation.json',canon({'utc':utc(),'result':reconciliation}))
    exclusive(HERE/'direct-review-capture.json',transcript_raw)
    exclusive(HERE/'owner-choice-capture.json',owner_raw)
    exclusive(HERE/'review-receipt.json',review_raw)
    # Repeat the complete exact-input and current-main/PR check immediately before intent.
    last_fit_start=time.monotonic()
    final_preflight=subprocess.run(snapshot_command(HERE.parent/'case-caller.py','101232e78ab3020b7963bdd26dbeb193adfa2a96a004eb7a51e7ada8845cdac9','before','--review-subject'),capture_output=True,check=True,timeout=120)
    if json.loads(final_preflight.stdout)!=preflight_json: raise RuntimeError('Final preflight differs; HOLD')
    main=subprocess.check_output(['git','-C','/workspace/repo','rev-parse','origin/main'],text=True).strip()
    remote=subprocess.check_output(['git','-C','/workspace/repo','ls-remote','origin','refs/heads/main'],text=True).strip().split('\t')[0]
    if remote!=main: raise RuntimeError('Main moved after preflight; HOLD')
    fit_raw=exact(a.fit_receipt,a.fit_receipt_sha256)
    fit_value=json.loads(fit_raw); dispositions=fit_value.get('overlapDisposition')
    if not isinstance(dispositions,list) or len(dispositions)!=len(preflight_json['subject']['openOverlap']): raise RuntimeError('Overlap dispositions missing')
    for item,overlap in zip(dispositions,preflight_json['subject']['openOverlap']):
        if not isinstance(item,dict) or set(item)!={'number','head','reason'} or item['number']!=overlap['number'] or item['head']!=overlap['head'] or not isinstance(item['reason'],str) or len(item['reason'].strip())<20: raise RuntimeError('Overlap disposition invalid')
    fit_expected={'reviewer':REVIEWER,'verdict':'SCOPED_WRITERS_CLEAR','owner':OWNER,'base':'fb3df2e043b6bc0542af27822486d4d5e4e00455','currentMain':main,
      'seams':preflight_json['subject']['seams'],'openOverlap':preflight_json['subject']['openOverlap'],'overlapDisposition':dispositions}
    if json.loads(fit_raw)!=fit_expected: raise RuntimeError('Scoped writer disposition mismatch')
    fit_transcript=subprocess.run(['cuttle','--json','sessions','read',REVIEWER,'--turns','200'],capture_output=True,check=True,timeout=45).stdout
    fit_proof=authority['exact_turn'](fit_transcript,REVIEWER,fit_expected)
    fit_time=datetime.fromisoformat(fit_proof['turnUTC'].replace('Z','+00:00'))
    age=(datetime.now(timezone.utc)-fit_time).total_seconds()
    clock=json.loads((HERE/'prestart-clock.json').read_bytes())
    if not 0<=age<=120 or not 0<=time.monotonic()-clock['startedMonotonic']<=120 or time.monotonic()-last_fit_start>30: raise RuntimeError('Freshness window expired; HOLD')
    exclusive(HERE/'fit-receipt.json',fit_raw)
    exclusive(HERE/'fit-capture.json',fit_transcript)
    exclusive(HERE/'fresh-prestart.json',canon({'utc':utc(),'namespace':'Autismo-Yah','base':BASE,'sourceAndAbsences':'PASS','providerReconciliation':reconciliation,'scope':'own author workspace and exclusive same-UID job root; no global lease','requestSHA256':REQUEST_SHA,'planSHA256':a.selected_plan_sha256,'reviewReceiptSHA256':a.review_receipt_sha256,'messageID':'UNKNOWN'}))
    intent={'stage':'SUBMISSION_PENDING','utc':utc(),'executor':OWNER,'collector':OWNER,'requestSHA256':REQUEST_SHA,'request':request,'plan':selected,'directCaptureSHA256':sha(transcript_raw),'reviewReceiptSHA256':sha(review_raw),'ownerCaptureSHA256':sha(owner_raw)}
    exclusive(pending,canon(intent))
    result=api.run_job(namespace='Autismo-Yah',**request)
    record={'stage':'SUBMITTED','namespace':'Autismo-Yah','request':request,'plan':selected,'job':dataclasses.asdict(result),'directCaptureSHA256':sha(transcript_raw),'reviewReceiptSHA256':sha(review_raw),'ownerCaptureSHA256':sha(owner_raw)}
    exclusive(submission,json.dumps(record,default=str,indent=2).encode())
    compare_job(result,request,result.id)
    print(json.dumps({'stage':'SUBMITTED','jobId':result.id}))
if __name__=='__main__': main()
