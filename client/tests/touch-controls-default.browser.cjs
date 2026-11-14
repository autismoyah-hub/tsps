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
  return {
    scenario:window.__touch5133Scenario ?? null, pluginCount:candidates.length,
    loggedIn:client?.isLoggedIn?.() === true, gameState:client?.gameState ?? null,
    pluginEnabled:plugin?.isEnabled?.() ?? false, state:plugin?.getUiState?.() ?? null,
    movementAllowed:plugin?.canMove?.() === true,
    padMode:client?.runeLite?.configManager.getConfiguration('touchcontrols','padMode') ?? null,
    media:{coarse:matchMedia('(pointer: coarse)').matches,fine:matchMedia('(pointer: fine)').matches,
      noHover:matchMedia('(hover: none)').matches,points:navigator.maxTouchPoints},
    touch:new URLSearchParams(location.search).get('touch'),
    joystick:{present:!!node,visible:!!node && !!rect && rect.width > 0 && rect.height > 0
      && style?.display !== 'none' && style?.visibility !== 'hidden',
      width:rect?.width ?? 0,height:rect?.height ?? 0},
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
  assert.equal(actual.state.visible,scenario.expected,'TOUCH5133_CONNECTED_DEFAULT_VISIBILITY');
  assert.equal(actual.joystick.present,scenario.expected,'actual HUD node presence');
  assert.equal(actual.joystick.visible,scenario.expected,'actual HUD geometry/visibility');
  return actual;
}

module.exports={scenarios,prepare,verify};
