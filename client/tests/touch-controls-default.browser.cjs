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

// One optional post-capture observation for phone-default, never pixel proof.
const paintStyles=['display','visibility','opacity','position','z-index','transform','transform-style','isolation','contain','content-visibility','overflow-x','overflow-y','clip','clip-path','mask-image','filter','backdrop-filter','mix-blend-mode','background-color','border-top-color','border-top-width','box-shadow','width','height'];
async function paintSnapshotDiagnostic(cdp, capture, emit, {budgetMs=5000}={}) {
  const crypto=require('node:crypto');
  const report={status:'PENDING',budgetMs,requests:[],identities:[],nodes:[],
    scope:'Post-capture layout flush to compositing inputs clean; non-atomic observations and enclosing paint-layer order, not raster or visible-HUD proof'};
  let usedMs=0,handlesMayExist=false;
  const now=()=>Number(process.hrtime.bigint())/1e6;
  const group='touch5133-paint-snapshot';
  const fail=code=>{throw Object.assign(new Error('TOUCH5133_PAINT_'+code),{paintReason:code});};
  const need=(ok,code)=>{if(!ok)fail(code);};
  const digest=value=>crypto.createHash('sha256').update(value).digest('hex');
  const safeUrl=value=>{const u=new URL(value);u.username='';u.password='';u.hash='';for(const k of [...u.searchParams.keys()])if(!['touch','renderer','username'].includes(k))u.searchParams.set(k,'***');return u.href;};
  async function request(method,params={},cleanup=false) {
    const remaining=Math.floor(budgetMs-usedMs-(cleanup?0:Math.min(500,budgetMs)));
    need(remaining>0&&report.requests.length<12,'REQUEST_BUDGET');
    const record={method,params,requestId:null,responseId:null};report.requests.push(record);
    const started=now();
    try {
      const reply=await cdp.send(method,params,remaining,id=>{record.requestId=id;},method==='DOMSnapshot.captureSnapshot');
      record.responseId=reply.id??null;
      need(Number.isSafeInteger(record.requestId)&&record.requestId>0&&reply.id===record.requestId,'RESPONSE_ID');
      if(reply.snapshotTransport)record.transport=reply.snapshotTransport;
      if(reply.error){record.protocolError={code:Number.isInteger(reply.error.code)?reply.error.code:null,messageSHA256:digest(String(reply.error.message??''))};fail('PROTOCOL_ERROR');}
      if(method==='DOMSnapshot.captureSnapshot')need(record.transport?.status==='BOUNDED_REPLY'&&Number.isSafeInteger(record.transport.bytes)&&record.transport.bytes>0&&record.transport.bytes<=4194304&&/^[a-f0-9]{64}$/.test(record.transport.sha256),'RAW_REPLY_GUARD');
      return reply.result;
    } catch(error) {
      if(error.snapshotTransport)record.transport=error.snapshotTransport;
      record.error=error.paintReason??'TRANSPORT_REFUSAL';throw error;
    } finally {record.elapsedMs=now()-started;usedMs+=record.elapsedMs;}
  }
  async function identity() {
    const target=await request('Target.getTargetInfo'),frames=await request('Page.getFrameTree');
    const t=target?.targetInfo,f=frames?.frameTree?.frame;
    need(t?.type==='page'&&typeof t.targetId==='string'&&typeof t.url==='string'&&f?.id&&f.loaderId&&typeof f.url==='string','PAGE_IDENTITY');
    need(t.url===f.url,'PAGE_URL');
    need([t.targetId,f.id,f.loaderId].every(x=>typeof x==='string'&&Buffer.byteLength(x)<=256)&&Buffer.byteLength(f.url)<=2048,'IDENTITY_LIMIT');
    return {targetId:t.targetId,frameId:f.id,loaderId:f.loaderId,url:f.url};
  }
  function safeIdentity(x){return {...x,url:safeUrl(x.url)};}
  function subset(snapshot, ids, before) {
    need(Array.isArray(snapshot?.documents)&&snapshot.documents.length>0&&snapshot.documents.length<=8&&Array.isArray(snapshot.strings)&&snapshot.strings.length<=32768,'TABLE_LIMIT');
    const strings=snapshot.strings;
    // Never serialize this table, attributes, node text, or input values.
    const text=(i,empty=false)=>{if(empty&&i===-1)return '';need(Number.isInteger(i)&&i>=0&&i<strings.length&&typeof strings[i]==='string','STRING_INDEX');return strings[i];};
    let totalNodes=0,totalLayouts=0;
    for(const d of snapshot.documents){need(Array.isArray(d.nodes?.backendNodeId)&&Array.isArray(d.layout?.nodeIndex),'TABLE_SCHEMA');totalNodes+=d.nodes.backendNodeId.length;totalLayouts+=d.layout.nodeIndex.length;}
    need(totalNodes<=8192&&totalLayouts<=8192,'TABLE_LIMIT');
    const docs=snapshot.documents.filter(d=>text(d.frameId)===before.frameId);
    need(docs.length===1,'DOCUMENT_IDENTITY');const doc=docs[0],nodes=doc.nodes,layout=doc.layout;
    need(text(doc.documentURL)===before.url,'DOCUMENT_URL');
    const n=nodes.backendNodeId.length,l=layout.nodeIndex.length;
    need(Array.isArray(nodes.parentIndex)&&nodes.parentIndex.length===n&&Array.isArray(nodes.nodeName)&&nodes.nodeName.length===n&&Array.isArray(nodes.nodeType)&&nodes.nodeType.length===n&&Array.isArray(nodes.attributes)&&nodes.attributes.length===n,'NODE_SCHEMA');
    need([layout.styles,layout.bounds,layout.paintOrders].every(x=>Array.isArray(x)&&x.length===l)&&Array.isArray(layout.stackingContexts?.index),'LAYOUT_SCHEMA');
    const stacking=new Set(layout.stackingContexts.index);need(stacking.size===layout.stackingContexts.index.length&&[...stacking].every(x=>Number.isInteger(x)&&x>=0&&x<l),'STACKING_INDEX');
    const selected=new Set(),roles=new Map();
    need(new Set(Object.values(ids)).size===3,'DISTINCT_BACKENDS');
    for(const [role,id]of Object.entries(ids)) {
      let matches=0,index=-1;
      for(const d of snapshot.documents)for(let i=0;i<d.nodes.backendNodeId.length;i++)if(d.nodes.backendNodeId[i]===id){matches++;if(d===doc)index=i;}
      need(matches===1&&index>=0,'BACKEND_IDENTITY');roles.set(index,role);
      need(text(nodes.nodeName[index])===(role==='knob'?'SPAN':role==='canvas'?'CANVAS':'DIV'),'NODE_TAG');
      const attrs=nodes.attributes[index];need(Array.isArray(attrs)&&attrs.length<=128&&attrs.length%2===0,'NODE_ATTRIBUTES');
      const attr=name=>{let found=[];for(let a=0;a<attrs.length;a+=2)if(text(attrs[a])===name)found.push(text(attrs[a+1]));need(found.length<=1,'DUPLICATE_ATTRIBUTE');return found[0];};
      if(role==='joystick')need(attr('aria-label')==='Movement joystick'&&attr('class')?.split(/\s+/).includes('touch-joystick'),'JOYSTICK_IDENTITY');
      if(role==='knob')need(attr('class')?.split(/\s+/).includes('touch-joystick-knob'),'KNOB_IDENTITY');
      let current=index;const chain=new Set();
      while(current!==-1){need(Number.isInteger(current)&&current>=0&&current<n&&!chain.has(current)&&chain.size<32,'ANCESTOR_LIMIT');chain.add(current);selected.add(current);need(selected.size<=64,'NODE_LIMIT');current=nodes.parentIndex[current];}
      need(nodes.nodeType[[...chain].at(-1)]===9,'DOCUMENT_ANCESTRY');
    }
    const targetIndexes=Object.fromEntries([...roles].map(([index,role])=>[role,index]));
    need(nodes.parentIndex[targetIndexes.knob]===targetIndexes.joystick,'KNOB_PARENT');
    const rowMap=new Map();
    for(let i=0;i<l;i++){const ni=layout.nodeIndex[i];need(Number.isInteger(ni)&&ni>=0&&ni<n,'LAYOUT_NODE_INDEX');if(selected.has(ni)){const rows=rowMap.get(ni)??[];rows.push(i);need(rows.length<=4,'LAYOUT_FRAGMENT_LIMIT');rowMap.set(ni,rows);}}
    let retainedLayouts=0;
    for(const index of [...selected].sort((a,b)=>a-b)) {
      const tag=text(nodes.nodeName[index]);need(/^(?:[A-Z][A-Z0-9-]{0,31}|#document)$/.test(tag),'TAG_LIMIT');
      const rows=rowMap.get(index)??[];
      need(Number.isSafeInteger(nodes.backendNodeId[index])&&nodes.backendNodeId[index]>0,'ANCESTOR_BACKEND_ID');
      // An absent row is new diagnostic information, never a visibility verdict.
      const record={role:roles.get(index)??'ancestor',nodeIndex:index,backendNodeId:nodes.backendNodeId[index],tag,parentIndex:nodes.parentIndex[index],layoutPresent:rows.length>0,layouts:[]};
      for(const li of rows){need(++retainedLayouts<=128,'RETAINED_LAYOUT_LIMIT');const bounds=layout.bounds[li],styles=layout.styles[li],order=layout.paintOrders[li];
        need(Array.isArray(bounds)&&bounds.length===4&&bounds.every(x=>Number.isFinite(x)&&Math.abs(x)<=1e7)&&Number.isInteger(order)&&order>=0,'LAYOUT_VALUES');
        need(Array.isArray(styles)&&(styles.length===paintStyles.length||(nodes.nodeType[index]===9&&styles.length===0)),'STYLE_SCHEMA');
        const values={};for(let k=0;k<styles.length;k++){const value=text(styles[k],true);need(Buffer.byteLength(value)<=256,'STYLE_VALUE_LIMIT');values[paintStyles[k]]=/url\s*\(/i.test(value)?{redacted:true,sha256:digest(value),bytes:Buffer.byteLength(value)}:value;}
        record.layouts.push({layoutIndex:li,bounds,paintOrder:order,stackingContext:stacking.has(li),styles:values});
      }
      report.nodes.push(record);
    }
    const geometry={};for(const key of ['scrollOffsetX','scrollOffsetY','contentWidth','contentHeight']){const value=doc[key];need(value===undefined||(Number.isFinite(value)&&Math.abs(value)<=1e7),'DOCUMENT_GEOMETRY');geometry[key]=value??null;}
    report.document={frameId:before.frameId,url:safeUrl(before.url),nodeCount:n,layoutCount:l,coordinateSpace:'document CSS pixels',...geometry};
  }
  try {
    need(Number.isInteger(budgetMs)&&budgetMs>0&&budgetMs<=5000,'REQUEST_BUDGET');
    need(capture?.status==='OBSERVED_NOT_VISUALLY_ACCEPTED'&&capture.images?.length===2&&capture.images[0].kind==='canonical-default-surface'&&capture.images[1].kind==='diagnostic-view','CAPTURES_REQUIRED');
    const observed=capture.observations?.at(-1);need(observed?.actual?.scenario?.name==='phone-default'&&observed.actual.state?.visible===true,'POSITIVE_PHONE_ONLY');
    const before=await identity();report.before=safeIdentity(before);
    need(before.targetId===observed.target.id&&before.frameId===observed.frame.id&&before.loaderId===observed.frame.loaderId&&safeUrl(before.url)===safeUrl(observed.frame.url),'CAPTURE_DOCUMENT_LINK');
    const ids={};
    const expressions={joystick:'document.querySelector(\'[aria-label="Movement joystick"]\')',knob:'document.querySelector(\'[aria-label="Movement joystick"]\')?.querySelector(\'.touch-joystick-knob\')',canvas:'(window.osrsClient??window.__osrsClient??window.__WEBGPU_RENDERER__?.osrsClient)?.renderer?.canvas'};
    for(const [role,expression]of Object.entries(expressions)) {
      handlesMayExist=true;
      const result=await request('Runtime.evaluate',{expression,objectGroup:group,returnByValue:false,awaitPromise:false});
      need(!result?.exceptionDetails&&result?.result?.subtype==='node'&&typeof result.result.objectId==='string'&&Buffer.byteLength(result.result.objectId)<=256,'NODE_HANDLE');
      const description=await request('DOM.describeNode',{objectId:result.result.objectId,depth:0,pierce:false});
      need(Number.isSafeInteger(description?.node?.backendNodeId)&&description.node.backendNodeId>0,'BACKEND_ID');ids[role]=description.node.backendNodeId;
      report.identities.push({role,backendNodeId:ids[role],objectIdSHA256:digest(result.result.objectId),evaluateRequestId:report.requests.at(-2).requestId,describeRequestId:report.requests.at(-1).requestId});
    }
    const snapshot=await request('DOMSnapshot.captureSnapshot',{computedStyles:paintStyles,includePaintOrder:true,includeDOMRects:true});
    subset(snapshot,ids,before);
    const after=await identity();report.after=safeIdentity(after);
    need(before.targetId===after.targetId&&before.frameId===after.frameId&&before.loaderId===after.loaderId&&before.url===after.url,'DOCUMENT_CHANGED');
    report.status='OBSERVED_NOT_RASTER_PROOF';
  }catch(error){report.status='REFUSED';report.error=error.paintReason??'TRANSPORT_REFUSAL';}
  finally {
    if(handlesMayExist)try{await request('Runtime.releaseObjectGroup',{objectGroup:group},true);report.handlesReleased=true;}catch{report.handlesReleased=false;report.status='REFUSED';report.cleanupError='HANDLE_RELEASE_REFUSAL';}
    else report.handlesReleased=true;
  }
  report.usedMs=usedMs;
  if(Buffer.byteLength(JSON.stringify({event:'touch5133-paint-snapshot',...report}))>65536){report.nodes=[];report.status='REFUSED';report.error='OUTPUT_LIMIT';}
  emit({event:'touch5133-paint-snapshot',...report});return report;
}

// CPU replay of ONE uniquely backend-linked content picture, not page pixels.
async function layerPictureDiagnostic(cdp,capture,paint,out,emit,{budgetMs=5000}={}) {
  const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
  const report={status:'PENDING',budgetMs,requests:[],
    scope:'Later full lifecycle flush; layer-local CPU paint replay may omit external opacity/transform effects; not composited page pixels or an explanation of earlier captures'};
  const now=()=>Number(process.hrtime.bigint())/1e6;
  const fail=code=>{throw Object.assign(new Error('TOUCH5133_LAYER_'+code),{layerReason:code});};
  const need=(ok,code)=>{if(!ok)fail(code);};
  const hash=value=>crypto.createHash('sha256').update(value).digest('hex');
  const safeUrl=value=>{const u=new URL(value);u.username='';u.password='';u.hash='';for(const k of [...u.searchParams.keys()])if(!['touch','renderer','username'].includes(k))u.searchParams.set(k,'***');return u.href;};
  let usedMs=0,begun=false,enableAttempted=false,snapshotId=null;
  const reserve=Math.min(1000,budgetMs);
  async function request(method,params={},cleanupSlots=0) {
    if(!cleanupSlots)cdp.checkLayerPictureObservation();
    const remaining=Math.floor((budgetMs-usedMs-(cleanupSlots?0:reserve))/(cleanupSlots||1));
    need(remaining>0&&report.requests.length<9,'REQUEST_BUDGET');
    const record={method,params,requestId:null,responseId:null};report.requests.push(record);const started=now();
    try{
      const reply=await cdp.send(method,params,remaining,id=>{record.requestId=id;},false,true);
      record.responseId=reply.id??null;need(Number.isSafeInteger(record.requestId)&&record.requestId>0&&reply.id===record.requestId,'RESPONSE_ID');
      const t=reply.layerPictureTransport;need(t?.status==='BOUNDED_REPLY'&&Number.isSafeInteger(t.bytes)&&t.bytes>0&&t.bytes<=4194304&&/^[a-f0-9]{64}$/.test(t.sha256),'REPLY_GUARD');record.transport={status:t.status,bytes:t.bytes,sha256:t.sha256};
      if(reply.error){record.protocolError={code:Number.isInteger(reply.error.code)?reply.error.code:null,messageSHA256:hash(String(reply.error.message??''))};fail('PROTOCOL_ERROR');}
      return reply.result;
    }catch(error){if(error.layerPictureTransport)record.transport=error.layerPictureTransport;record.error=error.layerReason??'TRANSPORT_REFUSAL';throw error;}
    finally{record.elapsedMs=now()-started;usedMs+=record.elapsedMs;}
  }
  async function identity(){
    const t=(await request('Target.getTargetInfo'))?.targetInfo,f=(await request('Page.getFrameTree'))?.frameTree?.frame;
    need(t?.type==='page'&&[t.targetId,f?.id,f?.loaderId].every(x=>typeof x==='string'&&Buffer.byteLength(x)<=256)&&typeof t.url==='string'&&t.url===f.url&&Buffer.byteLength(t.url)<=2048,'DOCUMENT_IDENTITY');
    return{targetId:t.targetId,frameId:f.id,loaderId:f.loaderId,url:safeUrl(t.url),rawURLSHA256:hash(t.url)};
  }
  try{
    need(Number.isInteger(budgetMs)&&budgetMs>0&&budgetMs<=5000,'REQUEST_BUDGET');
    const obs=capture?.observations?.at(-1);
    need(capture?.status==='OBSERVED_NOT_VISUALLY_ACCEPTED'&&capture.images?.length===2&&capture.images[0].kind==='canonical-default-surface'&&capture.images[1].kind==='diagnostic-view'&&obs?.actual?.scenario?.name==='phone-default'&&obs.actual.state?.visible===true,'POSITIVE_CAPTURES_REQUIRED');
    need(paint?.status==='OBSERVED_NOT_RASTER_PROOF'&&paint.handlesReleased===true,'PAINT_REQUIRED');
    const roles=['joystick','knob'].map(role=>{const nodes=paint.nodes?.filter(x=>x.role===role);need(nodes?.length===1&&Number.isSafeInteger(nodes[0].backendNodeId)&&nodes[0].backendNodeId>0&&nodes[0].layouts?.length===1,'PAINT_NODE_LINK');return nodes[0];});
    need(roles[0].backendNodeId!==roles[1].backendNodeId,'DISTINCT_BACKENDS');
    cdp.beginLayerPictureObservation();begun=true;
    const before=await identity();report.before=before;
    need(before.targetId===paint.after?.targetId&&before.frameId===paint.after.frameId&&before.loaderId===paint.after.loaderId&&before.url===safeUrl(paint.after.url)&&before.frameId===paint.document?.frameId&&before.url===safeUrl(paint.document.url)&&before.targetId===obs.target.id&&before.frameId===obs.frame.id&&before.loaderId===obs.frame.loaderId&&before.url===safeUrl(obs.frame.url),'PAINT_CAPTURE_DOCUMENT_LINK');
    enableAttempted=true;await request('LayerTree.enable');
    const waitMs=Math.floor(budgetMs-usedMs-reserve);need(waitMs>0,'EVENT_BUDGET');const eventStart=now();let event;
    try{event=await cdp.waitLayerPictureEvent(waitMs);}finally{report.eventWaitMs=now()-eventStart;usedMs+=report.eventWaitMs;}
    cdp.checkLayerPictureObservation();
    need(event?.transport&&event.transport.bytes>0&&event.transport.bytes<=65536&&/^[a-f0-9]{64}$/.test(event.transport.sha256),'TREE_EVENT_GUARD');
    const layers=event.params?.layers;need(Array.isArray(layers)&&layers.length>0&&layers.length<=128,'LAYER_TABLE_LIMIT');
    const validId=x=>typeof x==='string'&&/^[0-9]{1,32}$/.test(x);
    need(layers.every(x=>x&&validId(x.layerId)&&(x.parentLayerId===undefined||validId(x.parentLayerId))&&typeof x.drawsContent==='boolean'&&Number.isFinite(x.width)&&Number.isFinite(x.height)&&x.width>=0&&x.height>=0),'LAYER_SCHEMA');
    need(new Set(layers.map(x=>x.layerId)).size===layers.length,'DUPLICATE_LAYER_ID');
    const candidates=layers.filter(x=>roles.some(n=>n.backendNodeId===x.backendNodeId)&&x.drawsContent&&x.invisible!==true);
    need(candidates.length===1,'NO_UNIQUE_CONTENT_LAYER');const layer=candidates[0],node=roles.find(n=>n.backendNodeId===layer.backendNodeId),bounds=node.layouts[0].bounds;
    need(Array.isArray(bounds)&&bounds.length===4&&bounds.every(Number.isFinite),'PAINT_BOUNDS');
    need(Number.isInteger(layer.width)&&Number.isInteger(layer.height)&&layer.width>0&&layer.height>0&&layer.width<=1280&&layer.height<=600&&layer.width===Math.ceil(bounds[2])&&layer.height===Math.ceil(bounds[3]),'SHARED_OR_UNBOUNDED_LAYER');
    // No ancestor fallback or unrelated content within this selected subtree.
    for(const other of layers){let parent=other.parentLayerId,seen=new Set();while(parent!==undefined){need(!seen.has(parent)&&seen.size<32,'LAYER_ANCESTRY');seen.add(parent);if(parent===layer.layerId&&other.drawsContent)need(roles.some(n=>n.backendNodeId===other.backendNodeId),'SHARED_CONTENT');const ancestor=layers.find(x=>x.layerId===parent);need(ancestor,'LAYER_PARENT_MISSING');parent=ancestor.parentLayerId;}}
    const transform=layer.transform??null;need(transform===null||(Array.isArray(transform)&&transform.length===16&&transform.every(x=>Number.isFinite(x)&&Math.abs(x)<=1e7)),'LAYER_TRANSFORM');
    need([layer.offsetX,layer.offsetY].every(x=>Number.isFinite(x)&&Math.abs(x)<=1e7)&&Number.isInteger(layer.paintCount)&&layer.paintCount>=0,'LAYER_GEOMETRY');
    report.tree={transport:event.transport,totalLayers:layers.length};
    report.layer={layerId:layer.layerId,parentLayerId:layer.parentLayerId??null,role:node.role,backendNodeId:layer.backendNodeId,width:layer.width,height:layer.height,offsetX:layer.offsetX,offsetY:layer.offsetY,transform,paintCount:layer.paintCount,drawsContent:true};
    const made=await request('LayerTree.makeSnapshot',{layerId:layer.layerId});
    need(typeof made?.snapshotId==='string'&&/^[0-9]{1,32}$/.test(made.snapshotId),'NO_PICTURE');snapshotId=made.snapshotId;report.snapshotId=snapshotId;
    const replay=await request('LayerTree.replaySnapshot',{snapshotId,scale:1});
    need(typeof replay?.dataURL==='string'&&replay.dataURL.startsWith('data:image/png;base64,'),'REPLAY_PNG');
    const encoded=replay.dataURL.slice(22);need(encoded.length>0&&encoded.length<=2796204&&encoded.length%4===0&&/^[A-Za-z0-9+/]+={0,2}$/.test(encoded),'IMAGE_BYTE_LIMIT');
    const bytes=Buffer.from(encoded,'base64');need(bytes.length<=2097152&&bytes.length>=24&&bytes.subarray(0,8).equals(Buffer.from([137,80,78,71,13,10,26,10]))&&bytes.toString('ascii',12,16)==='IHDR','PNG_HEADER');
    const width=bytes.readUInt32BE(16),height=bytes.readUInt32BE(20);need(width===layer.width&&height===layer.height,'PNG_DIMENSIONS');
    const after=await identity();report.after=after;need(JSON.stringify(before)===JSON.stringify(after),'DOCUMENT_CHANGED');cdp.checkLayerPictureObservation();
    const file=out+'.layer.png';fs.writeFileSync(file,bytes,{flag:'wx'});
    report.image={kind:'cpu-layer-picture-not-canonical',path:path.resolve(file),bytes:bytes.length,sha256:hash(bytes),width,height,base64Chars:encoded.length,base64SHA256:hash(encoded)};
    report.status='OBSERVED_CPU_LAYER_PICTURE_NOT_CANONICAL_PROOF';
  }catch(error){report.status='REFUSED';report.error=error.layerReason??'TRANSPORT_REFUSAL';}
  finally{
    if(snapshotId)try{await request('LayerTree.releaseSnapshot',{snapshotId},2);report.snapshotReleased=true;}catch{report.snapshotReleased=false;report.status='REFUSED';report.releaseError='RELEASE_REFUSAL';report.error??='RELEASE_REFUSAL';}
    else report.snapshotReleased=null;
    if(enableAttempted)try{await request('LayerTree.disable',{},1);report.disabled=true;}catch{report.disabled=false;report.status='REFUSED';report.disableError='DISABLE_REFUSAL';report.error??='DISABLE_REFUSAL';}
    else report.disabled=null;
    if(begun){try{cdp.checkLayerPictureObservation();}catch{report.status='REFUSED';report.observerError='TRANSPORT_REFUSAL';report.error??='TRANSPORT_REFUSAL';}report.observer=cdp.endLayerPictureObservation();}
  }
  report.usedMs=usedMs;
  if(Buffer.byteLength(JSON.stringify({event:'touch5133-layer-picture',...report}))>16384){delete report.layer;delete report.before;delete report.after;report.status='REFUSED';report.error='OUTPUT_LIMIT';}
  emit({event:'touch5133-layer-picture',...report});return report;
}

module.exports={scenarios,prepare,verify,captureDiagnostic,paintSnapshotDiagnostic,layerPictureDiagnostic};
