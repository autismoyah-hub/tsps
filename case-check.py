"""Offline identity check for held minnow profiles. This program cannot submit jobs."""
import hashlib,json,pathlib,sys
root=pathlib.Path(__file__).resolve().parent
pins={
 'before-profile.json':'76766036de049e701cbbf3cfae7c921d38607d02fc69622fa56a17e0dbb9e9b7',
 'after-profile.json':'fe494252ab4587a407c346485f3fdd847b5e659233461d9db41687e4bb1c82f4',
 'before-request.json':'4d72bb430af2905e1f63fdf6f995bda09871508874c6bea5de8da7264c774228',
 'after-request.json':'c6658b6347a0d359cdf147b230d09e466817c129a065aaca23aacc49d3f65905',
 'native-payload-602.cjs':'6028946a60c124772178a55f61285b6ac9eef1a05a5362199a836e8bea7ebfc3',
 'package.json':'ac9cb75167ec66ac1c1f4345d65bda679a20ac1b9a85aba45dd1919dcd18ac78',
 'pilot.py':'2d7c5e44bb37f3043311962f8c05deee90e83ab6fa5ee5e3b499df238ed98200',
 'hf-project-access-v4.py':'fab4bc92dc8601143ce61e910f8121774c22a0d19be6d2617bba9449f7131326',
 'frame_transport.py':'f53689d4f2389af93ad8f6a43fe0de94caf8dcaabde3eacbd13b1c957b129a76',
 'retain-job-result.py':'bdcb7f104ebfa8c26d2aa68c1dd31cf7780f12174eb92d3e98ffb928797a9071',
 'retain-contiguous-inputs003.py':'b399ca13e8fbfee6f7633a9d430d85c8204774c2941a2e4d5f74c642820fa690',
}
for name,digest in pins.items():
 actual=hashlib.sha256((root/name).read_bytes()).hexdigest()
 if actual!=digest: raise ValueError('identity mismatch: '+name)
for side in ('before','after'):
 plan=json.loads((root/(side+'-profile.json')).read_text());req=json.loads((root/(side+'-request.json')).read_text())
 if not plan['executionHold'] or plan['ownerSelection'] is not None: raise ValueError('not held/unselected')
 command=(root/plan['payloadFile']).read_text().replace('__PLAN__',json.dumps(plan))
 expected={k:plan[k] for k in ('name','image','flavor','timeout','attempts')};expected.update(command=['node','-e',command],labels={'project':'tsps','source':plan['source'],'purpose':'overflow-qualification'})
 if req!=expected: raise ValueError('request mismatch: '+side)
 raw=(root/(side+'-manifest.json')).read_bytes();ref=plan['expectedInputsRef']
 if len(raw)!=ref['bytes'] or hashlib.sha256(raw).hexdigest()!=ref['sha256']: raise ValueError('manifest mismatch: '+side)
print(json.dumps({'status':'HELD','verifiedFiles':len(pins),'requests':2,'selected':False,'submitted':False}))
