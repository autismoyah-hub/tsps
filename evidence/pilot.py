"""One bounded HF qualification job; default prints the request without submitting.

Run with the project-local .tools/hf-overflow/Scripts/python.exe.
--submit submits once; --collect retains status/logs without launching work.
An ambiguous submission is never automatically retried.
"""
import argparse
import dataclasses
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def save(path, value):
    path.write_text(json.dumps(value, indent=2, default=str) + '\n', encoding='utf-8')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--plan', type=Path, default=ROOT / 'pilot.json')
    action = parser.add_mutually_exclusive_group()
    action.add_argument('--submit', action='store_true')
    action.add_argument('--collect', action='store_true')
    args = parser.parse_args()
    plan_path = args.plan.resolve()
    here = plan_path.parent
    plan = json.loads(plan_path.read_text(encoding='utf-8'))
    payload = (here / plan.get('payloadFile', 'payload.cjs')).read_text(encoding='utf-8')
    payload = payload.replace('__PLAN__', json.dumps(plan))
    receipt_path = here / 'submission.json'
    request = {key: plan[key] for key in ('name', 'image', 'flavor', 'timeout', 'attempts')}
    request.update(command=['node', '-e', payload], labels={
        'project': 'tsps', 'source': plan['source'], 'purpose': 'overflow-qualification'})
    if not args.submit and not args.collect:
        print(json.dumps({'request': request, 'plan': plan}, indent=2))
        return

    from huggingface_hub import HfApi
    api = HfApi()
    identity = api.whoami()['name']  # Fails before submission if no account is connected.
    if args.submit:
        if plan.get('executionHold'):
            raise RuntimeError('Execution hold: ' + plan['executionHold'])
        package = json.loads((ROOT / 'package.json').read_text(encoding='utf-8'))
        if identity != package['namespace']:
            raise RuntimeError('Connected account differs from authorized package owner')
        if (request['flavor'] != package['flavor'] or request['attempts'] != 1
                or not 0 < request['timeout'] <= package['maximumJobSeconds']):
            raise RuntimeError('Request exceeds the frozen compute package')
        # None explicitly means no project-imposed concurrency ceiling.
        # Provider/account limits still apply at run_job; finite policies retain their guard.
        maximum = package['maximumConcurrentJobs']
        if maximum is not None:
            active = []
            terminal = {'COMPLETED', 'ERROR', 'CANCELED', 'CANCELLED', 'DELETED'}
            for job in api.list_jobs(namespace=identity, labels={'project': 'tsps'}):
                stage = getattr(job.status.stage, 'value', job.status.stage)
                if stage not in terminal:
                    active.append(job.id)
            if len(active) >= maximum:
                raise RuntimeError(f"{maximum} project cloud jobs already active; leave this request unsubmitted")
        # Exclusive intent prevents accidental duplicate submission after a timeout/restart.
        intent = {'stage': 'SUBMISSION_PENDING', 'utc': datetime.now(timezone.utc).isoformat(),
                  'namespace': identity, 'request': request,
                  'payloadSHA256': hashlib.sha256(payload.encode()).hexdigest(), 'plan': plan}
        with receipt_path.open('x', encoding='utf-8') as stream:
            json.dump(intent, stream, indent=2)
        info = api.run_job(namespace=identity, **request)
        intent.update(stage='SUBMITTED', job=dataclasses.asdict(info))
        save(receipt_path, intent)
        print(json.dumps({'stage': 'SUBMITTED', 'namespace': identity, 'jobId': info.id}))
        return

    receipt = json.loads(receipt_path.read_text(encoding='utf-8'))
    if receipt['namespace'] != identity:
        raise RuntimeError('Connected account differs from submission owner')
    if 'job' not in receipt:
        raise RuntimeError('Ambiguous submission: inspect owned Jobs by source/name before reconciliation; do not resubmit')
    job_id = receipt['job']['id']
    info = api.inspect_job(job_id=job_id, namespace=identity)
    stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')
    save(here / f'status-{stamp}.json', dataclasses.asdict(info))
    logs = list(api.fetch_job_logs(job_id=job_id, namespace=identity, follow=False))
    (here / f'logs-{stamp}.txt').write_text('\n'.join(logs) + '\n', encoding='utf-8')
    from frame_transport import reconstruct_child_outputs
    framed = reconstruct_child_outputs(logs, here / f'decoded-{stamp}')
    print(json.dumps({'jobId': job_id, 'status': dataclasses.asdict(info.status),
                      'artifacts': str(here), 'scope': plan['scope'],
                      'framedOutputComplete': framed['complete'] if framed else None}, default=str))


if __name__ == '__main__':
    main()
