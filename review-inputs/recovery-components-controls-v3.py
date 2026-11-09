"""Unselected non-GAME controls for recovery draft components; no network."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import threading
import unittest
from unittest import mock

ROOT = Path('/tmp/zogre-hf-case')

def module(name, path, expected):
    import types
    raw = path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != expected:
        raise RuntimeError('COMPONENT_HASH_MISMATCH')
    result = types.ModuleType(name)
    exec(compile(raw, str(path), 'exec'), result.__dict__)
    return result

binding = module('binding', ROOT / 'recovery-binding-draft-v1.py', '722073938c846568ff073098702b72c2fd083b3d55081af050decd4b9b0daab6')
records = module('records', ROOT / 'recovery-records-draft-v5.py', 'b37e74ba6bde114d968dead3a0ad2b4d7f2306976e6c402af98368584826b22c')

class BindingControls(unittest.TestCase):
    def setUp(self):
        names = {'return': 'provider-return.json', 'unknown': 'submission-unknown.json', 'intent': 'case-intent-v16.json', 'selection': 'selection.json'}
        self.raw = {key: (ROOT / value).read_bytes() for key, value in names.items()}
        req = json.loads(self.raw['intent'])['request']
        self.observed = {'id': binding.JOB, 'owner': binding.NAMESPACE, 'image': req['image'], 'command': req['command'], 'flavor': req['flavor'], 'retry': 0, 'labels': dict(req['labels'], name=req['name']), 'stage': 'ERROR'}

    def test_exact_binding_and_bad_values(self):
        self.assertEqual(binding.validate(self.raw, self.observed), 'BOUND_TERMINAL')
        for field in ('id', 'owner', 'image', 'command', 'flavor', 'retry'):
            with self.subTest(field=field):
                self.assertNotEqual(binding.validate(self.raw, dict(self.observed, **{field: False})), 'BOUND_TERMINAL')
        for stage in ('RUNNING', 'COMPLETED', 'CANCELED', 'DELETED', None):
            self.assertEqual(binding.validate(self.raw, dict(self.observed, stage=stage)), 'STATUS_NOT_EXPECTED')
        for key in self.raw:
            altered = dict(self.raw); altered[key] += b' '
            self.assertEqual(binding.validate(altered, self.observed), 'ORIGINAL_HASH_MISMATCH')
        for key in self.observed['labels']:
            altered = dict(self.observed); altered['labels'] = dict(self.observed['labels']); altered['labels'][key] = 'bad'
            self.assertEqual(binding.validate(self.raw, altered), 'JOB_LABEL_MISMATCH')
        for labels in (None, [], {}):
            altered = dict(self.observed, labels=labels)
            self.assertEqual(binding.validate(self.raw, altered), 'JOB_LABEL_MISMATCH')
        self.assertEqual(binding.validate(self.raw, {}), 'JOB_IDENTITY_MISMATCH')
        missing = dict(self.raw); missing.pop('return')
        self.assertEqual(binding.validate(missing, self.observed), 'ORIGINAL_SET_MISMATCH')
        extra = dict(self.raw, extra=b'')
        self.assertEqual(binding.validate(extra, self.observed), 'ORIGINAL_SET_MISMATCH')
        altered = dict(self.observed); altered['labels'] = dict(self.observed['labels'], extra='bad')
        self.assertEqual(binding.validate(self.raw, altered), 'JOB_LABEL_MISMATCH')

class RecordControls(unittest.TestCase):
    def test_records_and_failures(self):
        with tempfile.TemporaryDirectory(prefix='zogre-control-') as tmp:
            d = Path(tmp)
            result = records.retain_records(['one', 'é\n'], d / 'complete')
            self.assertEqual(result['outcome'], 'COMPLETE')
            self.assertEqual(result['fileSHA256'], hashlib.sha256((d / 'complete').read_bytes()).hexdigest())
            self.assertEqual([json.loads(s) for s in (d / 'complete').read_text().splitlines()], ['one', 'é\n'])
            def timeout():
                yield 'one'
                raise TimeoutError()
            result = records.retain_records(timeout(), d / 'timeout')
            self.assertEqual((result['outcome'], result['reason'], result['records']), ('UNKNOWN', 'TIMEOUT', 1))
            self.assertEqual(result['fileSHA256'], hashlib.sha256((d / 'timeout').read_bytes()).hexdigest())
            original_write = os.write
            with mock.patch.object(records.os, 'write', side_effect=lambda fd, data: original_write(fd, data[:1])):
                result = records.retain_records(['one'], d / 'short')
            self.assertEqual((result['outcome'], result['reason'], result['fileBytes']), ('UNKNOWN', 'SHORT_WRITE', 1))
            with mock.patch.object(records.json, 'dumps', side_effect=TimeoutError()):
                self.assertEqual(records.retain_records(['one'], d / 'encode')['reason'], 'TIMEOUT')
            with mock.patch.object(records.os, 'fsync', side_effect=OSError()):
                result = records.retain_records(['one'], d / 'fsync')
            self.assertEqual(result['outcome'], 'UNKNOWN')
            self.assertIn('FSYNC_ERROR', result['cleanupErrors'])
            self.assertIn('DIRECTORY_FSYNC_ERROR', result['cleanupErrors'])
            self.assertEqual(records.retain_records(['x'], d / 'complete')['outcome'], 'UNKNOWN')

    def test_competing_file_creation(self):
        with tempfile.TemporaryDirectory(prefix='zogre-control-') as tmp:
            p = Path(tmp) / 'same'; barrier = threading.Barrier(2); results = []
            errors = []
            def run():
                try:
                    barrier.wait(timeout=2); results.append(records.retain_records(['x'], p)['outcome'])
                except BaseException as exc:
                    errors.append(type(exc).__name__)
            threads = [threading.Thread(target=run, daemon=True) for _ in range(2)]
            for thread in threads: thread.start()
            for thread in threads: thread.join(timeout=3)
            self.assertFalse(any(thread.is_alive() for thread in threads))
            self.assertEqual(errors, [])
            self.assertCountEqual(results, ['COMPLETE', 'UNKNOWN'])

if __name__ == '__main__':
    unittest.main()
