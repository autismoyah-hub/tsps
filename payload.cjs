// Linux qualification of existing upstream commands, not a gameplay harness.
// __PLAN__ is replaced with the frozen JSON plan by pilot.py, using SDK argv.
const PLAN = __PLAN__;
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const zlib = require('node:zlib');
const {spawn} = require('node:child_process');
const root = '/tmp/tsps-native';
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
function scopedManifest(directory, excludeOutputs) {
  const result = {};
  const excluded = new Set(['server/node_modules','server/dist','server/caches','server/.yarn']);
  function walk(relative) {
    if (excludeOutputs && excluded.has(relative)) return;
    const full=path.join(directory,relative), stat=fs.lstatSync(full);
    if (stat.isSymbolicLink()) throw new Error(`Scoped symlink ${relative}`);
    if (stat.isDirectory()) {
      for (const name of fs.readdirSync(full).sort()) walk(`${relative}/${name}`);
    } else if (stat.isFile()) {
      result[relative]={sha256:fileHash(full),mode:(stat.mode & 0o111) ? '100755' : '100644'};
    } else throw new Error(`Scoped non-file ${relative}`);
  }
  walk('server');
  for (const file of ['.yarnrc.yml','package.json','yarn.lock','scripts/run-with-env.mjs']) {
    if (fs.existsSync(path.join(directory,file))) walk(file);
  }
  return result;
}
function verifyScope(directory, inputs, modes, excludeOutputs) {
  const actual=scopedManifest(directory,excludeOutputs), keys=Object.keys(inputs).sort();
  if (JSON.stringify(Object.keys(actual).sort())!==JSON.stringify(keys)) throw new Error('Scoped source keyset differs');
  for (const file of keys) if (actual[file].sha256!==inputs[file] || actual[file].mode!==modes[file]) throw new Error(`Scoped source differs: ${file}`);
  return sha(JSON.stringify(Object.keys(actual).sort().map(file => [file,actual[file]])));
}

let sealState = null;

