// Linux qualification of existing upstream commands, not a gameplay harness.
// __PLAN__ is replaced with the frozen JSON plan by pilot.py, using SDK argv.
const PLAN = __PLAN__;
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const zlib = require('node:zlib');
const {spawn} = require('node:child_process');
const root = '/tmp/tsps-shades-260b-materialize';
const server = path.join(root, 'server');
const emit = value => console.log(JSON.stringify(value));
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const fileHash = file => sha(fs.readFileSync(file));
function treeManifest(directory) {
  return fs.readdirSync(directory, {recursive: true}).sort().flatMap(relative => {
    const full = path.join(directory, relative);
    const stat = fs.lstatSync(full);
    if (stat.isSymbolicLink()) return [{file: relative, link: fs.readlinkSync(full)}];
    return stat.isFile() ? [{file: relative, sha256: fileHash(full)}] : [];
  });
}
let sealState = null;

function captureSeals(afterFailure) {
  if (!sealState) {
    emit({event: 'source-closure', afterFailure, available: false,
      limitation: 'Source extraction did not finish; no post-input seal can be claimed'});
    return false;
  }
  const {originalSource, sourceManifestHash, distManifest, cacheFiles} = sealState;
  const errors = [];
  let sourceStatus = 'UNAVAILABLE';
  let distStatus = 'UNAVAILABLE';
  let cacheStatus = 'UNAVAILABLE';
  let postSourceHash = null;
  let postDistHash = null;
  const cacheMismatches = [];
  try {
    const postSource = originalSource.map(row => row.link !== undefined
      ? {file: row.file, link: fs.readlinkSync(path.join(root, row.file))}
      : {file: row.file, sha256: fileHash(path.join(root, row.file))});
    postSourceHash = sha(JSON.stringify(postSource));
    sourceStatus = postSourceHash === sourceManifestHash ? 'PASS' : 'MISMATCH';
    if (sourceStatus !== 'PASS') errors.push('Tracked source content changed');
  } catch (error) { sourceStatus = 'MISMATCH'; errors.push(`Source seal: ${error.message}`); }
  if (distManifest) {
    try {
      postDistHash = sha(JSON.stringify(treeManifest(path.join(server, 'dist'))));
      distStatus = postDistHash === sha(JSON.stringify(distManifest)) ? 'PASS' : 'MISMATCH';
      if (distStatus !== 'PASS') errors.push('Compiled output changed');
    } catch (error) { distStatus = 'MISMATCH'; errors.push(`Dist seal: ${error.message}`); }
  }
  if (cacheFiles) {
    try {
      const actual = {};
      const cacheTree = path.join(server, 'caches');
      if (!fs.lstatSync(cacheTree).isDirectory() || fs.lstatSync(cacheTree).isSymbolicLink()) throw new Error('Cache root is not a regular directory');
      const entries = fs.readdirSync(cacheTree, {recursive: true}).map(file => `caches/${file.split(path.sep).join('/')}`);
      for (const file of entries.sort()) {
        const full = path.join(server, file);
        const stat = fs.lstatSync(full);
        if (stat.isSymbolicLink()) throw new Error(`Unexpected cache symlink: ${file}`);
        if (stat.isDirectory()) {
          if (!Object.keys(cacheFiles).some(key => key.startsWith(file + '/'))) throw new Error(`Unexpected cache directory: ${file}`);
          continue;
        }
        if (!stat.isFile()) throw new Error(`Unexpected cache entry type: ${file}`);
        actual[file] = fileHash(full);
      }
      for (const file of new Set([...Object.keys(cacheFiles), ...Object.keys(actual)])) {
        if (actual[file] !== cacheFiles[file]) cacheMismatches.push(file);
      }
      cacheStatus = cacheMismatches.length ? 'MISMATCH' : 'PASS';
    } catch (error) { cacheStatus = 'MISMATCH'; errors.push(`Cache seal: ${error.message}`); }
    if (cacheMismatches.length) errors.push('Cache membership or content changed');
  }
  const complete = sourceStatus !== 'UNAVAILABLE' && distStatus !== 'UNAVAILABLE' && cacheStatus !== 'UNAVAILABLE';
  const sealsUnchanged = complete && [sourceStatus, distStatus, cacheStatus].every(status => status === 'PASS');
  emit({event: 'source-closure', afterFailure, available: true, sourceManifestHash,
    postSourceHash, originalSourceFiles: originalSource.length,
    distManifestHash: distManifest ? sha(JSON.stringify(distManifest)) : null,
    postDistHash, postCacheFilesChecked: cacheFiles ? Object.keys(cacheFiles).length : 0,
    sourceStatus, distStatus, cacheStatus, complete, cacheMismatches, errors, sealsUnchanged,
    limitation: 'Content identities only; no process or sandbox-wide closure inference'});
  return sealsUnchanged;
}
const env = {PATH: '/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin',
  LANG: 'C.UTF-8', TZ: 'UTC', DISABLE_PLAYER_BOTS: '1', NODE_DISABLE_COMPILE_CACHE: '1',
  YARN_ENABLE_SCRIPTS: 'false', npm_config_ignore_scripts: 'true',
  CI: '1', npm_config_cache: '/tmp/npm-cache', HOME: '/tmp/tsps-private-home',
  TMPDIR: '/tmp/tsps-private-temp', TEMP: '/tmp/tsps-private-temp', TMP: '/tmp/tsps-private-temp'};

