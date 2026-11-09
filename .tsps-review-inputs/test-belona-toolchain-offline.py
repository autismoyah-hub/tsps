"""Synthetic PATH/SDK identity mutation controls; no auth or provider calls."""
import hashlib,importlib.util,json,os,tempfile
from pathlib import Path
from unittest.mock import patch
root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('prestart',root/'belona-prestart.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def rejected(fn):
 try:fn()
 except ValueError:return
 raise AssertionError('expected refusal')
with tempfile.TemporaryDirectory() as tmp:
 d=Path(tmp);cli=d/'cuttle';node=d/'node';module=d/'cli.mjs';module.write_text('synthetic');node.write_text('node fake');node.chmod(0o755)
 cli.write_text('#!/bin/sh\nexec node '+str(module.resolve())+' "$@"\n');cli.chmod(0o755)
 sdk=d/'sdk';metadata=sdk/'huggingface_hub-2.4.0.dist-info'/'METADATA';metadata.parent.mkdir(parents=True);metadata.write_text('Metadata-Version: 2.1\nName: huggingface_hub\nVersion: 2.4.0\n')
 manifest=d/'sdk.json';manifest.write_text(json.dumps({'root':str(sdk),'version':'2.4.0','files':{metadata.relative_to(sdk).as_posix():digest(metadata)}}))
 subject={'cliSHA256':digest(cli),'nodeSHA256':digest(node),'cliModuleSHA256':digest(module),'sdkManifestSHA256':digest(manifest)}
 with patch.dict(os.environ,{'PATH':str(d)}):
  m.verify_toolchain(subject,cli,node,module);m.verify_sdk(subject,manifest,sdk)
  module.write_text('changed');rejected(lambda:m.verify_toolchain(subject,cli,node,module));module.write_text('synthetic')
  node.write_text('changed');rejected(lambda:m.verify_toolchain(subject,cli,node,module));node.write_text('node fake')
  wrong=d/'elsewhere';wrong.mkdir();(wrong/'node').write_text('different');(wrong/'node').chmod(0o755)
  with patch.dict(os.environ,{'PATH':str(wrong)+os.pathsep+str(d)}):rejected(lambda:m.verify_toolchain(subject,cli,node,module))
  with patch.dict(os.environ,{'PATH':str(wrong)}):rejected(lambda:m.verify_toolchain(subject,cli,node,module))
  metadata.write_text('changed');rejected(lambda:m.verify_sdk(subject,manifest,sdk));metadata.write_text('Metadata-Version: 2.1\nName: huggingface_hub\nVersion: 2.4.0\n')
  extra=sdk/'extra';extra.write_text('x');rejected(lambda:m.verify_sdk(subject,manifest,sdk));extra.unlink()
  rejected(lambda:m.verify_sdk(subject,manifest,wrong))
print('synthetic CLI PATH/node/module and SDK content/membership/root mismatches refused; no auth/provider')
