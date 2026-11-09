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
        'liveSHA256','sdkPythonSHA256','sdkManifestSHA256','cliSHA256','requestSHA256')
subject = {k:'a'*64 for k in keys}
subject.update(owner=m.OWNER, reviewer=m.REVIEWER, source='247ed387d6a631a00440904a00aa8e1a2e44a962', sdkVersion='2.4.0', heldPlanSHA256='e50f6ec1e341799fd48bd3e8d39897717f68d8b657afa9297e3b7a968118fabf', selectedPlanSHA256='6d739505fa4ca4aa04fb3e273b47b40814256edd4af4b14d176e3d15c30a2b16', manifestSHA256='a44a97f57d259f84a8cc2ad3adc6ce9dbd65ec8013e7def9258702c78a2aa305')
class Result:
    def __init__(self,data): self.stdout=json.dumps({'turns':data}).encode()
def rejected(fn):
    try: fn()
    except (ValueError,FileExistsError): return
    raise AssertionError('expected rejection')
with tempfile.TemporaryDirectory() as tmp:
    root=Path(tmp);cli=root/'fake-cli';cli.write_bytes(b'non-executable synthetic data');cli_sha=m.sha(cli.read_bytes());subject['cliSHA256']=cli_sha
    digest=m.sha(m.canonical_subject(subject));turn={'role':'assistant','kind':'message','text':'BELONA_PROFILE_GO '+digest,'ts':'synthetic-time'}
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