function resources() {
  const out = {};
  for (const key of ['memory.max', 'memory.peak', 'memory.events', 'pids.current']) {
    try { out[key] = fs.readFileSync(`/sys/fs/cgroup/${key}`, 'utf8').trim(); } catch {}
  }
  return out;
}

function stage(name, command, args, cwd, timeoutMs) {
  emit({event: 'stage-start', name, command, args, utc: new Date().toISOString()});
  return new Promise((resolve, reject) => {
    const started = Date.now();
    const child = spawn(command, args, {cwd, env, detached: true, stdio: ['ignore', 'pipe', 'pipe']});
    let output = '';
    let outputExceeded = false;
    child.stdout.on('data', chunk => {
      process.stdout.write(chunk);
      if (name.startsWith('test:') || name === 'ldd') {
        if (output.length + chunk.length <= 4 * 1024 * 1024) output += chunk;
        else outputExceeded = true;
      }
    });
    child.stderr.on('data', chunk => process.stderr.write(chunk));
    let timedOut = false;
    const timer = setTimeout(() => {
      timedOut = true;
      try { process.kill(-child.pid, 'SIGKILL'); } catch {}
    }, timeoutMs);
    child.once('error', error => {clearTimeout(timer); reject(error);});
    child.once('close', (code, signal) => {
      clearTimeout(timer);
      let leftoverGroup = false;
      try { process.kill(-child.pid, 0); leftoverGroup = true; } catch {}
      if (leftoverGroup) {
        try { process.kill(-child.pid, 'SIGKILL'); } catch {}
      }
      const tests = /^# tests (\d+)$/m.exec(output);
      const passes = /^# pass (\d+)$/m.exec(output);
      const failures = /^# fail (\d+)$/m.exec(output);
      const skipped = /^# skipped (\d+)$/m.exec(output);
      const canceled = /^# cancelled (\d+)$/m.exec(output);
      const isTest = name.startsWith('test:');
      const expected = isTest ? PLAN.expectedTestCounts?.[name.slice(5)] : null;
      const tapValid = !isTest || (!outputExceeded && tests && passes && failures && skipped && canceled
        && Number(tests[1]) > 0 && Number(tests[1]) === Number(passes[1])
        && Number(failures[1]) === 0 && Number(skipped[1]) === 0 && Number(canceled[1]) === 0
        && (expected == null || Number(tests[1]) === expected));
      emit({event: 'stage-result', name, code, signal, timedOut, leftoverGroup, tapValid: !!tapValid,
        closureScope: 'process-group only; reparented or setsid descendants not proved absent',
        tests: tests ? Number(tests[1]) : null, passes: passes ? Number(passes[1]) : null,
        durationMs: Date.now() - started, resources: resources()});
      if (timedOut || leftoverGroup || code !== 0 || !tapValid) {
        const error = new Error(`${name}: exit=${code}, signal=${signal}, timeout=${timedOut}, leftover=${leftoverGroup}, tapValid=${!!tapValid}`);
        error.exitCode = 70;
        error.outcome = timedOut || leftoverGroup ? 'UNKNOWN' : isTest ? 'PAYLOAD_NONZERO_OR_TAP_UNCLASSIFIED' : 'SETUP_FAILURE';
        reject(error);
      } else resolve(output);
    });
  });
}

