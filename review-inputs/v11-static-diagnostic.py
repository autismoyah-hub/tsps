"""UNREVIEWED offline reconstruction only; no original failure attribution."""
import datetime,hashlib,json,pathlib,re,shlex
R=pathlib.Path('/tmp/zogre-hf-case')
PINS={'selection.json':'5d8f097580fa66bea497f12ccc5a0ed5d1bdf3d1852a90d6b96ea13b90d4c063','case-entry-v11.py':'67b533c7d9f877a125ef6b089a84ed49e707e33fcfa40ec4cb7edccc72b11c4e','native-payload.cjs':'220ef3554ec1bf8951c22dc8e50b849feeeaaa5b0b0b06f664ea7b393cb1dde4','pilot-selected-draft.json':'f8013c6ec9a9bc191db918dc8d93c2903cbb1991590887420b97a77cae2b3f9f','owner-v11-context.json':'597b433996de9a7d0cec597daf09a8fa3ab8c6149e9efa2b4731c1f638e82372','parent-v11-context.json':'57dd1a93fb561b439977ed2c8fb213ec0f79f81e00e4bb1e5e5db79bcf66f641','peer-v11-context.json':'3055ce8effe63a1655991263ed3dd1c7f614835cf0eaa38c50835f2778b9e6f4'}
def hash_(b):return hashlib.sha256(b).hexdigest()
def tm(t):return datetime.datetime.fromisoformat(t.replace('Z','+00:00'))
def held(text):
 body='\n'.join(x for x in text.splitlines() if not x.lstrip().startswith('>')).lower()
 return bool(re.search(r'\b(stop|pause|cancel|revoke|halt|hold)\b',body) or re.search(r'\b(do not|don.t|must not|never)\b.{0,80}\b(submit|launch|execute|start|run|select|claim)\b',body,re.S))
def main():
 data={k:(R/k).read_bytes() for k in PINS};seals={k:hash_(data[k])==v for k,v in PINS.items()}
 if not all(seals.values()):print(json.dumps({'stage':'INPUT_MISMATCH','seals':seals}));return
 s=json.loads(data['selection.json']);start=tm(s['selectedUTC']);end=tm('2026-11-09T03:51:50+00:00');p=json.loads(data['pilot-selected-draft.json']);payload=data['native-payload.cjs'].decode();program=payload.replace('__PLAN__',json.dumps(p)).encode()
 files={n:json.loads(data[n])['turns'] for n in ('owner-v11-context.json','parent-v11-context.json','peer-v11-context.json')}
 windows={n:bool(v and v[0].get('ts') and tm(v[0]['ts'])<=start) for n,v in files.items()}
 held_counts={n:sum(held(t.get('text') or '') for t in turns if t.get('role')=='user' and t.get('ts') and start<tm(t['ts'])<end) for n,turns in files.items() if n!='peer-v11-context.json'}
 marker_key='selection='+PINS['selection.json'];actions=[]
 for i,t in enumerate(files['peer-v11-context.json']):
  name=(t.get('tool') or {}).get('name') or ''
  if t.get('role')!='assistant' or t.get('kind')!='tool' or marker_key not in name or not t.get('ts') or not start<tm(t['ts'])<end:continue
  try:argv=shlex.split(name)
  except ValueError:continue
  if len(argv)!=5 or argv[:4]!=['cuttle','sessions','send',s['owner']]:continue
  results=[x for x in files['peer-v11-context.json'][i+1:] if x.get('kind')=='tool_result' and (x.get('tool') or {}).get('id')==(t.get('tool') or {}).get('id')]
  delivered=[x for x in files['owner-v11-context.json'] if x.get('role')=='user' and x.get('ts') and start<tm(x['ts'])<end and s['reviewer'] in (x.get('text') or '') and (x.get('text') or '').split('\n',1)[-1]==argv[4]]
  if results and delivered:actions.append(True)
 tokens=[i+1 for i,line in enumerate(payload.splitlines()) if '__PLAN__' in line]
 print(json.dumps({'stage':'STATIC_RECONSTRUCTION_ONLY','seals':seals,'fixtureClockUTC':end.isoformat(),'windowsStartBeforeSelection':windows,'knownHoldCounts':held_counts,'peerSelectionActionResultAndDeliveryCount':len(actions),'payloadTokenLines':tokens,'renderedProgramMatchesSelection':hash_(program)==s['programSHA256'],'sessionConfigCredentialsAndOriginalControlFlow':'UNKNOWN','providerAccess':False,'originalExceptionAttribution':'UNKNOWN'}))
if __name__=='__main__':main()
