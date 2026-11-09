"""Held authenticated entry draft; only loaded by the reviewed outer launcher."""
import argparse
import hashlib
import json
from pathlib import Path
import types

def sha(raw):return hashlib.sha256(raw).hexdigest()
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--plan',required=True,type=Path);args=parser.parse_args()
    plan_path=args.plan.resolve(strict=True);case=plan_path.parent
    ctx=json.loads((case/'belona-context.json').read_bytes());subject=ctx['subject']
    if Path(ctx['plan']).resolve(strict=True)!=plan_path or Path(ctx['caseDir']).resolve(strict=True)!=case:
        raise ValueError('context plan/case mismatch')
    if sha(Path(__file__).read_bytes())!=subject['entrySHA256']:raise ValueError('entry identity mismatch')
    if sha(plan_path.read_bytes())!=subject['selectedPlanSHA256']:raise ValueError('selected plan changed in auth handoff')
    live_path=Path(ctx['liveSource']).resolve(strict=True);raw=live_path.read_bytes()
    if sha(raw)!=subject['liveSHA256']:raise ValueError('live source mismatch')
    live=types.ModuleType('belona_verified_live');live.__file__=str(live_path);live.__dict__['__BELONA_SNAPSHOT_SHA__']=sha(raw)
    exec(compile(raw,str(live_path),'exec'),live.__dict__)
    if live.HARD_RUNTIME_HOLD:raise RuntimeError('HARD_RUNTIME_HOLD: no provider action')
    from huggingface_hub import HfApi
    if ctx.get('mode')=='submit': result=live.guarded_submit(HfApi(),ctx)
    elif ctx.get('mode')=='collect': result=live.guarded_status_collect(HfApi(),ctx)
    else:raise ValueError('unsupported case mode')
    print(json.dumps(result,sort_keys=True))
if __name__=='__main__':main()
