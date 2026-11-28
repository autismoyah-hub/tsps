"""build_v2.py -> fight-v2.json: the Warden-General redesigned to the measured rules (REDESIGN.md explains every field).
Reads fx-warden-general/fight.json (unchanged) for the kept assets and beats. usage: python -I build_v2.py
"""
import json, pathlib, copy
HERE = pathlib.Path(__file__).resolve().parent
SRC = HERE.parent / 'fx-warden-general' / 'fight.json'
old = json.loads(SRC.read_text(encoding='utf-8'))
HP = 900


def rel(a):
    a = copy.deepcopy(a)
    if 'file' in a: a['file'] = '../fx-warden-general/' + a['file']
    return a


assets = {k: rel(v) for k, v in old['assets'].items()}
# New assets: NOT BUILT. Ids are the next free ones after the General's 4180-4186 (to confirm against the cache when built).
assets.update({
    'meridian-line-ns': {'kind': 'spot', 'id': 4187, 'status': 'to build', 'note': 'meridian-line turned 90 degrees (a spotanim cannot be rotated by packet), same 90-cycle warning'},
    'meridian-strike-ns': {'kind': 'spot', 'id': 4188, 'status': 'to build', 'note': 'meridian-strike turned 90 degrees'},
    'sighting-mark': {'kind': 'spot', 'id': 4189, 'status': 'to build', 'note': 'the four-point sight over the marked player, 120 cycles (4 ticks), closing to a point at the volley'},
    'bearing-release': {'kind': 'spot', 'id': 4190, 'status': 'to build', 'note': 'actor graphic on the General when an un-arrested dial feeds him (the heal tell)'},
})
A = copy.deepcopy(old['attacks'])

lance = A['star_lance']
lance['mechanic'] = {'kind': 'auto', 'solve': 'prayer', 'learnable': 'fixed', 'trigger': 'rotation slots 1, 3, 5',
    'solve_text': 'Protect from Missiles before tick 3 (one style all fight: set once)', 'tell': 'pose 20271, the lance in flight', 'tell_ticks': 3,
    'punish_max': 32, 'solved_max': 0, 'punish_other': 'none', 'inputs': 1}

double = copy.deepcopy(old['attacks']['star_lance'])
p2 = copy.deepcopy([b for b in double['beats'] if b['do'] == 'projectile'][0]); p2['tick'] = 1
double['beats'] += [p2, {'do': 'impact', 'tick': 4, 'asset': 'lance-impact', 'on': 'player', 'height': 100},
                    {'do': 'hit', 'tick': 4, 'target': 'player', 'maxHit': 32, 'style': 'ranged'}]
double['speedTicks'] = 4
double['mechanic'] = {'kind': 'enrage', 'solve': 'prayer', 'learnable': 'fixed', 'trigger': 'replaces Star Lance below 20%',
    'solve_text': 'keep Protect from Missiles; two bolts one tick apart', 'tell': 'pose 20271, two lances in flight', 'tell_ticks': 3,
    'punish_max': 64, 'solved_max': 0, 'punish_other': 'none', 'inputs': 0}

line = A['meridian_line']
line['mechanic'] = {'kind': 'special', 'solve': 'move_off_tiles', 'learnable': 'fixed', 'trigger': 'rotation slot 6 (phase 1)',
    'solve_text': 'step one tile north or south off the marked row before tick 3', 'tell': 'pose 20272, seven meridian-line tiles through you', 'tell_ticks': 3,
    'punish_max': 44, 'solved_max': 0, 'punish_other': 'none', 'inputs': 1}

# Meridian Cross: the row plus the column through the player; the diagonals are the only adjacent safe tiles.
cross = {'speedTicks': 4, 'beats': [{'do': 'npc_anim', 'tick': 0, 'seq': 'pose.line'}]}
row = [[dx, 0] for dx in range(-3, 4)]; col = [[0, dy] for dy in range(-3, 4) if dy != 0]
for t in row: cross['beats'].append({'do': 'telegraph', 'tick': 0, 'asset': 'meridian-line', 'tile': t, 'warnTicks': 3})
for t in col: cross['beats'].append({'do': 'telegraph', 'tick': 0, 'asset': 'meridian-line-ns', 'tile': t, 'warnTicks': 3})
for t in row + col:
    cross['beats'].append({'do': 'tile_graphic', 'tick': 3, 'asset': 'meridian-strike' if t[1] == 0 else 'meridian-strike-ns', 'tile': t})
    cross['beats'].append({'do': 'hit', 'tick': 3, 'target': 'tile', 'tile': t, 'radius': 0, 'maxHit': 44, 'style': 'typeless'})
