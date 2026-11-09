"""Bind this collection-only correction to the original selected job and direct peer turn."""
import hashlib,json,subprocess,time,types
from pathlib import Path
ROOT=Path(__file__).resolve().parent
JOB='6af179c00e0338d1bb854892'
RECEIPT='769d4147791049381e2129aa755bc751ce26635537e7a72c45699f84feb4df76'
PRESTART='d88d6c2b214d062dae2591d908cfed7fafc6abaedcbf08bb389324a5adce6cf4'
PLAN='d05658b35546e6bdb6369a3e8f514b868551673d13c6fb8ebbeb50e9962e5a27'
PEER='ed3a4125-645c-4266-8b86-4e199dd5bc63'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def verify(review_path,review_sha):
 if not __debug__:raise RuntimeError('Optimized interpreter forbidden')
 review_path=Path(review_path).resolve(); assert review_path.parent==ROOT and sha(review_path)==review_sha
 assert sha(ROOT/'submission.json')==RECEIPT and sha(ROOT/'prestart-004.json')==PRESTART and sha(ROOT/'pilot.json')==PLAN
 submission=json.loads((ROOT/'submission.json').read_bytes());assert submission['job']['id']==JOB
 intent=json.loads((ROOT/'exclusive-intent.json').read_bytes());assert intent['planSHA256']==PLAN and intent['prestartSHA256']==PRESTART
 names=('correction-verify.py','job_identity-corrected.py','status-corrected.py','status-invoke-corrected.py','collector-entry-corrected.py','collect-corrected.py')
 subject={'owner':'a57b09b1-fcf5-4aea-b25e-7f19910af934','jobId':JOB,'submissionSHA256':RECEIPT,'prestartSHA256':PRESTART,'planSHA256':PLAN,'files':{n:sha(ROOT/n) for n in names}}
 raw=(ROOT/'authority.py').read_bytes();assert hashlib.sha256(raw).hexdigest()=='218404c595589d13c18658b64c13eb924d146544d1ed40fbfc221cae96002e63';m=types.ModuleType('authority');exec(compile(raw,str(ROOT/'authority.py'),'exec'),m.__dict__)
 transcript=subprocess.check_output(['cuttle','--json','sessions','read',PEER,'--turns','200']);proof=m.reviewer_receipt(review_path.read_bytes(),transcript,PEER,subject,'COLLECTION_DELTA_GO')
 capture=ROOT/('correction-peer-transcript-'+str(time.time_ns())+'.json')
 with capture.open('xb') as f:f.write(transcript);f.flush();__import__('os').fsync(f.fileno())
 return subject,proof
