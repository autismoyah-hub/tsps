'use strict';
const assert = require('node:assert/strict');

// Used by the existing authenticated HF/browser executor. No launcher, server,
// credentials, provider calls, or automatic retries live in this fixture.
// Primary capabilities are explicitly emulated before client construction;
// these are not measurements of a physical phone or touchscreen laptop.
const scenarios = [
  {name:'phone-default', pointer:'coarse', hover:false, points:1, mode:'DEFAULT', touch:null, expected:true},
  {name:'laptop-default', pointer:'fine', hover:true, points:10, mode:'DEFAULT', touch:null, expected:false},
  {name:'desktop-default', pointer:'fine', hover:true, points:0, mode:'DEFAULT', touch:null, expected:false},
  {name:'laptop-on', pointer:'fine', hover:true, points:10, mode:'ON', touch:null, expected:true},
  {name:'phone-off', pointer:'coarse', hover:false, points:1, mode:'OFF', touch:null, expected:false},
  {name:'laptop-url-on', pointer:'fine', hover:true, points:10, mode:'DEFAULT', touch:'1', expected:true},
  {name:'phone-url-off', pointer:'coarse', hover:false, points:1, mode:'DEFAULT', touch:'0', expected:false},
  {name:'explicit-off-beats-url-on', pointer:'fine', hover:true, points:10, mode:'OFF', touch:'1', expected:false},
  {name:'explicit-on-beats-url-off', pointer:'coarse', hover:false, points:1, mode:'ON', touch:'0', expected:true},
];

function pageSetup(s) {
  const realMatchMedia = window.matchMedia.bind(window);
  const matches = {
    '(pointer: coarse)':s.pointer === 'coarse', '(pointer: fine)':s.pointer === 'fine',
    '(pointer: none)':s.pointer === 'none', '(hover: none)':!s.hover, '(hover: hover)':s.hover,
    '(pointer: coarse) and (hover: none)':s.pointer === 'coarse' && !s.hover,
  };
  window.matchMedia = query => {
    const result = realMatchMedia(query);
    if (Object.prototype.hasOwnProperty.call(matches,query)) {
      Object.defineProperty(result,'matches',{value:matches[query]});
    }
    return result;
  };
  Object.defineProperty(navigator,'maxTouchPoints',{configurable:true,get:()=>s.points});
  // Present on both disclosed touchscreen profiles, including the laptop.
  if (s.points > 0) Object.defineProperty(window,'ontouchstart',{configurable:true,value:null});
  localStorage.setItem('rl.config.touchcontrols.padMode',s.mode);
  localStorage.setItem('rl.config.runelite.touchcontrolsplugin','true');
  window.__touch5133Scenario = {name:s.name,pointer:s.pointer,hover:s.hover,points:s.points,mode:s.mode,touch:s.touch};
}

async function prepare(cdp, scenario) {
  assert.ok(scenarios.includes(scenario), 'use an exact reviewed scenario');
  const before = await cdp.evaluate('!!(window.osrsClient || window.__osrsClient)');
  assert.equal(before,false,'install capability fixture before client construction/navigation');
  const source = `(${pageSetup.toString()})(${JSON.stringify(scenario)})`;
  const reply = await cdp.send('Page.addScriptToEvaluateOnNewDocument',{source});
  assert.ok(!reply.error,'capability initialization accepted');
  return {source,scenario:scenario.name};
}

