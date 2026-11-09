// Source-only controls for the unselected diagnostic candidate: no GAME imports.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),os=require('node:os'),vm=require('node:vm');
const src=fs.readFileSync(path.join(__dirname,'payload-prospective.cjs'),'utf8');
const cut=src.slice(0,src.indexOf('let sealState = null;')).replace('const PLAN = __PLAN__;','const PLAN = {};');
const events=[];const ctx={require,console:{log:s=>events.push(JSON.parse(s))}};vm.createContext(ctx);vm.runInContext(cut+'\nglobalThis.check=verifyScope; globalThis.fail=failureEvent; globalThis.summary=cacheSummary; globalThis.report=reportFailure; globalThis.cache=enumerateCache; globalThis.validate=validateCacheRows; globalThis.resources=()=>({});',ctx);
function mustThrow(fn,pattern){let error;try{fn()}catch(e){error=e}assert(error,'expected actual exception');assert.match(error.message,pattern);return error}
(async()=>{
const root=fs.mkdtempSync(path.join(os.tmpdir(),'frem-scope-control-'));
try {
 fs.mkdirSync(path.join(root,'server'));
 for(let i=0;i<105;i++)fs.writeFileSync(path.join(root,'server',`PrivatePlayer${i}.json`),'x');
 fs.writeFileSync(path.join(root,'server','tokenLikeSecret_123456789.txt'),'x');
 let failed=false;try{ctx.check(root,{}, {},false)}catch(e){failed=true;assert.equal(e.message,'Scoped source keyset differs')};assert(failed);
 const ev=events.at(-1);assert.equal(ev.addedCount,106);assert.equal(ev.pathsEmitted,false);assert(!JSON.stringify(ev).includes('PrivatePlayer'));assert(!JSON.stringify(ev).includes('tokenLikeSecret'));
 fs.symlinkSync('/tmp',path.join(root,'server','PlayerSecret'));try{ctx.check(root,{}, {},false);assert.fail('symlink accepted')}catch(e){assert.equal(e.message,'Scoped symlink');assert(!e.message.includes('PlayerSecret'))}
 const originalLstat=fs.lstatSync;fs.lstatSync=()=>{throw new Error('EACCES /server/PrivatePlayer secret')};
 try{const e=mustThrow(()=>ctx.check(root,{}, {},false),/EACCES/);ctx.report(e)}finally{fs.lstatSync=originalLstat}
 assert(!JSON.stringify(events.at(-1)).includes('PrivatePlayer'));assert.equal(events.at(-1).status,'SETUP_FAILURE');
 const cacheDir=path.join(root,'cache');fs.mkdirSync(cacheDir);fs.writeFileSync(path.join(cacheDir,'known'),'x');let rows=await ctx.cache(cacheDir);const hash=rows[0].sha256;const expected={'caches/fixture/known':hash,'caches/caches.json':'meta'};assert(ctx.validate(rows,'meta',expected,'fixture'));
 fs.writeFileSync(path.join(cacheDir,'PrivatePlayer-token'),'x');rows=await ctx.cache(cacheDir);const cacheError=mustThrow(()=>ctx.validate(rows,'meta',expected,'fixture'),/Cache file count differs/);ctx.report(cacheError);
 assert.throws(()=>mustThrow(()=>true,/Cache file count differs/),/expected actual exception/)
 assert(!JSON.stringify(events.at(-1)).includes('PrivatePlayer'));assert.equal(events.at(-1).status,'SETUP_FAILURE');
 assert(!JSON.stringify(events).includes('PrivatePlayer'));assert(!JSON.stringify(events).includes('tokenLikeSecret'));assert(!src.includes("error: error.message"));
 console.log('source-only counts, 106 private-like names, injected fs message and cache name redaction passed');
} finally {fs.rmSync(root,{recursive:true,force:true})}

})().catch(e=>{console.error(e);process.exitCode=1});
