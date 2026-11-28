// Open only a local test page, in a new temporary headless profile.
const fs = require('node:fs');
const path = require('node:path');
const {spawn} = require('node:child_process');
const {pathToFileURL} = require('node:url');
const {setTimeout: sleep} = require('node:timers/promises');
const WS = require((process.env.TSPS_CHECKOUT || 'tsps') + '/client/node_modules/ws');
const profile = fs.mkdtempSync(path.join(__dirname,'edge-profile-'));
const browser = spawn((process.env.EDGE || 'msedge'), [
  '--headless=new','--remote-debugging-port=0','--disable-background-networking','--disable-sync',
  '--no-first-run','--disable-component-update','--host-resolver-rules=MAP * ~NOTFOUND',
  '--user-data-dir='+profile,pathToFileURL(path.join(__dirname,'playback.html')).href
],{stdio:'ignore',windowsHide:true});
async function main(){
  let ws,port;
  for(let i=0;i<100;i++){
    await sleep(120);
    const portFile=path.join(profile,'DevToolsActivePort');
    if(fs.existsSync(portFile)){port=fs.readFileSync(portFile,'utf8').split(/\r?\n/)[0]; break;}
  }
  const tabs=await fetch('http://127.0.0.1:'+port+'/json/list').then(r=>r.json());
  ws=new WS(tabs.find(t=>t.url.endsWith('/playback.html')).webSocketDebuggerUrl);
  await new Promise((resolve,reject)=>{ws.once('open',resolve);ws.once('error',reject)});
  let id=0; const pending=new Map();
  ws.on('message',msg=>{const m=JSON.parse(msg);if(pending.has(m.id)){const [ok,no]=pending.get(m.id);pending.delete(m.id);m.error?no(m.error):ok(m.result)}});
  function call(method,params={}){return new Promise((ok,no)=>{pending.set(++id,[ok,no]);ws.send(JSON.stringify({id,method,params}))})}
  try{
    await call('Emulation.setDeviceMetricsOverride',{width:600,height:326,deviceScaleFactor:1,mobile:false});
    const state=await call('Runtime.evaluate',{expression:'document.images[0].decode().then(()=>[document.images[0].naturalWidth,document.images[0].naturalHeight])',awaitPromise:true,returnByValue:true});
    if(String(state.result.value)!=='600,326') throw Error(JSON.stringify(state));
    for(let i=0;i<2;i++){
      await sleep(i?780:80);
      const {data}=await call('Page.captureScreenshot',{format:'png'});
      fs.writeFileSync(path.join(__dirname,'edge-frame-'+i+'.png'),Buffer.from(data,'base64'));
    }
    console.log((await call('Browser.getVersion')).product);
  } finally {await call('Browser.close').catch(()=>{});ws.close()}
}
main().catch(e=>{console.error(e);process.exitCode=1;browser.kill()}).finally(async()=>{
  for(let i=0;i<50 && browser.exitCode===null;i++) await sleep(100);
  fs.rmSync(profile,{recursive:true,force:true,maxRetries:8,retryDelay:200});
});
