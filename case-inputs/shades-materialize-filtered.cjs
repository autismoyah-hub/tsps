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
  'c8e46107c504610a46366bd2ef1315e68c2e2fbd': '2204952cbd4300ec018e1a24564bba502fcb219bec5dca2f6bbd7e8c6f02b1db',
  '8b018e0871b41e5758ffba49d6763c05fdb6d92c': 'dec09e0788d45a43d4039f8a161eb3329f83b66191d6dbdc77556e2795fb55fb',
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
  if (files.length !== 5073 || JSON.stringify(files) !== JSON.stringify(Object.keys(manifest.sourceBlobs).sort())) throw Error('manifest membership');
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
