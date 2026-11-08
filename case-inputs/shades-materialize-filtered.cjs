// HELD source candidate: no provider execution until exact case review.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const { pathToFileURL } = require('node:url');
const sha = (algorithm, bytes) => crypto.createHash(algorithm).update(bytes).digest('hex');
async function materialize(manifest, root, repository) {
  const prefix = 'server/plugins/skills/crafting/';
  if (manifest.schema !== 1 || manifest.excludedPrefix !== prefix || manifest.excludedCount !== 11 ||
      !/^[0-9a-f]{40}$/.test(manifest.source) || repository !== 'autismoyah-hub/tsps') throw Error('profile mismatch');
  const files = Object.keys(manifest.expectedInputs).sort();
  if (files.length !== 5013 || JSON.stringify(files) !== JSON.stringify(Object.keys(manifest.sourceBlobs).sort())) throw Error('manifest membership');
  for (const file of files) {
    if (file.startsWith(prefix) || file.startsWith('/') || file.includes('\\') || file.includes(':') ||
        file.split('/').some(p => !p || p === '.' || p === '..') ||
        !/^[0-9a-f]{64}$/.test(manifest.expectedInputs[file]) || !/^[0-9a-f]{40}$/.test(manifest.sourceBlobs[file])) throw Error('unsafe manifest row');
  }
  fs.mkdirSync(root, { recursive:false });
  let next = 0;
  async function worker() {
    while (next < files.length) {
      const file = files[next++];
      const url = `https://raw.githubusercontent.com/${repository}/${manifest.source}/${file.split('/').map(encodeURIComponent).join('/')}`;
      const response = await fetch(url, {redirect:'error', signal:AbortSignal.timeout(60000)});
      if (!response.ok || response.url !== url) throw Error(`input download failed: ${file} status ${response.status}`);
      const data = Buffer.from(await response.arrayBuffer());
      if (sha('sha256', data) !== manifest.expectedInputs[file] ||
          sha('sha1', Buffer.concat([Buffer.from(`blob ${data.length}\0`), data])) !== manifest.sourceBlobs[file]) throw Error(`input identity mismatch: ${file}`);
      const target = path.join(root, file);
      fs.mkdirSync(path.dirname(target), {recursive:true});
      fs.writeFileSync(target, data, {flag:'wx'});
    }
  }
  await Promise.all(Array.from({length:8}, worker));
  const actual = fs.readdirSync(root, {recursive:true}).filter(p => fs.lstatSync(path.join(root,p)).isFile()).map(p => p.split(path.sep).join('/')).sort();
  if (JSON.stringify(actual) !== JSON.stringify(files)) throw Error('materialized membership mismatch');
  for (const entry of fs.readdirSync(root, {recursive:true})) {
    const stat = fs.lstatSync(path.join(root,entry));
    if (stat.isSymbolicLink() || (!stat.isFile() && !stat.isDirectory())) throw Error('nonregular materialized entry');
  }
  return {event:'filtered-materialized', source:manifest.source, included:files.length, excludedPrefix:prefix, excludedCount:11};
}
module.exports = { materialize };
