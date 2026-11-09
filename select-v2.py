"""Record the author's one explicit choice after a correlated exact peer verdict."""
import datetime,hashlib,importlib.util,json,os
from pathlib import Path
root=Path(__file__).resolve().parent
if os.environ.get('CCC_SESSION_ID')!='3f20dcbd-b1d9-4e4a-bc47-a3b02d6ee6bc':raise ValueError('owner')
spec=importlib.util.spec_from_file_location('reviewed_guard',root/'guard-v2.py');mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
review=mod.current_review(None)
record={'action':'SELECT_ONCE','owner':mod.OWNER,'source':mod.SOURCE,'planSHA256':mod.PLAN_SHA,'reviewCaptureSHA256':review['correlatedCaptureSHA256'],'createdUTC':datetime.datetime.now(datetime.timezone.utc).isoformat()}
p=root/'selection-v2.json'
fd=os.open(p,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
with os.fdopen(fd,'w',encoding='utf8') as f:json.dump(record,f,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
fd=os.open(root,os.O_RDONLY);os.fsync(fd);os.close(fd)
print(json.dumps({'selectionSHA256':hashlib.sha256(p.read_bytes()).hexdigest(),'action':'SELECT_ONCE'}))
