"""Case candidate: authenticated duplicate check then unchanged pilot submit. Not selected."""
import argparse, base64, datetime, hashlib, json, os, subprocess, sys, time, types
from pathlib import Path
from huggingface_hub import HfApi

ROOT = Path(__file__).resolve().parent
SOURCE = '9e20a9f3b3f84c21cc248d8eff15384a4f66b063'
NAME = 'tsps-woodcutting-eat-after-a57b-001'
OWNER = 'a57b09b1-fcf5-4aea-b25e-7f19910af934'

def digest(path): return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def main():
    if not __debug__: raise RuntimeError('Optimized interpreter forbidden')
    p=argparse.ArgumentParser()
    p.add_argument('--plan', required=True); p.add_argument('--plan-sha256', required=True)
    p.add_argument('--prestart', required=True); p.add_argument('--prestart-sha256', required=True)
    p.add_argument('--selection', required=True); p.add_argument('--selection-sha256', required=True)
    a=p.parse_args()
    planpath=Path(a.plan).resolve(); prepath=Path(a.prestart).resolve(); selpath=Path(a.selection).resolve()
    assert all(x.parent==ROOT for x in (planpath,prepath,selpath))
    assert digest(planpath)==a.plan_sha256 and digest(prepath)==a.prestart_sha256 and digest(selpath)==a.selection_sha256
    plan=json.loads(planpath.read_bytes()); pre=json.loads(prepath.read_bytes()); sel=json.loads(selpath.read_bytes())
    assert plan['source']==SOURCE and plan['name']==NAME and plan['attempts']==1 and plan['timeout']==1800 and not plan.get('executionHold')
    assert pre['source']==SOURCE and pre['planSHA256']==a.plan_sha256 and pre['selectionSHA256']==a.selection_sha256 and pre['owner']==OWNER
    age=(datetime.datetime.now(datetime.timezone.utc)-datetime.datetime.fromisoformat(pre['createdUTC'])).total_seconds(); assert 0<=age<=300
    assert sel['source']==SOURCE and sel['planSHA256']==a.plan_sha256 and sel['owner']==OWNER and sel['action']=='SELECT_ONCE' and sel['attempts']==1
    for path,sha in pre['files'].items(): assert digest(ROOT/path)==sha
    parserraw=(ROOT/'authority.py').read_bytes(); assert hashlib.sha256(parserraw).hexdigest()=='218404c595589d13c18658b64c13eb924d146544d1ed40fbfc221cae96002e63'
    parser=types.ModuleType('case_authority');exec(compile(parserraw,str(ROOT/'authority.py'),'exec'),parser.__dict__)
    subject={'owner':OWNER,'source':SOURCE,'planSHA256':a.plan_sha256,'manifestSHA256':plan['expectedInputsRef']['sha256'],'connectedDiffSHA256':'b5f60a95b4c4d1d066a87dbf255b16ea0fcf49133e1f31127ffc4d5fed8acc4c','files':pre['files'],'authAdapterSHA256':'fab4bc92dc8601143ce61e910f8121774c22a0d19be6d2617bba9449f7131326','authHelperSHA256':'00ff45d75be1eae3bc8f04995cfb8bcbe2bc5adc28c1162b843cfe88c87024f9'}
    assert pre['authoritySubject']==subject and hashlib.sha256(parser.canonical(pre['identityValue'])).hexdigest()=='4e3369be85775623307b4d51186e2fb5bfa996899ed2046b3b61aba859574cd2'
    for label,session,expected in [('peer','ed3a4125-645c-4266-8b86-4e199dd5bc63',{'reviewer':'ed3a4125-645c-4266-8b86-4e199dd5bc63','verdict':'EXACT_INPUT_PROFILE_GO','subject':subject}),('owner',OWNER,sel),('mc','eaae9ca0-2f2b-4c05-85cd-c3e70178aeac',pre['identityValue'])]:
        transcript=(ROOT.parent/'mc-unique-turns.json').read_bytes() if label=='mc' else subprocess.check_output(['cuttle','--json','sessions','read',session,'--turns','200'])
        if label=='mc': assert hashlib.sha256(transcript).hexdigest()=='9fe18cf5221f74e80ac34b17b6717ee85644a8d7b69cda72722a39b715490a3a'
        proof=parser.exact_turn(transcript,session,expected)
        assert proof['turnSHA256']==pre[label+'Proof']['turnSHA256']
    assert os.environ.get('CCC_SESSION_ID')==OWNER
    # Refresh the complete reviewed prestart from a verified source snapshot immediately before provider checks.
    prestartpath=ROOT/'prestart.py'; prestartbytes=prestartpath.read_bytes()
    assert hashlib.sha256(prestartbytes).hexdigest()==pre['files']['prestart.py']
    def refresh():
        freshpath=ROOT/('guard-prestart-'+str(time.time_ns())+'.json')
        bootstrap="import base64,hashlib,sys,types; raw=base64.b64decode(sys.argv.pop(1),validate=True); expected=sys.argv.pop(1); path=sys.argv.pop(1); assert hashlib.sha256(raw).hexdigest()==expected; sys.argv[0]=path; m=types.ModuleType('__main__'); m.__dict__.update(__file__=path,__name__='__main__',__package__='',__spec__=None,__cached__=None); sys.modules['__main__']=m; exec(compile(raw,path,'exec'),m.__dict__)"
        cmd=[sys.executable,'-I','-B','-c',bootstrap,base64.b64encode(prestartbytes).decode(),pre['files']['prestart.py'],str(prestartpath),'--plan',str(planpath),'--selection',str(selpath),'--output',str(freshpath),'--review',pre['reviewPath'],'--review-sha256',pre['reviewSHA256'],'--identity',pre['identityPath'],'--identity-sha256',pre['identitySHA256']]
        assert all(len(x.encode())+1<131072 and '\0' not in x for x in cmd)
        try:
            result=subprocess.run(cmd,capture_output=True,timeout=150)
        except subprocess.TimeoutExpired as error:
            (ROOT/(freshpath.stem+'-stdout.txt')).write_bytes(error.stdout or b'')
            (ROOT/(freshpath.stem+'-stderr.txt')).write_bytes(error.stderr or b'')
            (ROOT/(freshpath.stem+'-timeout.json')).write_text(json.dumps({'outcome':'TIMEOUT','stdoutBytes':len(error.stdout or b''),'stderrBytes':len(error.stderr or b'')}))
            raise
        (ROOT/(freshpath.stem+'-stdout.txt')).write_bytes(result.stdout)
        (ROOT/(freshpath.stem+'-stderr.txt')).write_bytes(result.stderr)
        if result.returncode!=0: raise RuntimeError('Fresh prestart failed; retained output, no submission')
        fresh=json.loads(freshpath.read_bytes())
        assert fresh['authoritySubject']==subject and fresh['selectionSHA256']==a.selection_sha256 and fresh['planSHA256']==a.plan_sha256
        for label in ('peerProof','ownerProof','mcProof'): assert fresh[label]['turnSHA256']==pre[label]['turnSHA256']
        assert fresh['forkRef']==pre['forkRef'] and fresh['connectedDifferences']==pre['connectedDifferences'] and fresh['model']==pre['model'] and fresh['effort']==pre['effort'] and fresh['files']==pre['files']
        return freshpath
    firstpath=refresh()
    assert not (ROOT/'submission.json').exists() and not (ROOT/'exclusive-intent.json').exists()
    for candidate in ROOT.parent.rglob('submission.json'):
        prior=json.loads(candidate.read_bytes()); priorplan=prior.get('plan',{})
        if priorplan.get('source')==SOURCE or priorplan.get('name')==NAME:
            raise RuntimeError('Matching local source or name: stop and reconcile')
    for candidate in ROOT.parent.rglob('exclusive-intent.json'):
        prior=json.loads(candidate.read_bytes())
        if prior.get('source')==SOURCE or prior.get('name')==NAME:
            raise RuntimeError('Matching local intent: stop and reconcile')
    api=HfApi(); assert api.whoami()['name']=='Autismo-Yah'
    count=0
    for job in api.list_jobs(namespace='Autismo-Yah'):
        count+=1
        if count>100000: raise RuntimeError('Provider scan bound exceeded: duplicate state unknown')
        labels=job.labels or {}; command=job.command
        if command is None and labels.get('source') != SOURCE:
            raise RuntimeError('Provider returned incomplete command: duplicate state unknown')
        command=command or []
        # Name may be encoded in the submitted program; no log/secret is printed.
        if labels.get('source')==SOURCE or any(NAME in str(part) for part in command):
            raise RuntimeError('Matching provider source or name: stop and reconcile')
    freshpath=refresh()
    checked=time.monotonic()
    assert digest(planpath)==a.plan_sha256 and digest(selpath)==a.selection_sha256
    assert not (ROOT/'submission.json').exists() and not (ROOT/'exclusive-intent.json').exists()
    if time.monotonic()-checked>5: raise RuntimeError('Final freshness window expired')
    record={'owner':OWNER,'source':SOURCE,'name':NAME,'checkedUTC':datetime.datetime.now(datetime.timezone.utc).isoformat(),'jobsChecked':count,'planSHA256':a.plan_sha256,'prestartSHA256':a.prestart_sha256,'firstPrestartSHA256':digest(firstpath),'freshPrestartSHA256':digest(freshpath),'freshPrestartPath':str(freshpath),'selectionSHA256':a.selection_sha256,'action':'SUBMISSION_STARTING_NO_RETRY','collector':OWNER}
    intent=ROOT/'exclusive-intent.json'
    with intent.open('x') as out:
        json.dump(record,out,indent=2); out.flush(); os.fsync(out.fileno())
    directory=os.open(ROOT,os.O_RDONLY | os.O_DIRECTORY)
    try: os.fsync(directory)
    finally: os.close(directory)
    assert json.loads(intent.read_text())==record
    # The pilot's reviewed bytes are captured once and checked before execution.
    pilotpath=ROOT/'pilot.py'; pilotbytes=pilotpath.read_bytes()
    assert hashlib.sha256(pilotbytes).hexdigest()==pre['files']['pilot.py']
    module=types.ModuleType('__main__'); module.__dict__.update(__file__=str(pilotpath),__name__='__main__',__package__='',__spec__=None,__cached__=None)
    sys.argv=[str(pilotpath),'--plan',str(planpath),'--submit']; sys.modules['__main__']=module
    exec(compile(pilotbytes,str(pilotpath),'exec'),module.__dict__)
if __name__=='__main__': main()