function readPage() {
  const client = window.osrsClient ?? window.__osrsClient ?? window.__WEBGPU_RENDERER__?.osrsClient;
  const manager = client?.runeLite?.pluginManager;
  const candidates = manager?.getPlugins().filter(p => manager.getDescriptor(p).configKey === 'touchcontrolsplugin') ?? [];
  const plugin = candidates.length === 1 ? candidates[0] : null;
  const node = document.querySelector('[aria-label="Movement joystick"]');
  const rect = node?.getBoundingClientRect();
  const style = node ? getComputedStyle(node) : null;
  const box = r => r ? {x:r.x,y:r.y,left:r.left,top:r.top,right:r.right,bottom:r.bottom,width:r.width,height:r.height} : null;
  const viewport = {width:innerWidth,height:innerHeight,visual:window.visualViewport ? {
    left:visualViewport.offsetLeft,top:visualViewport.offsetTop,width:visualViewport.width,height:visualViewport.height,scale:visualViewport.scale,
  } : null};
  const clip = {left:0,top:0,right:innerWidth,bottom:innerHeight};
  if (viewport.visual) {
    clip.left = Math.max(clip.left,viewport.visual.left); clip.top = Math.max(clip.top,viewport.visual.top);
    clip.right = Math.min(clip.right,viewport.visual.left+viewport.visual.width);
    clip.bottom = Math.min(clip.bottom,viewport.visual.top+viewport.visual.height);
  }
  const ancestors = []; let paintAllowed = true, parent = node;
  for (; parent && ancestors.length < 32; parent = parent.parentElement) {
    const css = getComputedStyle(parent), bounds = parent.getBoundingClientRect();
    if (css.display === 'none' || css.visibility !== 'visible' || Number(css.opacity) === 0) paintAllowed = false;
    const clipX = /^(hidden|clip|scroll|auto)$/.test(css.overflowX);
    const clipY = /^(hidden|clip|scroll|auto)$/.test(css.overflowY);
    if (parent !== node) {
      if (clipX) { clip.left=Math.max(clip.left,bounds.left); clip.right=Math.min(clip.right,bounds.right); }
      if (clipY) { clip.top=Math.max(clip.top,bounds.top); clip.bottom=Math.min(clip.bottom,bounds.bottom); }
    }
    ancestors.push({tag:parent.tagName,className:String(parent.className).slice(0,160),rect:box(bounds),
      overflowX:css.overflowX,overflowY:css.overflowY,display:css.display,visibility:css.visibility,opacity:css.opacity});
  }
  const inViewport = !!rect && rect.width > 0 && rect.height > 0
    && rect.left >= clip.left && rect.top >= clip.top && rect.right <= clip.right && rect.bottom <= clip.bottom;
  const hit = rect && inViewport ? document.elementFromPoint(rect.left+rect.width/2,rect.top+rect.height/2) : null;
  const hitTarget = !!node && !!hit && node.contains(hit);
  return {
    scenario:window.__touch5133Scenario ?? null, pluginCount:candidates.length,
    loggedIn:client?.isLoggedIn?.() === true, gameState:client?.gameState ?? null,
    pluginEnabled:plugin?.isEnabled?.() ?? false, state:plugin?.getUiState?.() ?? null,
    movementAllowed:plugin?.canMove?.() === true,
    padMode:client?.runeLite?.configManager.getConfiguration('touchcontrols','padMode') ?? null,
    media:{coarse:matchMedia('(pointer: coarse)').matches,fine:matchMedia('(pointer: fine)').matches,
      noHover:matchMedia('(hover: none)').matches,points:navigator.maxTouchPoints},
    touch:new URLSearchParams(location.search).get('touch'),
    joystick:{present:!!node,visible:!!node && inViewport && paintAllowed && !parent && hitTarget,
      width:rect?.width ?? 0,height:rect?.height ?? 0,rect:box(rect),viewport,clip,ancestors,
      ancestryComplete:!parent,inViewport,paintAllowed,hitTarget,
      paintStyle:style ? {background:style.backgroundColor,opacity:style.opacity,border:style.borderColor} : null},
  };
}

