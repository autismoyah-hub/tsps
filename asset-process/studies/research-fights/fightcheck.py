"""fightcheck.py FIGHT.json [--quiet]: an encounter in the project's fight format, checked against the measured structure of
16 real end-game fights (FIGHTS.md; bands from metrics.json, built by analyze.py from measure/*.json).

Each rule prints  holds | OUTSIDE  with the fight's value and the real band. A band is the range of the measured bosses with
the single lowest and single highest dropped (14 of 16); the bosses outside it are named so a value can be compared with them.

THE FORMAT is research-effects2/fight/fight.py's (assets / stage / attacks / beats / timeline), unchanged; fight.py ignores the
fields below, so a file that carries them still plays and checks there. EXTENSIONS (all optional; missing ones are DERIVED from
the beats and marked "derived" in the output):

 fight level
  "hitpoints": N                          the boss's hitpoints
  "player_hitpoints": 99                  the player the punishments are measured against (default 99)
  "phases": [ {"name": str, "trigger": str, "at_hp_pct": N, "adds": [ATTACK, ..], "removes": [..],
               "speeds": {ATTACK: newSpeedTicks}, "changes": str} ]       one entry per phase CHANGE (not the opening phase)
            the legacy "phase": {...} object (fight.json) is read as one change that adds and speeds nothing
  "rotation": {PHASE_NAME: [ATTACK, ..]}  the repeating order of attacks in a phase (documentation; the timeline is what is counted)
  "overlaps": [ {"a": ATTACK, "b": ATTACK, "conflicting_answers": true|false, "note": str} ]
            live at once; conflicting = the two answers differ and doing one wrongly or too early fails the other
  "cadence": {"action_tick_share": 0..1, "how": str}      overrides the timeline count below
 per attack: "mechanic": {
     "kind":  "auto" | "special" | "hazard" | "add" | "phase" | "enrage",
     "solve": "prayer" | "move_off_tiles" | "step_under_or_flinch" | "line_of_sight" | "kill_adds" | "safe_spot" | "run_away"
              | "interact" | "gear_or_consumable" | "unavoidable" | "none",
     "solve_text": str,                   what the player does, by which tick of the attack
     "learnable": "fixed" | "reactive" | "mixed",
     "tell": str, "tell_ticks": N,        ticks from the first thing the player can see/hear to the damage tick
     "punish_max": N,                     damage to one player for one failed occurrence (sum over the occurrence)
     "solved_max": N,                     damage when answered (default 0)
     "punish_other": str,                 boss-side punishment: heal / enrage / reset / immune ... ("none" if none)
     "inputs": N,                         player inputs one occurrence demands (default 1; prayer: counted on style change)
     "cadence_ticks": N,                  how often it comes (default: mean gap between its timeline entries)
     "trigger": str }
DERIVATION when "mechanic" is missing: tell = first beat tick to first damage tick; punish = the player's hits summed, or the
  largest single tile's hits (a player stands on one tile), or an object's top tickDamage x lifetimeTicks; solve = prayer for a
  styled hit on the player, move_off_tiles for tile hits or a damaging object, none if nothing hits; kind = auto / special /
  hazard / phase accordingly; an object not on the player's tile ([0,0]) demands 0 inputs; learnable assumed fixed.
TIMELINE "timeline": [[tick, ATTACK], ..] is read as one representative stretch of the fight: attacks per minute = non-phase
  entries per 100 ticks of its span (last tick + that attack's speedTicks - first tick); action-tick share = inputs / span.

usage: python -I fightcheck.py FIGHT.json
"""
import sys, json, pathlib, statistics as st
HERE = pathlib.Path(__file__).resolve().parent
SPATIAL = {'move_off_tiles', 'step_under_or_flinch', 'line_of_sight', 'safe_spot', 'run_away'}
BOSS_SIDE = ('heal', 'enrage', 'reset', 'immune', 'invulner', 'regrow', 'shield')
VISIBLE = ('npc_anim', 'projectile', 'telegraph', 'impact', 'tile_graphic', 'object', 'camera_shake')