function captureSeals(afterFailure) {
  if (!sealState) {
    emit({event: 'source-closure', afterFailure, available: false,
      limitation: 'Source extraction did not finish; no post-input seal can be claimed'});
    return false;
  }
  const {originalSource, sourceManifestHash, distManifest, cacheFiles, expectedInputs, sourceModes} = sealState;
  const errors = [];
  let sourceStatus = 'UNAVAILABLE';
  let distStatus = 'UNAVAILABLE';
  let cacheStatus = 'UNAVAILABLE';
  let postSourceHash = null;
  let postDistHash = null;
  const cacheMismatches = [];
  try {
    verifyScope(root, expectedInputs, sourceModes, true);
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

function baselineTest() {
  return new Promise((resolve, reject) => {
    const args = ['--max-old-space-size=3072', '--test', '--test-concurrency=1', '--test-reporter=tap', '--test-name-pattern=the three spell puzzle targets|soft clay makes a camel mask', 'tests/qh728-q032.test.cjs'];
    emit({event:'baseline-start', args});
    const child = spawn(process.execPath, args, {cwd:server, env, detached:true, stdio:['ignore','pipe','pipe']});
    let output='', exceeded=false, timeout=false;
    for (const stream of [child.stdout, child.stderr]) stream.on('data', chunk => {
      process.stdout.write(chunk);
      if (output.length + chunk.length <= 4194304) output += chunk; else exceeded=true;
    });
    const timer=setTimeout(() => { timeout=true; try { process.kill(-child.pid,'SIGKILL'); } catch {} },600000);
    child.once('error', error => {clearTimeout(timer); reject(error);});
    child.once('close',(code, signal) => {
      clearTimeout(timer); let leftover=false;
      try {process.kill(-child.pid,0);leftover=true;process.kill(-child.pid,'SIGKILL');} catch {}
      const block=output.match(/not ok \d+ - soft clay makes a camel mask after the granite head is placed[\s\S]*?(?=\n(?:ok|not ok) \d+ -|\n1\.\.|$)/)?.[0] ?? '';
      const target=block.length>0;
      const causal=/undefined !== 1/.test(block) && /qh728-q032\.test\.cjs/.test(block);
      const pass=/# pass 1$/m.test(output), fail=/# fail 1$/m.test(output);
      const valid=!exceeded&&!timeout&&!leftover&&code===1&&!signal&&target&&causal&&pass&&fail;
      emit({event:'baseline-result',code,signal,timeout,leftover,target,causal,valid,closureScope:'process-group only'});
      if (!valid) reject(new Error('Baseline red unclassified')); else resolve();
    });
  });
}

// SOURCE CANDIDATE: immutable full SHA/Gitblob manifest; no submission logic.
async function resolveInputManifest(plan, fetcher = fetch) {
  if (!plan.expectedInputsRef) {
    if (!plan.expectedInputs || typeof plan.expectedInputs !== 'object') throw Error('Inline input map missing');
    return {expectedInputs:plan.expectedInputs, sourceBlobs:null, provenance:null};
  }
  const crypto = require('node:crypto');
  const fail = message => { throw Error(`Input manifest: ${message}`); };
  const spec = plan.expectedInputsRef;
  const exactKeys = (value, keys) => value && typeof value==='object' && !Array.isArray(value)
    && Object.keys(value).sort().join('\0')===keys.sort().join('\0');
  if (plan.expectedInputs || plan.sourceBlobs) fail('inline/reference ambiguity');
  if (!exactKeys(spec,['schema','repository','artifactCommit','path','sha256','bytes','source','count'])) fail('reference schema');
  if (spec.schema!==1 || spec.repository!==plan.repository || !/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(spec.repository)) fail('repository/schema');
  if (!/^[a-f0-9]{40}$/.test(spec.artifactCommit) || !/^[a-f0-9]{64}$/.test(spec.sha256)
      || spec.source!==plan.source || !/^[a-f0-9]{40}$/.test(spec.source)) fail('immutable identity/source');
  if (!Number.isSafeInteger(spec.bytes) || spec.bytes<1 || spec.bytes>2*1024*1024
      || !Number.isSafeInteger(spec.count) || spec.count<1 || spec.count>10000) fail('bounds');
  const safe = file => typeof file==='string' && file.length>0 && !file.includes('\\') && !file.includes(':')
    && !file.includes('\0') && !file.startsWith('/') && file.split('/').every(x=>x && x!=='.' && x!=='..');
  if (!safe(spec.path) || !/^[A-Za-z0-9_./-]+\.json$/.test(spec.path)) fail('artifact path');
  const url=`https://raw.githubusercontent.com/${spec.repository}/${spec.artifactCommit}/${spec.path}`;
  const response=await fetcher(url,{redirect:'error',signal:AbortSignal.timeout(60000)});
  if (!response.ok) fail(`HTTP ${response.status}`);
  if (response.url && response.url!==url) fail('response URL differs');
  const chunks=[];let count=0;
  for await(const chunk of response.body) {
    count+=chunk.length;if(count>spec.bytes) fail('byte limit exceeded');chunks.push(Buffer.from(chunk));
  }
  const raw=Buffer.concat(chunks), digest=crypto.createHash('sha256').update(raw).digest('hex');
  if(raw.length!==spec.bytes || digest!==spec.sha256) fail('byte identity');
  let doc;try{doc=JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(raw));}catch{fail('UTF8/JSON');}
  if(!exactKeys(doc,['schema','source','tests','expectedTestCounts','expectedInputs','sourceBlobs','sourceModes'])) fail('document schema');
  if(doc.schema!==1 || doc.source!==plan.source || JSON.stringify(doc.tests)!==JSON.stringify(plan.tests)
      || JSON.stringify(doc.expectedTestCounts)!==JSON.stringify(plan.expectedTestCounts??{})) fail('source/test binding');
  const keys=Object.keys(doc.expectedInputs??{}).sort(), blobKeys=Object.keys(doc.sourceBlobs??{}).sort(), modeKeys=Object.keys(doc.sourceModes??{}).sort();
  if(keys.length!==spec.count || keys.join('\0')!==blobKeys.join('\0') || keys.join('\0')!==modeKeys.join('\0')) fail('count/keyset');
  for(const file of keys) if(!safe(file) || !/^[a-f0-9]{64}$/.test(doc.expectedInputs[file])
      || !/^[a-f0-9]{40}$/.test(doc.sourceBlobs[file]) || !['100644','100755'].includes(doc.sourceModes[file])) fail('path/SHA/blob row');
  for(const file of plan.tests??[]) if(!keys.includes(`server/tests/${file}`)) fail('selected test missing');
  return {expectedInputs:doc.expectedInputs,sourceBlobs:doc.sourceBlobs,sourceModes:doc.sourceModes,
    provenance:{schema:1,url,artifactCommit:spec.artifactCommit,path:spec.path,sha256:digest,bytes:raw.length,source:doc.source,count:keys.length}};
}

(async () => {
  if (process.platform !== 'linux') throw new Error('Linux qualification only');
  if (!/^[a-f0-9]{40}$/.test(PLAN.source)) throw new Error('Full immutable Git SHA required');
  if ((!PLAN.expectedInputs && !PLAN.expectedInputsRef) || !PLAN.tests?.length) throw new Error('Frozen source/input/test request required');
  const inputManifest = await resolveInputManifest(PLAN);
  const expectedInputs = inputManifest.expectedInputs;
  if (inputManifest.provenance) emit({event:'input-manifest-reference',...inputManifest.provenance});
  fs.mkdirSync(env.HOME);
  fs.mkdirSync(env.TMPDIR);
  const response = await fetch(`https://codeload.github.com/${PLAN.repository}/tar.gz/${PLAN.source}`, {signal: AbortSignal.timeout(90000)});
  if (!response.ok) throw new Error(`Source HTTP ${response.status}`);
  const chunks = [];
  let bytes = 0;
  for await (const chunk of response.body) {
    bytes += chunk.length;
    if (bytes > 128 * 1024 * 1024) throw new Error('Source exceeds 128 MiB');
    chunks.push(chunk);
  }
  const archive = Buffer.concat(chunks);
  fs.writeFileSync('/tmp/tsps-native.tar.gz', archive);
  fs.mkdirSync(root);
  await stage('extract', 'tar', ['-xzf', '/tmp/tsps-native.tar.gz', '-C', root, '--strip-components=1'], '/tmp', 60000);
  const originalSource = treeManifest(root);
  const sourceManifestHash = sha(JSON.stringify(originalSource));
  const scopeHash=verifyScope(root,expectedInputs,inputManifest.sourceModes,false);
  emit({event:'source-scope',status:'PASS',scopeHash,files:Object.keys(expectedInputs).length});
  sealState = {originalSource, sourceManifestHash, distManifest: null, cacheFiles: null, expectedInputs, sourceModes:inputManifest.sourceModes};
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
    archiveSHA256: sha(archive), sourceManifestHash, sourceFileCount: originalSource.length,
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
  const nativeResponse = await fetch(nativeURL, {signal: AbortSignal.timeout(60000)});
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
  emit({event: 'native-input', url: nativeURL, archiveSHA256: sha(nativeArchive), member,
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
  // Compare the same fixture against the exact parent plugin, restoring it before AFTER.
  const plugin = path.join(server, 'plugins/quests/quests/EnakhrasLament.Quest.js');
  const fixed = fs.readFileSync(plugin);
  if (sha(fixed) !== PLAN.fixedPluginSHA256) throw new Error('Fixed plugin identity');
  const beforeResponse = await fetch(`https://raw.githubusercontent.com/${PLAN.repository}/${PLAN.beforeSource}/server/plugins/quests/quests/EnakhrasLament.Quest.js`, {redirect:'error', signal: AbortSignal.timeout(60000)});
  if (!beforeResponse.ok) throw new Error(`Baseline HTTP ${beforeResponse.status}`);
  const before = Buffer.from(await beforeResponse.arrayBuffer());
  if (sha(before) !== PLAN.beforePluginSHA256) throw new Error('Baseline plugin identity');
  const oldGate = 'quest.getStage(player) >= STAGE_COMPLETE && hasBit(player, BIT_MOULD)';
  const newGate = 'quest.getStage(player) >= STAGE_PEDESTAL_DONE';
  if (before.toString().split(oldGate).length !== 2 || before.toString().replace(oldGate, newGate) !== fixed.toString()) throw new Error('Baseline delta differs');
  fs.writeFileSync(plugin, before);
  try {
    await baselineTest();
  } finally {
    fs.writeFileSync(plugin, fixed);
  }
  if (fileHash(plugin) !== PLAN.fixedPluginSHA256) throw new Error('Fixed plugin restore failed');
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
