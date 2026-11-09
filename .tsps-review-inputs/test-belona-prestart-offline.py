"""Synthetic review capture controls only; no Cuttle or provider calls."""
import importlib.util
import json
from pathlib import Path
import tempfile
from unittest.mock import patch

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('prestart',root/'belona-prestart.py')
m = importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
keys = ('heldPlanSHA256','selectedPlanSHA256','manifestSHA256','callerSHA256','prestartSHA256',
        'payloadSHA256','pilotSHA256','adapterSHA256','childSHA256','classifierSHA256',
        'liveSHA256','entrySHA256','launcherSHA256','sourceGateSHA256','sourceProfileSHA256','cacheInventorySHA256','sdkPythonSHA256','sdkManifestSHA256','cliSHA256','requestSHA256')
subject = {k:'a'*64 for k in keys}
subject.update(owner=m.OWNER, reviewer=m.REVIEWER, model='gpt-6.1-terra', reasoningEffort='medium', source='247ed387d6a631a00440904a00aa8e1a2e44a962', sdkVersion='2.4.0', heldPlanSHA256='e50f6ec1e341799fd48bd3e8d39897717f68d8b657afa9297e3b7a968118fabf', selectedPlanSHA256='6d739505fa4ca4aa04fb3e273b47b40814256edd4af4b14d176e3d15c30a2b16', manifestSHA256='a44a97f57d259f84a8cc2ad3adc6ce9dbd65ec8013e7def9258702c78a2aa305')
class Result:
    def __init__(self,data): self.stdout=json.dumps({'turns':data}).encode()
def rejected(fn):
    try: fn()
    except (ValueError,FileExistsError): return
    raise AssertionError('expected rejection')
with tempfile.TemporaryDirectory() as tmp:
    root=Path(tmp);cli=root/'fake-cli';cli.write_bytes(b'non-executable synthetic data');cli_sha=m.sha(cli.read_bytes());subject['cliSHA256']=cli_sha
    digest=m.sha(m.canonical_subject(subject));turn={'role':'assistant','kind':'message','text':'BELONA_PROFILE_GO '+digest,'ts':'2026-11-09T00:00:00Z'}
    receipt=root/'review.json'
    with patch.object(m.subprocess,'run',return_value=Result([turn])):
        m.capture_review(subject,cli,cli_sha,receipt)
        rejected(lambda:m.capture_review(subject,cli,cli_sha,receipt),)
    m.validate_review(receipt.read_bytes(),subject,cli_sha)
    changed=dict(subject);changed['requestSHA256']='b'*64
    rejected(lambda:m.validate_review(receipt.read_bytes(),changed,cli_sha))
    with patch.object(m.subprocess,'run',return_value=Result([turn,turn])):
        rejected(lambda:m.capture_review(subject,cli,cli_sha,root/'duplicate'))
    with patch.object(m.subprocess,'run',return_value=Result([dict(turn,role='user')])):
        rejected(lambda:m.capture_review(subject,cli,cli_sha,root/'wrong-role'))
    assert not (root/'duplicate').exists() and not (root/'wrong-role').exists()
print('synthetic capture exact-subject, create-only, changed-subject, duplicate and wrong-role controls passed; no Cuttle/provider call')

# Actual author decision is not created here: every session response below is fake.
caller_spec=importlib.util.spec_from_file_location('caller',Path(__file__).resolve().parent/'belona-caller.py')
caller=importlib.util.module_from_spec(caller_spec);caller_spec.loader.exec_module(caller)
_, selected=caller.prepare_selected((Path(__file__).resolve().parent/'belona-before-held-plan.json').read_bytes())
with tempfile.TemporaryDirectory() as tmp:
    root=Path(tmp);cli=root/'fake-cli';cli.write_bytes(b'synthetic-cli');cli_sha=m.sha(cli.read_bytes());subject['cliSHA256']=cli_sha
    digest=m.sha(m.canonical_subject(subject));review={'role':'assistant','kind':'message','text':'BELONA_PROFILE_GO '+digest,'ts':'2026-11-09T00:00:00Z'}
    choice=m.selected_choice(subject,selected)
    text='BELONA_SELECT_ONCE '+choice['subjectSHA256']+' '+choice['selectedPlanSHA256']+' '+choice['requestSHA256']
    selected_turn={'role':'assistant','kind':'message','text':text,'ts':'2026-11-09T00:00:01Z'}
    with patch.dict(m.os.environ,{'CCC_SESSION_ID':m.OWNER}):
        with patch.object(m.subprocess,'run',return_value=Result([review])): m.capture_review(subject,cli,cli_sha,root/'review')
        review_raw=(root/'review').read_bytes()
        with patch.object(m.subprocess,'run',return_value=Result([selected_turn])): m.capture_choice(subject,selected,review_raw,cli,cli_sha,root/'choice')
        choice_raw=(root/'choice').read_bytes()
        with patch.object(m.subprocess,'run',return_value=Result([selected_turn])): assert m.validate_choice_live(subject,selected,choice_raw,review_raw,cli,cli_sha)==choice
        earlier = dict(selected_turn,ts='2026-11-08T23:59:59Z')
        with patch.object(m.subprocess,'run',return_value=Result([earlier])):
            m.capture_choice(subject,selected,review_raw,cli,cli_sha,root/'earlier-choice')
            assert m.validate_choice_live(subject,selected,(root/'earlier-choice').read_bytes(),review_raw,cli,cli_sha)==choice
        for turns in ([selected_turn,selected_turn],[dict(selected_turn,role='user')],[dict(selected_turn,ts='')]):
            with patch.object(m.subprocess,'run',return_value=Result(turns)):
                rejected(lambda:m.validate_choice_live(subject,selected,choice_raw,review_raw,cli,cli_sha))
        altered=dict(subject);altered['requestSHA256']='b'*64
        with patch.object(m.subprocess,'run',return_value=Result([selected_turn])): rejected(lambda:m.validate_choice_live(altered,selected,choice_raw,review_raw,cli,cli_sha))
print('synthetic own-choice exact turn, timestamp presence, duplicate/wrong-role and changed-request controls passed; no actual selection')
class Response:
    status=200
    def __init__(self,options): self.raw=json.dumps({'ok':True,'configOptions':options}).encode()
    def __enter__(self): return self
    def __exit__(self,*args): pass
    def read(self,n): return self.raw
class Opener:
    def __init__(self,options):self.options=options
    def open(self,*args,**kwargs):return Response(self.options)
options=[{'id':'model','currentValue':'gpt-6.1-terra'},{'id':'reasoning_effort','currentValue':'medium'}]
with patch.dict(m.os.environ,{'CCC_BASE_URL':'http://localhost.invalid','CCC_TOKEN':'fake-test-only'}):
    with patch.object(m.urllib.request,'build_opener',return_value=Opener(options)):
        cfg=m.effective_config();assert cfg['model']=='gpt-6.1-terra' and cfg['reasoningEffort']=='medium'
    for bad in (options+[options[0]],[options[0],{'id':'reasoning_effort','currentValue':'high'}],options[:1]):
        with patch.object(m.urllib.request,'build_opener',return_value=Opener(bad)):rejected(m.effective_config)
print('synthetic effective-config exact model/effort and duplicate/mismatch/missing controls passed; no HTTP call')
