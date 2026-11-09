// Source-only controls for the unselected diagnostic candidate: no GAME imports.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),os=require('node:os'),vm=require('node:vm');
const src=fs.readFileSync(path.join(__dirname,'payload-prospective.cjs'),'utf8');
const cut=src.slice(0,src.indexOf('let sealState = null;')).replace('const PLAN = __PLAN__;','const PLAN = {};');
const events=[];const ctx={require,console:{log:s=>events.push(JSON.parse(s))}};vm.createContext(ctx);vm.runInContext(cut+'\nglobalThis.check=verifyScope;',ctx);
const root=fs.mkdtempSync(path.join(os.tmpdir(),'frem-scope-control-'));
try {
 fs.mkdirSync(path.join(root,'server'));
 for(let i=0;i<105;i++)fs.writeFileSync(path.join(root,'server',`PrivatePlayer${i}.json`),'x');
 fs.writeFileSync(path.join(root,'server','tokenLikeSecret_123456789.txt'),'x');
 let failed=false;try{ctx.check(root,{}, {},false)}catch(e){failed=true;assert.equal(e.message,'Scoped source keyset differs')};assert(failed);
 const ev=events.at(-1);assert.equal(ev.addedCount,106);assert.equal(ev.pathsEmitted,false);assert(!JSON.stringify(ev).includes('PrivatePlayer'));assert(!JSON.stringify(ev).includes('tokenLikeSecret'));
 fs.symlinkSync('/tmp',path.join(root,'server','PlayerSecret'));try{ctx.check(root,{}, {},false);assert.fail('symlink accepted')}catch(e){assert.equal(e.message,'Scoped symlink');assert(!e.message.includes('PlayerSecret'))}
 console.log('source-only keyset counts, 106 private-like names and symlink path redaction passed');
} finally {fs.rmSync(root,{recursive:true,force:true})}
