"""UNREVIEWED read-only reconstruction, not the original exception.
Pure checks on explicit retained artifacts; no credentials, network or mutation.
"""
import datetime,hashlib,json,pathlib,re,shlex
ROOT=pathlib.Path('/tmp/zogre-hf-case')
EXPECTED={'selection.json':'c0b26af04195ab30cadea9803fc02cea276e4e977556a0972c8e00ba747e3bd2','native-payload.cjs':'220ef3554ec1bf8951c22dc8e50b849feeeaaa5b0b0b06f664ea7b393cb1dde4','case-entry-v10.py':'151d21c8d636ad5aec035a8350111d33a345cc360b170f48a330720cddf090ed','owner-v10-actual-context.json':'9c47260c12f4a29386c95077f97f1039db507ac8f3c62aaebd2c8dea5e6b0457','parent-v10-actual-context.json':'146724c1fea180b7b69dc15ff9277233e1307bb07376bd148729ae63765e1cd2'}
def read(name):return (ROOT/name).read_bytes()
def sha(b):return hashlib.sha256(b).hexdigest()
def parse(t):return datetime.datetime.fromisoformat(t.replace('Z','+00:00'))
def known_hold(text):
 lines=[x for x in text.splitlines() if not x.lstrip().startswith('>')];body='\n'.join(lines).lower()
 return bool(re.search(r'\b(stop|pause|cancel|revoke|halt|hold)\b',body) or re.search(r'\b(do not|don.t|must not|never)\b.{0,80}\b(submit|launch|execute|start|run|select|claim)\b',body,re.S))
def main():
 seals={name:sha(read(name))==digest for name,digest in EXPECTED.items()}
 if not all(seals.values()):print(json.dumps({'stage':'INPUT_MISMATCH','seals':seals}));return
 selection=json.loads(read('selection.json'));start=parse(selection['selectedUTC']);end=parse('2026-11-09T03:34:16+00:00')
 outputs={}
 for name in ('owner-v10-actual-context.json','parent-v10-actual-context.json'):
  turns=json.loads(read(name))['turns']; relevant=[t for t in turns if t.get('role')=='user' and t.get('ts') and start<parse(t['ts'])<end]
  outputs[name]={'windowStartsBeforeSelection':bool(turns and turns[0].get('ts') and parse(turns[0]['ts'])<=start),'knownHoldCount':sum(known_hold(t.get('text') or '') for t in relevant)}
 payload=read('native-payload.cjs').decode();lines=payload.splitlines();indices=[i+1 for i,line in enumerate(lines) if '__PLAN__' in line]
 print(json.dumps({'stage':'STATIC_RECONSTRUCTION_ONLY','seals':seals,'fixtureClockUTC':end.isoformat(),'transcriptChecks':outputs,'payloadTokenCount':payload.count('__PLAN__'),'payloadTokenLines':indices,'v10GuardLiteralFound':"payload.count('__PLAN__')!=1" in read('case-entry-v10.py').decode(),'liveConfigIdentityAndControlFlow':'UNKNOWN','providerAccess':False,'originalExceptionAttribution':'UNKNOWN'}))
if __name__=='__main__':main()