async function verify(cdp, scenario, emit, {timeoutMs=10000}={}) {
  const deadline=Date.now()+timeoutMs; let actual;
  do {
    actual=await cdp.evaluate(`(${readPage.toString()})()`);
    if (actual.pluginCount === 1 && actual.loggedIn && actual.gameState === 30 && actual.pluginEnabled && actual.movementAllowed
      && actual.state?.visible === scenario.expected && actual.joystick.visible === scenario.expected) break;
    await new Promise(resolve=>setTimeout(resolve,50));
  } while(Date.now()<deadline);
  emit({event:'touch5133-hud-observed',scenario:scenario.name,actual,
    scope:'real connected plugin and React HUD under pre-construction emulated primary capabilities; not physical hardware proof'});
  assert.equal(actual.pluginCount,1,'exact registered Touch Controls instance');
  assert.equal(actual.loggedIn,true,'actual client logged in'); assert.equal(actual.gameState,30);
  assert.equal(actual.pluginEnabled,true,'plugin enabled, not hidden by disabling plugin');
  assert.equal(actual.movementAllowed,true,'TOUCH5133_PREREQUISITE: actual plugin canMove must permit movement');
  assert.equal(actual.scenario?.name,scenario.name,'actual page fixture identity');
  assert.equal(actual.media.points,scenario.points,'actual emulated touch count');
  assert.equal(actual.media.coarse,scenario.pointer==='coarse'); assert.equal(actual.media.noHover,!scenario.hover);
  assert.equal(actual.media.fine,scenario.pointer==='fine','TOUCH5133_PREREQUISITE: declared primary fine pointer');
  assert.equal(actual.touch,scenario.touch,'literal URL override, not fixture-only expectation');
  assert.equal(actual.padMode,scenario.mode,'actual config storage value');
  if (actual.state.visible || actual.joystick.present) {
    assert.equal(actual.joystick.present,true,'TOUCH5133_PREREQUISITE: visible state has actual HUD node');
    assert.equal(actual.joystick.inViewport,true,'TOUCH5133_PREREQUISITE: HUD is within viewport and ancestor clip bounds');
    assert.equal(actual.joystick.ancestryComplete,true,'TOUCH5133_PREREQUISITE: complete clipping ancestry');
    assert.equal(actual.joystick.paintAllowed,true,'TOUCH5133_PREREQUISITE: HUD and ancestors permit painting');
    assert.equal(actual.joystick.hitTarget,true,'TOUCH5133_PREREQUISITE: HUD center is not occluded');
  }
  assert.equal(actual.state.visible,scenario.expected,'TOUCH5133_CONNECTED_DEFAULT_VISIBILITY');
  assert.equal(actual.joystick.present,scenario.expected,'actual HUD node presence');
  assert.equal(actual.joystick.visible,scenario.expected,'actual HUD geometry/visibility');
  return actual;
}


