"""Synthetic plan handoff controls; the entry fails closed before importing SDK."""
import hashlib,json,subprocess,sys,tempfile
from pathlib import Path
root=Path(__file__).resolve().parent
sha=lambda data:hashlib.sha256(data).hexdigest()
entry=root/'belona-live-entry.py';live=root/'belona-live-boundaries.py'
with tempfile.TemporaryDirectory() as tmp:
 case=Path(tmp);plan=case/'selected.json';raw=b'{"synthetic":true}\n';plan.write_bytes(raw)
 subject={'entrySHA256':sha(entry.read_bytes()),'selectedPlanSHA256':sha(raw),'liveSHA256':sha(live.read_bytes())}
 context={'plan':str(plan),'caseDir':str(case),'subject':subject,'liveSource':str(live),'mode':'submit'}
 (case/'belona-context.json').write_text(json.dumps(context))
 def run():return subprocess.run([sys.executable,str(entry),'--plan',str(plan)],capture_output=True,text=True)
 result=run();assert result.returncode!=0 and 'outer pre-auth source gate evidence missing' in result.stderr
 bootstrap='import builtins,hashlib,json,runpy,sys; subject=json.loads(sys.argv.pop(1)); entry=sys.argv.pop(1); plan=sys.argv.pop(1); builtins.__BELONA_PREAUTH_EVIDENCE__={"subjectSHA256":hashlib.sha256(json.dumps(subject,sort_keys=True,separators=(",",":")).encode()).hexdigest(),"selectedPlanSHA256":subject["selectedPlanSHA256"]}; sys.argv=[entry,"--plan",plan]; runpy.run_path(entry,run_name="__main__")'
 result=subprocess.run([sys.executable,'-B','-c',bootstrap,json.dumps(subject),str(entry),str(plan)],capture_output=True,text=True)
 assert result.returncode!=0 and 'HARD_RUNTIME_HOLD' in result.stderr
 plan.write_bytes(raw+b' ');result=run();assert result.returncode!=0 and 'selected plan changed in auth handoff' in result.stderr
print('direct entry without marker refused; synthetic marked handoff reaches hard hold; changed plan refused before SDK/provider')