cross['mechanic'] = {'kind': 'special', 'solve': 'move_off_tiles', 'learnable': 'fixed', 'trigger': 'replaces Meridian Line from 50%',
    'solve_text': 'one DIAGONAL step before tick 3 (north or south now lands on the column)', 'tell': 'pose 20272, a cross of meridian lines through you',
    'tell_ticks': 3, 'punish_max': 44, 'solved_max': 0, 'punish_other': 'none', 'inputs': 1}


def dial_attack(tiles, inputs):
    a = {'speedTicks': old['attacks']['orbit']['speedTicks'] if inputs == 1 else 4, 'beats': [{'do': 'npc_anim', 'tick': 0, 'seq': 'pose.orbit'}]}
    for t in tiles:
        a['beats'].append({'do': 'object', 'tick': 1, 'tile': t, 'asset': 'orbit', 'lifetimeTicks': 17, 'tickDamage': [4, 8],
                           'despawn': 'despawn', 'despawnCycles': 30, 'arrest': {'op': 'Arrest', 'from': 'adjacent', 'ticks': 1, 'sequence': 'despawn'}})
    a['beats'].append({'do': 'impact', 'tick': 18, 'asset': 'bearing-release', 'on': 'npc', 'height': 742, 'per': 'dial not arrested'})
    n = len(tiles)
    a['mechanic'] = {'kind': 'add', 'solve': 'interact', 'learnable': 'mixed', 'trigger': 'rotation slot 2',
        'solve_text': f'arrest {"the dial" if n == 1 else "both dials"} (op on the loc, from an adjacent tile, 1 tick each) between the Sighting volley and tick 18; never stand on a dial',
        'tell': 'pose, the dial set on a random side of you ([1,0] or [-1,0]) and turning for 17 ticks', 'tell_ticks': 17,
        'punish_max': 16, 'solved_max': 0, 'punish_note': 'a misstep onto the dial: 4-8 a tick, 2 ticks to step off; the real punishment is the heal',
        'punish_other': f'heal: +{HP // 20} hitpoints ({100 // 20}% of {HP}) per dial released, and his next Star Lance is doubled (enrage)', 'inputs': inputs}
    return a


orbit = dial_attack([[1, 0]], 1)
pair = dial_attack([[1, 0], [-1, 0]], 3)

sight = {'speedTicks': 5, 'beats': [{'do': 'npc_anim', 'tick': 0, 'seq': 'pose.lance'},
                                    {'do': 'impact', 'tick': 0, 'asset': 'sighting-mark', 'on': 'player', 'height': 230}]}
for i in range(3):
    sight['beats'].append({'do': 'projectile', 'tick': 1, 'asset': 'star-lance', 'from': 'npc', 'to': 'player', 'start': 0 + 10 * i, 'end': 90 + 10 * i,
                           'startHeight': 81, 'endHeight': 25, 'angle': 5, 'progress': 0})
sight['beats'].append({'do': 'hit', 'tick': 4, 'target': 'player', 'maxHit': 60, 'style': 'typeless', 'unlessSheltered': 'orbit',
                       'note': '3 bolts x 20; none if a dial stands on the line from his centre to the player'})
sight['mechanic'] = {'kind': 'special', 'solve': 'line_of_sight', 'learnable': 'fixed', 'trigger': 'rotation slot 4, while a dial stands',
    'solve_text': 'by tick 4 stand on the far side of a live dial from him (for a dial at [1,0]: [1,-1] or [1,-2]); prayer does nothing',
    'tell': 'pose 20271 + the sight closing over your head for 4 ticks', 'tell_ticks': 4, 'punish_max': 60, 'solved_max': 0, 'punish_other': 'none', 'inputs': 1}