const materialize = (() => { const module = {exports:{}};
// HELD case-specific source candidate. Do not execute before exact review/selection.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const ROOT = '/tmp/tsps-shades-260b-materialize';
const PREFIX = 'server/plugins/skills/crafting/';
const FILE_LIMIT = 64 * 1024 * 1024;
const TOTAL_LIMIT = 192 * 1024 * 1024;
// This bounds network/async work; synchronous fsync and bounded readback can exceed it.
const DEADLINE_MS = 900000;
const MANIFESTS = {
  '8489c88a87c96730aa0f441895c07884bd04c358': '3d72850b9a395846a82d0cc1477298e5c2c9948cc6e9a0baedbaf25e36dda73f',
  'edc764a2d709df88110f7db6204aa263d11307c4': 'f20e028e29adbde75a48f0fd565dd6e0793571240c7bf91a4b2abbd5eb4c7e91',
};
// Caller must bind this job as the exclusive same-UID owner of ROOT until it exits.
// Directory checks cannot prevent a hostile same-UID process from renaming an ancestor.
const digest = (kind, value) => crypto.createHash(kind).update(value).digest('hex');
function checkDirectory(dir) {
  const parts = path.resolve(dir).split(path.sep).filter(Boolean);
  let current = path.parse(path.resolve(dir)).root;
  for (const part of parts) {
    current = path.join(current, part);
    const stat = fs.lstatSync(current);
    if (stat.isSymbolicLink() || !stat.isDirectory() || fs.realpathSync(current) !== current) throw Error('unsafe directory');
  }
}
function ensureParents(file) {
  const dir = path.dirname(file);
  if (dir !== ROOT && !dir.startsWith(ROOT + path.sep)) throw Error('target escape');
  let current = ROOT;
  for (const part of path.relative(ROOT, dir).split(path.sep).filter(Boolean)) {
    current = path.join(current, part);
    try { fs.mkdirSync(current, {mode:0o700}); } catch (error) { if (error.code !== 'EEXIST') throw error; }
    checkDirectory(current);
  }
}
function checkOwnedRoot() {
  checkDirectory(ROOT);
  const stat = fs.lstatSync(ROOT);
  if (stat.uid !== process.getuid() || (stat.mode & 0o777) !== 0o700) throw Error('case root ownership or mode mismatch');
}
async function materialize(manifest, root, repository) {
  if (root !== ROOT || manifest.schema !== 1 || manifest.excludedPrefix !== PREFIX || manifest.excludedCount !== 11 ||
      !MANIFESTS[manifest.source] || digest('sha256', Buffer.from(JSON.stringify(manifest) + '\n')) !== MANIFESTS[manifest.source] ||
      repository !== 'autismoyah-hub/tsps') throw Error('profile mismatch');
  const files = Object.keys(manifest.expectedInputs).sort();
  if (files.length !== 5036 || JSON.stringify(files) !== JSON.stringify(Object.keys(manifest.sourceBlobs).sort())) throw Error('manifest membership');
  for (const file of files) {
    if (file.startsWith(PREFIX) || file.startsWith('/') || file.includes('\\') || file.includes(':') ||
        file.split('/').some(p => !p || p === '.' || p === '..') ||
        !/^[0-9a-f]{64}$/.test(manifest.expectedInputs[file]) || !/^[0-9a-f]{40}$/.test(manifest.sourceBlobs[file])) throw Error('unsafe manifest row');
  }
  checkDirectory(path.dirname(ROOT));
  fs.mkdirSync(ROOT, {mode:0o700}); // Refuse any previous root, including partial attempts.
  checkOwnedRoot();
  let next = 0, total = 0, completed = 0;
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), DEADLINE_MS);
  async function worker() {
    while (next < files.length && !controller.signal.aborted) {
      const file = files[next++], target = path.join(ROOT, file);
      const url = `https://raw.githubusercontent.com/${repository}/${manifest.source}/${file.split('/').map(encodeURIComponent).join('/')}`;
      const response = await fetch(url, {redirect:'error', signal:AbortSignal.any([controller.signal, AbortSignal.timeout(60000)])});
      if (!response.ok || response.url !== url || !response.body) throw Error(`input HTTP status ${response.status}: ${file}`);
      checkOwnedRoot();
      ensureParents(target);
      const fd = fs.openSync(target, fs.constants.O_WRONLY | fs.constants.O_CREAT | fs.constants.O_EXCL | fs.constants.O_NOFOLLOW, 0o600);
      let size = 0;
      const h = crypto.createHash('sha256');
      try {
        for await (const chunk of response.body) {
          size += chunk.length; total += chunk.length;
          if (size > FILE_LIMIT || total > TOTAL_LIMIT) throw Error('input byte limit');
          h.update(chunk);
          let offset = 0;
          while (offset < chunk.length) {
            const written = fs.writeSync(fd, chunk, offset, chunk.length - offset);
            if (written <= 0) throw Error('zero-byte output write');
            offset += written;
          }
        }
        fs.fsyncSync(fd);
      } finally { fs.closeSync(fd); }
      checkOwnedRoot();
      checkDirectory(path.dirname(target));
      const stat = fs.lstatSync(target);
      if (!stat.isFile() || stat.isSymbolicLink() || fs.realpathSync(target) !== target || stat.size !== size) throw Error('unsafe output file');
      if (h.digest('hex') !== manifest.expectedInputs[file]) throw Error('input SHA256 mismatch');
      const blobHash = crypto.createHash('sha1').update(`blob ${size}\0`);
      for await (const chunk of fs.createReadStream(target)) blobHash.update(chunk);
      if (blobHash.digest('hex') !== manifest.sourceBlobs[file]) throw Error('input blob mismatch');
      completed++;
    }
  }
  try {
    const results = await Promise.allSettled(Array.from({length:8}, async () => {
      try { await worker(); } catch (error) { controller.abort(); throw error; }
    }));
    const failed = results.find(r => r.status === 'rejected');
    if (failed) throw failed.reason;
    if (controller.signal.aborted || completed !== files.length) throw Error('deadline or incomplete input');
    const actual = fs.readdirSync(ROOT, {recursive:true}).filter(p => fs.lstatSync(path.join(ROOT,p)).isFile()).map(p => p.split(path.sep).join('/')).sort();
    if (JSON.stringify(actual) !== JSON.stringify(files)) throw Error('materialized membership mismatch');
    for (const entry of fs.readdirSync(ROOT, {recursive:true})) {
      const full = path.join(ROOT, entry), stat = fs.lstatSync(full);
      if (stat.isSymbolicLink() || (!stat.isFile() && !stat.isDirectory()) || fs.realpathSync(full) !== full) throw Error('unsafe materialized entry');
    }
    return {event:'filtered-materialized', source:manifest.source, included:completed, bytes:total, excludedPrefix:PREFIX, excludedCount:11};
  } catch (error) {
    console.error(JSON.stringify({event:'filtered-materialization-failed', completed, bytes:total, error:String(error), partialRootRetained:ROOT}));
    throw error;
  } finally { clearTimeout(timer); }
}
module.exports = { materialize };