def derive(an, A):
    m = dict(A.get('mechanic', {})); d = []
    beats = A.get('beats', [])
    hits = [b for b in beats if b.get('do') == 'hit']
    objs = [b for b in beats if b.get('do') == 'object' and b.get('tickDamage')]
    first = min([b.get('tick', 0) for b in beats if b.get('do') in VISIBLE] or [0])
    dmg_ticks = [b.get('tick', 0) for b in hits] + [b.get('tick', 0) + 1 for b in objs]
    if 'tell_ticks' not in m:
        m['tell_ticks'] = (min(dmg_ticks) - first) if dmg_ticks else 0; d.append('tell_ticks')
    if 'punish_max' not in m:
        on_player = sum(b.get('maxHit', 0) for b in hits if b.get('target') == 'player')
        tiles = {}
        for b in hits:
            if b.get('target') == 'tile': tiles[str(b.get('tile'))] = tiles.get(str(b.get('tile')), 0) + b.get('maxHit', 0)
        obj = max([b['tickDamage'][1] * b.get('lifetimeTicks', 1) for b in objs] or [0])
        m['punish_max'] = max(on_player, max(tiles.values() or [0]), obj); d.append('punish_max')
    if 'solve' not in m:
        styled = [b for b in hits if b.get('target') == 'player' and b.get('style') in ('ranged', 'magic', 'melee')]
        m['solve'] = 'prayer' if styled else 'move_off_tiles' if (hits or objs) else 'none'; d.append('solve')
    if 'kind' not in m:
        m['kind'] = ('auto' if m['solve'] == 'prayer' else 'hazard' if objs and not hits else 'special' if hits else 'phase'); d.append('kind')
    if 'inputs' not in m:
        if objs and not hits and all(b.get('tile') not in ([0, 0], 'player', None) for b in objs): m['inputs'] = 0; d.append('inputs (object beside the player)')
        else: m['inputs'] = 0 if m['solve'] in ('none', 'unavoidable') else 1
    m.setdefault('solved_max', 0)
    if 'learnable' not in m: m['learnable'] = 'fixed'; d.append('learnable')
    m.setdefault('punish_other', 'none')
    m['style'] = next((b.get('style') for b in hits if b.get('target') == 'player'), None)
    m['_derived'] = d; m['name'] = an
    return m


def fight_metrics(F):
    HPp = F.get('player_hitpoints', 99)
    ms = {an: derive(an, A) for an, A in F['attacks'].items()}
    L = list(ms.values())
    non_auto = [m for m in L if m['kind'] != 'auto']
    avoid = [m for m in L if m['solve'] not in ('unavoidable', 'none')]
    tells = [m['tell_ticks'] for m in non_auto if m['solve'] not in ('unavoidable', 'none') and m['kind'] in ('special', 'hazard', 'add', 'enrage')]
    pun = [min(m['punish_max'], 999) for m in non_auto if m['solve'] not in ('unavoidable', 'none') and m['punish_max'] > 0]
    removed = [1 - m['solved_max'] / m['punish_max'] for m in avoid if m['punish_max'] > 0]
    autos = [m for m in L if m['kind'] == 'auto']
    tl = F.get('timeline', [])
    spans = []
    if tl:
        t0 = tl[0][0]; tl_end = tl[-1][0] + F['attacks'][tl[-1][1]].get('speedTicks', 1)
        span = max(1, tl_end - t0)
        n = len([1 for t, a in tl if ms[a]['kind'] != 'phase'])
        apm = round(100 * n / span, 1)
        inputs = 0; last_style = None
        for t, a in tl:
            m = ms[a]
            if m['solve'] == 'prayer':
                if m['style'] != last_style: inputs += 1
                last_style = m['style']
            else: inputs += m['inputs']
        share = round(inputs / span, 2)
        for an in ms:
            ts = [t for t, a in tl if a == an]
            if 'cadence_ticks' not in ms[an]:
                ms[an]['cadence_ticks'] = (ts[-1] - ts[0]) / (len(ts) - 1) if len(ts) > 1 else span
    else:
        apm = share = None
    if F.get('cadence', {}).get('action_tick_share') is not None: share = F['cadence']['action_tick_share']
    spec_cad = [m['cadence_ticks'] for m in non_auto if m['kind'] in ('special', 'add', 'hazard') and m.get('cadence_ticks', 0) >= 4]
    phases = F.get('phases')
    if phases is None and F.get('phase'): phases = [{'name': 'phase', 'trigger': F['phase'].get('trigger', ''), 'adds': [], 'speeds': {}, '_legacy': True}]
    phases = phases or []
    esc = bool(phases) and all(p.get('adds') or p.get('speeds') for p in phases)
    final = any((p.get('speeds') or p.get('adds')) and (p.get('at_hp_pct', 100) <= 34) for p in phases)
    ov = F.get('overlaps', [])
    return ms, {
        'solve_types': len({m['solve'] for m in avoid}),
        'non_prayer_solves': len({m['solve'] for m in avoid if m['solve'] != 'prayer'}),
        'non_auto': len(non_auto),
        'tell_median': st.median(tells) if tells else None,
        'auto_max_pct': round(100 * max([m['punish_max'] for m in autos] or [0]) / HPp),
        'special_punish_median_pct': round(100 * st.median(pun) / HPp) if pun else None,
        'worst_punish_pct': min(100, round(100 * max(pun) / HPp)) if pun else None,
        'boss_side_punish': len([m for m in L if any(k in m['punish_other'].lower() for k in BOSS_SIDE) and not m['punish_other'].lower().startswith('none')]),
        'removed_by_solve_median': round(st.median(removed), 2) if removed else None,
        'fixed_share': round(len([m for m in L if m['learnable'] == 'fixed']) / len(L), 2),
        'conflicts': len([o for o in ov if o.get('conflicting_answers')]),
        'phases': len(phases) + 1,
        'escalates': esc, 'final_speedup': final,
        'apm': apm, 'action_share': share,
        'special_cadence_median': st.median(spec_cad) if spec_cad else None,
    }