// Optional observation only: keep the canonical surface image and compare one
// view image of the same unchanged page. Neither image implies visual acceptance.
async function captureDiagnostic(cdp, out, canonical, emit, {budgetMs=5000}={}) {
  const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
  const report={status:'PENDING',budgetMs,requests:[],observations:[],images:[]};
  let usedMs=0;
  const safeUrl=value=>{const u=new URL(value);if(u.searchParams.has('password'))u.searchParams.set('password','***');return u.href;};
  async function request(method,params={}) {
    const remaining=Math.floor(budgetMs-usedMs);
    if(remaining<=0)throw Error('diagnostic observation budget exhausted');
    const record={method,params,requestId:null,responseId:null,startedMs:Date.now()};report.requests.push(record);
    const started=Date.now();
    try {
      const reply=await cdp.send(method,params,remaining,id=>{record.requestId=id;});
      record.responseId=reply.id??null;
      assert.ok(Number.isSafeInteger(record.requestId)&&record.requestId>0&&reply.id===record.requestId,'diagnostic actual CDP request/response identity');
      if(reply.error){record.protocolError=reply.error;throw Error(reply.error.message||'diagnostic protocol refusal');}
      if(typeof reply.result?.data==='string'){record.base64Chars=reply.result.data.length;record.base64SHA256=crypto.createHash('sha256').update(reply.result.data).digest('hex');}
      return reply.result;
    }catch(error){record.error=String(error.message??error);throw error;}
    finally{record.elapsedMs=Date.now()-started;usedMs+=record.elapsedMs;}
  }
  async function observe(label) {
    const target=await request('Target.getTargetInfo');
    const frames=await request('Page.getFrameTree');
    const hud=await request('Runtime.evaluate',{expression:`(${readPage.toString()})()`,returnByValue:true,awaitPromise:true});
    assert.ok(!hud.exceptionDetails&&hud.result?.value,'diagnostic actual HUD observation');
    const info=target.targetInfo,frame=frames.frameTree?.frame;
    assert.ok(info?.targetId&&info.type==='page'&&frame?.id,'diagnostic page/frame identity');
    report.observations.push({label,target:{id:info.targetId,type:info.type,url:safeUrl(info.url)},
      frame:{id:frame.id,loaderId:frame.loaderId,url:safeUrl(frame.url)},actual:hud.result.value});
  }
  function imageRecord(kind,file,bytes) {
    assert.ok(bytes.length>=24&&bytes.subarray(0,8).equals(Buffer.from([137,80,78,71,13,10,26,10]))&&bytes.toString('ascii',12,16)==='IHDR','diagnostic PNG header');
    const width=bytes.readUInt32BE(16),height=bytes.readUInt32BE(20);assert.ok(width>0&&height>0,'diagnostic positive PNG dimensions');
    report.images.push({kind,path:path.resolve(file),bytes:bytes.length,sha256:crypto.createHash('sha256').update(bytes).digest('hex'),width,height});
  }
  let failure=null;
  try{await observe('before-canonical');}catch(error){failure=error;}
  // Metadata refusal must never prevent the unchanged canonical capture/write.
  const canonicalRecord={method:'Page.captureScreenshot',params:{format:'png'},requestId:null,responseId:null,startedMs:Date.now()};
  report.requests.push(canonicalRecord);
  try {
    const reply=await canonical(id=>{canonicalRecord.requestId=id;});
    canonicalRecord.responseId=reply.id??null;
    if(typeof reply.result?.data==='string'){canonicalRecord.base64Chars=reply.result.data.length;canonicalRecord.base64SHA256=crypto.createHash('sha256').update(reply.result.data).digest('hex');}
    assert.ok(Number.isSafeInteger(canonicalRecord.requestId)&&canonicalRecord.requestId>0&&reply.id===canonicalRecord.requestId,'canonical actual CDP request/response identity');
  }catch(error){canonicalRecord.responseId=error.responseId??canonicalRecord.responseId;if(error.protocolError)canonicalRecord.protocolError=error.protocolError;canonicalRecord.error=String(error.message??error);report.status='REFUSED';report.error=canonicalRecord.error;emit({event:'touch5133-capture-diagnostic',...report});throw error;}
  finally{canonicalRecord.elapsedMs=Date.now()-canonicalRecord.startedMs;}
  try {
    imageRecord('canonical-default-surface',out,fs.readFileSync(out));
    if(failure)throw failure;
    await observe('after-canonical-before-view');
    const image=await request('Page.captureScreenshot',{format:'png',fromSurface:false});
    assert.ok(typeof image?.data==='string'&&image.data.length>0,'diagnostic view capture returned pixels');
    const bytes=Buffer.from(image.data,'base64'),file=out+'.view.png';imageRecord('diagnostic-view',file,bytes);
    fs.writeFileSync(file,bytes,{flag:'wx'});
    await observe('after-view');
    const first=report.observations[0];
    assert.ok(report.observations.every(x=>x.target.id===first.target.id&&x.target.url===first.target.url&&x.frame.id===first.frame.id&&x.frame.loaderId===first.frame.loaderId&&x.frame.url===first.frame.url),'diagnostic same target/document across captures');
    assert.ok(report.observations.every(x=>JSON.stringify(x.actual.joystick.viewport)===JSON.stringify(first.actual.joystick.viewport)),'diagnostic observed viewport unchanged');
    report.status='OBSERVED_NOT_VISUALLY_ACCEPTED';
  }catch(error){report.status='REFUSED';report.error=String(error.message??error);}
  report.usedMs=usedMs;
  emit({event:'touch5133-capture-diagnostic',...report});
  return report;
}

module.exports={scenarios,prepare,verify,captureDiagnostic};