return module.exports.materialize; })();

(async () => {
  if (process.platform !== 'linux') throw new Error('Linux qualification only');
  if (!/^[a-f0-9]{40}$/.test(PLAN.source)) throw new Error('Full immutable Git SHA required');
  const expected = {
    '8489c88a87c96730aa0f441895c07884bd04c358': ['shades-logout-before.json','3d72850b9a395846a82d0cc1477298e5c2c9948cc6e9a0baedbaf25e36dda73f'],
    'edc764a2d709df88110f7db6204aa263d11307c4': ['shades-logout-after.json','f20e028e29adbde75a48f0fd565dd6e0793571240c7bf91a4b2abbd5eb4c7e91']
  }[PLAN.source];
  if (!expected || PLAN.repository !== 'autismoyah-hub/tsps' || JSON.stringify(PLAN.tests) !== JSON.stringify(['shades-sanctity-logout.test.cjs'])) throw Error('case input mismatch');
  const url = `https://raw.githubusercontent.com/autismoyah-hub/tsps/57a52f66f1fe2c3d5e1f60e9ac8a5378138ec54b/case-inputs/${expected[0]}`;
  const response = await fetch(url,{redirect:'error',signal:AbortSignal.timeout(60000)});
  if (!response.ok || response.url !== url || !response.body) throw Error(`manifest HTTP ${response.status}`);
  let bytes = 0; const parts = [];
  for await (const chunk of response.body) { bytes += chunk.length; if (bytes > 1092245) throw Error('manifest bound'); parts.push(Buffer.from(chunk)); }
  const raw = Buffer.concat(parts);
  if (bytes !== 1092245 || sha(raw) !== expected[1]) throw Error('manifest identity');
  const inputManifest = JSON.parse(raw.toString('utf8'));
  const expectedInputs = inputManifest.expectedInputs;
  fs.mkdirSync(env.HOME); fs.mkdirSync(env.TMPDIR);
  emit(await materialize(inputManifest, root, PLAN.repository));
  const originalSource = treeManifest(root);
  const sourceManifestHash = sha(JSON.stringify(originalSource));
  sealState = {originalSource, sourceManifestHash, distManifest: null, cacheFiles: null};
  const inputs = {};
  for (const [file, expected] of Object.entries(expectedInputs)) {
    const resolved = path.resolve(root, file);
    if (!resolved.startsWith(root + '/')) throw new Error('Input path escapes source');
    const actual = fileHash(resolved);
    inputs[file] = actual;
    if (actual !== expected) throw new Error(`Source input mismatch: ${file}`);
    if (inputManifest.sourceBlobs) {
      const raw=fs.readFileSync(resolved);
      const blob=crypto.createHash('sha1').update(Buffer.from(`blob ${raw.length}\0`)).update(raw).digest('hex');
      if(blob!==inputManifest.sourceBlobs[file]) throw new Error(`Source Gitblob mismatch: ${file}`);
    }
  }
  emit({event: 'inputs', source: PLAN.source, repository: PLAN.repository,
    manifestSHA256: expected[1], sourceManifestHash, sourceFileCount: originalSource.length,
    inputs, node: process.version, resources: resources(),
    packageManager: JSON.parse(fs.readFileSync(path.join(server, 'package.json'))).packageManager});
  const packageManager = JSON.parse(fs.readFileSync(path.join(server, 'package.json'))).packageManager;
  const yarnPin = /^yarn@4\.12\.0\+sha512\.([a-f0-9]{128})$/.exec(packageManager);
  if (!yarnPin) throw new Error('Unexpected Yarn packageManager pin');
  const yarnURL = 'https://repo.yarnpkg.com/4.12.0/packages/yarnpkg-cli/bin/yarn.js';
  const yarnResponse = await fetch(yarnURL, {signal: AbortSignal.timeout(60000)});
  if (!yarnResponse.ok) throw new Error(`Yarn HTTP ${yarnResponse.status}`);
  const yarnBytes = Buffer.from(await yarnResponse.arrayBuffer());
  const yarnSHA512 = crypto.createHash('sha512').update(yarnBytes).digest('hex');
  if (yarnSHA512 !== yarnPin[1]) throw new Error('Yarn CLI bytes do not match declared packageManager SHA512');
  const yarnPath = '/tmp/yarn-4.12.0.cjs';
  fs.writeFileSync(yarnPath, yarnBytes, {flag: 'wx'});
  emit({event: 'yarn-input', url: yarnURL, sha512: yarnSHA512, bytes: yarnBytes.length, packageManager});
  await stage('install-server', process.execPath, [yarnPath, 'install', '--immutable', '--mode=skip-build'], server, 480000);
  if (process.arch !== 'x64') throw new Error('Frozen native asset requires Linux x64');
  const nativeURL = 'https://github.com/murat-dogan/node-datachannel/releases/download/v0.32.1/node-datachannel-v0.32.1-napi-v8-linux-x64.tar.gz';
  // Official release asset API serves the same pinned archive via its binary representation.
  const nativeTransportURL = 'https://api.github.com/repos/murat-dogan/node-datachannel/releases/assets/352578537';
  const nativeResponse = await fetch(nativeTransportURL, {headers: {'Accept': 'application/octet-stream', 'User-Agent': 'tsps-verification'}, signal: AbortSignal.timeout(60000)});
  if (!nativeResponse.ok) throw new Error(`Native asset HTTP ${nativeResponse.status}`);
  const nativeArchive = Buffer.from(await nativeResponse.arrayBuffer());
  if (sha(nativeArchive) !== '1a31d28a7aec9c1538cb96b76260d062399715b4dcea0ff3a0ebd6f58df5e391') throw new Error('Native release digest mismatch');
  const tar = zlib.gunzipSync(nativeArchive, {maxOutputLength: 20 * 1024 * 1024});
  const member = tar.subarray(0, 100).toString().replace(/\0.*$/s, '');
  const size = parseInt(tar.subarray(124, 136).toString().replace(/\0.*$/s, '').trim(), 8);
  const type = tar[156];
  const prefix = tar.subarray(345, 500).toString().replace(/\0.*$/s, '');
  const link = tar.subarray(157, 257).toString().replace(/\0.*$/s, '');
  const end = 512 + Math.ceil(size / 512) * 512;
  if (member !== 'build/Release/node_datachannel.node' || prefix || link || ![0, 48].includes(type)
      || size !== 9048672 || tar.subarray(end).some(byte => byte !== 0)) throw new Error('Unexpected native archive layout');
  const binary = tar.subarray(512, 512 + size);
  if (sha(binary) !== '98ad24d9b992a7974628b339ce81dfe159ca8d255459da5531dfcd7be659af06'
      || binary.subarray(0, 4).toString('hex') !== '7f454c46' || binary.readUInt16LE(18) !== 62) throw new Error('Native ELF identity mismatch');
  const nativePath = path.join(server, 'node_modules/node-datachannel', member);
  fs.mkdirSync(path.dirname(nativePath), {recursive: true});
  fs.writeFileSync(nativePath, binary, {flag: 'wx', mode: 0o755});
  emit({event: 'native-input', url: nativeURL, transportURL: nativeTransportURL, archiveSHA256: sha(nativeArchive), member,
    binarySHA256: sha(binary), size, platform: process.platform, architecture: process.arch});
  const libraries = await stage('ldd', 'ldd', [nativePath], server, 30000);
  if (libraries.includes('not found')) throw new Error('Native shared library missing');
  await stage('native-load', process.execPath, ['-e', "require('./node_modules/node-datachannel'); console.log('NATIVE_LOAD_OK')"], server, 30000);
  await stage('build', process.execPath, ['--max-old-space-size=3072', 'node_modules/typescript/bin/tsc', '--build', 'tsconfig.json', '--pretty', 'false'], server, 240000);
  const distManifest = treeManifest(path.join(server, 'dist'));
  sealState.distManifest = distManifest;
  emit({event: 'build-output', files: distManifest.length, manifestSHA256: sha(JSON.stringify(distManifest))});
  await stage('cache', process.execPath, ['--max-old-space-size=4096', 'node_modules/ts-node/dist/bin.js', '--compiler-options', '{"target":"es2020"}', 'scripts/ensure-cache.ts'], server, 480000);
  const target = fs.readFileSync(path.join(server, 'target.txt'), 'utf8').trim();
  if (target !== PLAN.cacheTarget) throw new Error(`Cache target mismatch: ${target}`);
  // The native downloader retains metadata. Bind its actual selected identity;
  // this does not assert byte equality with a Windows cache seal.
  for (const file of ['info.json']) {
    const full = path.join(server, 'caches', target, file);
    if (fs.existsSync(full)) emit({event: 'cache-metadata', file, sha256: fileHash(full), contents: fs.readFileSync(full, 'utf8')});
  }
  const cacheInfo = JSON.parse(fs.readFileSync(path.join(server, 'caches', target, 'info.json'), 'utf8'));
  if (cacheInfo.id !== PLAN.cacheId) throw new Error(`Cache ID mismatch: ${cacheInfo.id}`);
  const cacheRoot = path.join(server, 'caches', target);
  const cacheManifest = [];
  for (const relative of fs.readdirSync(cacheRoot, {recursive: true}).sort()) {
    const full = path.join(cacheRoot, relative);
    const stat = fs.lstatSync(full);
    if (stat.isSymbolicLink()) throw new Error('Unexpected cache symlink');
    if (!stat.isFile()) continue;
    const digest = crypto.createHash('sha256');
    for await (const chunk of fs.createReadStream(full)) digest.update(chunk);
    cacheManifest.push({file: relative, bytes: stat.size, sha256: digest.digest('hex')});
  }
  emit({event: 'actual-cache-manifest', target, cacheId: cacheInfo.id, files: cacheManifest});
  if (PLAN.expectedCacheFiles) {
    const actual = Object.fromEntries(cacheManifest.map(row => [`caches/${target}/${row.file}`, row]));
    actual['caches/caches.json'] = {sha256: fileHash(path.join(server, 'caches/caches.json'))};
    for (const [file, expected] of Object.entries(PLAN.expectedCacheFiles)) {
      if (actual[file]?.sha256 !== expected) throw new Error(`Cache not equivalent: ${file}`);
    }
    if (Object.keys(actual).length !== Object.keys(PLAN.expectedCacheFiles).length) throw new Error('Cache file count differs from retained input');
    emit({event: 'cache-equivalence', matchedFiles: Object.keys(actual).length, status: 'PASS',
      reference: 'retained c2451c5f per-file inventory; no Windows executable equivalence claim'});
    sealState.cacheFiles = Object.fromEntries(Object.entries(actual).map(([file, row]) => [file, row.sha256]));
  }
  for (const file of PLAN.tests) {
    if (!/^[a-zA-Z0-9_-]+\.test\.cjs$/.test(file)) throw new Error('Bare existing server test filename required');
    emit({event: 'test-input', file, sha256: fileHash(path.join(server, 'tests', file))});
    await stage(`test:${file}`, process.execPath, ['--max-old-space-size=3072', '--test', '--test-concurrency=1', '--test-reporter=tap', `tests/${file}`], server, 600000);
  }
  for (const [file, expected] of Object.entries(expectedInputs)) {
    if (fileHash(path.join(root, file)) !== expected) throw new Error(`Post-run source changed: ${file}`);
  }
  if (!captureSeals(false)) throw new Error('Post-run content seal mismatch');
  emit({event: 'native-qualification-result', source: PLAN.source,
    status: 'TESTS_PASS_AWAIT_SANDBOX_TERMINAL', resources: resources(),
    closureScope: 'process groups checked; full descendant closure requires platform containment and terminal receipt',
    scope: 'Linux fresh dependency/build/cache/test route; no Windows equivalence, client rendering or concurrent capacity claim'});
})().catch(error => {
  // Retain the original nonzero/unknown outcome. Post-failure identity checks
  // neither catch a target assertion as success nor initiate another payload.
  captureSeals(true);
  emit({event: 'native-qualification-result', source: PLAN.source,
    status: error.outcome ?? 'SETUP_FAILURE', error: error.message, resources: resources()});
  process.exitCode = error.exitCode ?? 70;
});