RULES = [  # id, metric, what it says
    ('R1', 'solve_types', 'distinct answers the fight asks for'),
    ('R2', 'non_prayer_solves', 'distinct answers that are not a protection prayer'),
    ('R3', 'non_auto', 'mechanics beyond the auto-attacks (specials, hazards, adds, phases, enrage)'),
    ('R4', 'tell_median', 'median tell of specials/hazards/adds, ticks'),
    ('R5', 'auto_max_pct', 'unprayed auto-attack max hit, % of a 99-hp player'),
    ('R6', 'special_punish_median_pct', 'median punishment for failing a mechanic, % of 99 hp'),
    ('R7', 'worst_punish_pct', 'worst single failed mechanic, % of 99 hp (100 = lethal)'),
    ('R8', 'boss_side_punish', 'mechanics whose failure helps the BOSS (heal / enrage / reset / immunity)'),
    ('R9', 'removed_by_solve_median', 'share of the punishment the right answer removes (median)'),
    ('R10', 'fixed_share', 'share of mechanics that are learnable (fixed pattern)'),
    ('R11', 'conflicts', 'overlaps where two live mechanics demand different answers'),
    ('R12', 'phases', 'fight states (opening + each phase change)'),
    ('R13', 'escalates', 'every phase change adds a mechanic or speeds one'),
    ('R14', 'final_speedup', 'a last-third escalation (speed-up / enrage / added mechanic)'),
    ('R15', 'apm', 'attacks per minute'),
    ('R16', 'action_share', 'share of ticks in which the player must input something'),
    ('R17', 'special_cadence_median', 'ticks between occurrences of a mechanic (median)'),
]


def bands():
    M = json.loads((HERE / 'metrics.json').read_text(encoding='utf-8'))
    out = {}
    for _, k, _ in RULES:
        if k in ('escalates', 'final_speedup'):
            yes = [m['slug'] for m in M if m.get(k)]; no = [m['slug'] for m in M if not m.get(k)]
            out[k] = ('bool', len(yes), len(M), no); continue
        if k == 'boss_side_punish':  # a count whose band starts at 0 says nothing; the rule is "at least one"
            yes = [m['slug'] for m in M if m.get(k)]; no = [m['slug'] for m in M if not m.get(k)]
            out[k] = ('min1', len(yes), len(M), no); continue
        vals = sorted([(m[k], m['slug']) for m in M if m.get(k) is not None])
        lo, hi = vals[1][0], vals[-2][0]
        outside = [f'{s} {v}' for v, s in vals if v < lo or v > hi]
        out[k] = ('band', lo, hi, outside, st.median([v for v, _ in vals]))
    return out


def report(path, quiet=False):
    F = json.loads(pathlib.Path(path).read_text(encoding='utf-8'))
    ms, V = fight_metrics(F); B = bands(); lines = []; n_out = 0
    lines.append(f'fightcheck {F.get("title", F.get("name"))}  ({path})')
    if not quiet:
        for an, m in ms.items():
            lines.append(f'  {an:18s} {m["kind"]:8s} solve={m["solve"]:20s} tell={m["tell_ticks"]} punish={m["punish_max"]} solved={m["solved_max"]} '
                         f'{m["learnable"]} other={m["punish_other"][:40]}' + (f'  [derived: {", ".join(m["_derived"])}]' if m['_derived'] else ''))
    for rid, k, what in RULES:
        v = V[k]; b = B[k]
        if b[0] == 'min1':
            ok = v >= 1; txt = f'{v}  (real: {b[1]} of {b[2]} have at least one; none: {", ".join(b[3])})'
        elif b[0] == 'bool':
            ok = bool(v); txt = f'{v}  (real: {b[1]} of {b[2]} do; not: {", ".join(b[3])})'
        else:
            ok = v is not None and b[1] <= v <= b[2]
            txt = f'{v}  (real band {b[1]}-{b[2]}, median {b[4]}; outside it: {", ".join(b[3])})'
        n_out += not ok
        lines.append(f'{rid:4s} {"holds  " if ok else "OUTSIDE"} {what}: {txt}')
    lines.append(f'{len(RULES) - n_out} of {len(RULES)} rules hold')
    return '\n'.join(lines)


if __name__ == '__main__':
    if len(sys.argv) < 2: raise SystemExit(__doc__)
    print(report(sys.argv[1], '--quiet' in sys.argv))
