"""Synthetic controls only; never creates real choices or contacts a provider."""
import importlib.util
import json
from pathlib import Path
import tempfile

ROOT = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('caller', ROOT / 'belona-caller.py')
caller = importlib.util.module_from_spec(spec)
spec.loader.exec_module(caller)

def rejected(fn, error=ValueError):
    try: fn()
    except error: return
    raise AssertionError('expected refusal')

held = (ROOT / 'before-held-plan.json').read_bytes()
side, selected = caller.prepare_selected(held)
assert side == 'before'
a, b = json.loads(held), json.loads(selected)
assert set(a) - set(b) == {'executionHold'} and all(a[k] == b[k] for k in b)
for key, value in [('source','bad'), ('name','bad'), ('attempts',2), ('expectedInputsRef',{})]:
    changed = dict(a); changed[key] = value
    rejected(lambda: caller.prepare_selected(json.dumps(changed).encode()))
print('a: held transform exact; mutated held bytes refused')

payload = (ROOT / 'native-payload-mapref-candidate.cjs').read_bytes()
request, digest = caller.literal_request(selected, payload)
assert len(digest) == 64
for key in ('image','command'):
    changed = dict(request); changed[key] = 'mutated'
    rejected(lambda: caller.validate_saved_request(selected, payload, changed))
print('b: mutated saved request refused')

steps = []
def prestart():
    steps.append('pre')
    if steps.count('pre') == 2: raise ValueError('freshness')
rejected(lambda: caller.admission_sequence(prestart, lambda: [], lambda: steps.append('intent'), lambda: steps.append('pilot'), a['source'], a['name']))
assert steps == ['pre','pre']
print('c: second prestart failure left intent and pilot untouched')

for row in ({'id':'1','source':a['source'],'name':'other'}, {'id':'1','source':'other','name':a['name']}, {'id':'1','source':'other'}):
    rejected(lambda: caller.reject_duplicates([row], a['source'], a['name']))
print('d: source OR name duplicates and incomplete records refused')

with tempfile.TemporaryDirectory() as directory:
    root = Path(directory)
    intent = root / 'intent'
    caller.claim_intent(intent, {'state':'AMBIGUOUS'})
    rejected(lambda: caller.claim_intent(intent, {'state':'retry'}), FileExistsError)
    assert json.loads(intent.read_bytes()) == {'state':'AMBIGUOUS'}
    print('e: ambiguous create-only intent refuses retry')
    called = []
    rejected(lambda: caller.guarded_collect(selected, payload, request, {'id':'wrong'}, 'Autismo-Yah', 'expected', lambda _:called.append('inspect'), lambda:called.append('logs'), root/'unused'))
    assert called == ['inspect'] and not (root/'unused').exists()
    def broken_logs():
        yield 'prefix'
        raise RuntimeError('iterator failed')
    rejected(lambda: caller.retain_log_prefix(root/'partial', broken_logs()), RuntimeError)
    assert (root/'partial').read_text() == 'prefix\n'
    print('f: wrong job refused before logs; failed iterator kept prefix')
print('synthetic only; no selected plan, provider, or game operation')