last = {'speedTicks': 4, 'beats': [{'do': 'npc_anim', 'tick': 0, 'seq': 'pose.phase'},
                                   {'do': 'impact', 'tick': 0, 'asset': 'phase-change', 'on': 'npc', 'height': 742}],
        'mechanic': {'kind': 'phase', 'solve': 'none', 'learnable': 'fixed', 'tell': 'pose 20273 and the phase burst again', 'tell_ticks': 0,
                     'punish_max': 0, 'punish_other': 'none', 'inputs': 0}}
A['phase_change']['mechanic'] = {'kind': 'phase', 'solve': 'none', 'learnable': 'fixed', 'tell': 'pose 20273, burst, then the lit sight', 'tell_ticks': 0,
                                 'punish_max': 0, 'punish_other': 'none', 'inputs': 0}
A['phase_change']['speedTicks'] = 4
attacks = {'star_lance': lance, 'orbit': orbit, 'sighting': sight, 'meridian_line': line, 'phase_change': A['phase_change'],
           'orbit_pair': pair, 'meridian_cross': cross, 'last_bearing': last, 'star_lance_double': double}

P1 = ['star_lance', 'orbit', 'star_lance', 'sighting', 'star_lance', 'meridian_line']
P2 = ['star_lance', 'orbit_pair', 'star_lance', 'sighting', 'star_lance', 'meridian_cross']
P3 = ['star_lance_double', 'orbit_pair', 'star_lance_double', 'sighting', 'star_lance_double', 'meridian_cross']
tl = [[5 * i, a] for i, a in enumerate(P1)]
tl.append([30, 'phase_change']); tl += [[34 + 4 * i, a] for i, a in enumerate(P2)]
tl.append([58, 'last_bearing']); tl += [[62 + 4 * i, a] for i, a in enumerate(P3)]

F = {'name': 'fx-warden-general-v2', 'title': 'Warden-General (v2, to the measured rules)',
     'note': 'Design data, not built. Kept: star-lance, lance-impact, meridian-line, meridian-strike, orbit, phase-change, phase-light, sequences 20271-20273. '
             'New assets have status "to build". Extension fields are documented in research-fights/fightcheck.py.',
     'cache': '../fx-warden-general/cache-own', 'hitpoints': HP, 'player_hitpoints': 99,
     'assets': assets, 'stage': old['stage'],
     'phase': old['phase'],
     'phases': [
         {'name': 'Second bearing', 'trigger': 'first crossing at or below 50% (450)', 'at_hp_pct': 50, 'attack': 'phase_change',
          'adds': ['orbit_pair', 'meridian_cross'], 'removes': ['orbit', 'meridian_line'], 'speeds': {'star_lance': 4, 'sighting': 4},
          'changes': 'rotation every 4 ticks instead of 5; two dials; the row becomes a cross; the lit sight stays on (existing phase-light)'},
         {'name': 'Last bearing', 'trigger': 'first crossing at or below 20% (180)', 'at_hp_pct': 20, 'attack': 'last_bearing',
          'adds': ['star_lance_double'], 'removes': ['star_lance'], 'speeds': {},
          'changes': 'every Star Lance is two bolts one tick apart'}],
     'rotation': {'opening': P1, 'Second bearing': P2, 'Last bearing': P3},
     'overlaps': [
         {'a': 'sighting', 'b': 'orbit', 'conflicting_answers': True,
          'note': 'the dial is the only shelter from the Sighting AND must be arrested before it feeds him: arrest it before the volley (tick 19) and the volley hits; after tick 22 and he heals'},
         {'a': 'sighting', 'b': 'orbit_pair', 'conflicting_answers': True,
          'note': 'as above with two dials: shelter behind one, then arrest both inside a 5-tick window'},
         {'a': 'orbit_pair', 'b': 'meridian_cross', 'conflicting_answers': False,
          'note': 'the cross is cast while the second dial may still stand: the diagonal step must not be onto a dial tile (4-8 a tick)'},
         {'a': 'star_lance', 'b': 'meridian_line', 'conflicting_answers': False, 'note': 'prayer held while stepping: compatible'}],
     'timeline': tl, 'attacks': attacks}
(HERE / 'fight-v2.json').write_text(json.dumps(F, indent=1), encoding='utf-8')
print('fight-v2.json:', len(tl), 'timeline entries,', len(attacks), 'attacks')
