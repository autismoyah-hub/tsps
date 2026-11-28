"""analyze.py: the per-boss measurements (measure/*.json) -> TABLE.md (every mechanic, with sources) and metrics.json
(the per-boss numbers the rules in FIGHTS.md and fightcheck.py are banded on).

usage: python -I analyze.py
A failed-mechanic punishment of 99 or more (or the 9999 'instant death' marker) counts as LETHAL for a 99-hitpoint player.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import json, pathlib, statistics as st
HERE = pathlib.Path(__file__).resolve().parent
HP = 99
BOSS_SIDE = ('heal', 'enrage', 'reset', 'immune', 'invulner', 'regrow', 'shield')
AVOID = {'move_off_tiles', 'step_under_or_flinch', 'line_of_sight', 'kill_adds', 'safe_spot', 'run_away', 'interact', 'gear_or_consumable', 'prayer'}
SPATIAL = {'move_off_tiles', 'step_under_or_flinch', 'line_of_sight', 'safe_spot', 'run_away'}
# The Duke's preparation hazards are before the fight; they are tabled but not counted in the fight's numbers.
EXCLUDE = {('duke-sucellus', 'Preparation eye cones'), ('duke-sucellus', 'Preparation falling ice'), ('duke-sucellus', 'Preparation gas')}


# Does every phase change add a mechanic or speed one up? My reading of each file's "phases" entries (marked: judgement on read text).
ESCALATES = {'alchemical-hydra': True, 'araxxor': True, 'cerberus': True, 'duke-sucellus': True, 'leviathan': True, 'nex': True,
             'nightmare': True, 'olm': True, 'sotetseg': False, 'tekton': True, 'vardorvis': True, 'verzik': True, 'vorkath': False,
             'whisperer': True, 'yama': True, 'zulrah': False}
# A last-stage speed-up or enrage in the final third of hitpoints (or, for Tekton, after each repair): same reading.
FINAL_SPEEDUP = {'alchemical-hydra': '275/1100 hp: speed 6->4, max 35->55', 'araxxor': '25%: speed 6->4', 'cerberus': '<200 hp: lava added',
                 'duke-sucellus': '121/485 hp: speed 5->4', 'leviathan': '20%: enrage, orbs bypass prayer outside aura',
                 'nex': 'Zaros (20%): heal 500, curses', 'nightmare': 'phase 3: spores/surge', 'olm': 'final stand: crystals, siphon, +2 max',
                 'sotetseg': None, 'tekton': None, 'vardorvis': '1/3 hp: axes 13->8 ticks',
                 'verzik': '20%: speed 7->5, tornadoes', 'vorkath': None, 'whisperer': '0 hp: heal to 140, shot every 2 ticks',
                 'yama': 'phase 3: speed 8->7', 'zulrah': None}


def load():
    out = []
    for f in sorted((HERE / 'measure').glob('*.json')):
        d = json.loads(f.read_text(encoding='utf-8'))
        d['mechanics'] = [m for m in d['mechanics'] if (d['slug'], m['name']) not in EXCLUDE]
        out.append(d)
    return out


def boss_side(m):
    o = (m.get('punish_other') or '').lower()
    return any(k in o for k in BOSS_SIDE) and not o.startswith('none')


def metrics(d):
    ms = d['mechanics']
    non_auto = [m for m in ms if m['kind'] != 'auto']
    avoidable = [m for m in ms if m['solve'] != 'unavoidable']
    tells = [m['tell_ticks'] for m in non_auto if m['solve'] != 'unavoidable' and m['kind'] in ('special', 'hazard', 'add', 'enrage')]
    pun = [min(m['punish_max'], 999) for m in non_auto if m['solve'] != 'unavoidable' and m['punish_max'] > 0]
    removed = [1 - m['solved_max'] / m['punish_max'] for m in avoidable if m['punish_max'] > 0]
    spec_cad = [m['cadence_ticks'] for m in non_auto if m['kind'] in ('special', 'add') and (m.get('cadence_ticks') or 0) >= 4]
    autos = [m for m in ms if m['kind'] == 'auto']
    return {
        'boss': d['boss'], 'slug': d['slug'], 'hp': d.get('hitpoints'),
        'mechanics': len(ms), 'non_auto': len(non_auto),
        'solve_types': len({m['solve'] for m in avoidable}),
        'spatial_solves': len([m for m in avoidable if m['solve'] in SPATIAL]),
        'non_prayer_solves': len({m['solve'] for m in avoidable if m['solve'] != 'prayer'}),
        'tell_median': st.median(tells) if tells else None, 'tell_min': min(tells) if tells else None, 'tell_max': max(tells) if tells else None,
        'auto_max_pct': round(100 * max([m['punish_max'] for m in autos] or [0]) / HP),
        'special_punish_median_pct': round(100 * st.median(pun) / HP) if pun else None,
        'lethal_mechanics': len([p for p in pun if p >= HP]),
        'worst_punish_pct': min(100, round(100 * max(pun) / HP)) if pun else None,
        'escalates': ESCALATES.get(d['slug']), 'final_speedup': FINAL_SPEEDUP.get(d['slug']),
        'removed_by_solve_median': round(st.median(removed), 2) if removed else None,
        'boss_side_punish': len([m for m in ms if boss_side(m)]),
        'fixed_share': round(len([m for m in ms if m['learnable'] == 'fixed']) / len(ms), 2),
        'conflicts': len([o for o in d.get('overlaps', []) if o.get('conflicting_answers')]),
        'phases': len(d.get('phases', [])),
        'apm': d['cadence_summary'].get('attacks_per_minute'),
        'action_share': d['cadence_summary'].get('action_tick_share'),
        'special_cadence_median': st.median(spec_cad) if spec_cad else None,
        'inferred_fields': sum(len(m.get('inferred', [])) for m in ms),
    }


def table(ds):
    L = ['# Mechanics of the measured bosses', '',
         _AP.fix('One row per mechanic. tell = ticks from first visible warning to damage. punish = damage to one player on a failed '
         'occurrence (min-max), with % of a 99-hitpoint player at max; solved = damage when answered. src: paths relative to '
         '<TSPS_CHECKOUT>/. "inferred" names fields not read directly in source. Built by analyze.py from measure/*.json.'), '']
    for d in ds:
        L += [f'## {d["boss"]}', '',
              f'Hitpoints {d.get("hitpoints")} ({d.get("hitpoints_src")}). Base attack speed {d.get("base_attack_speed_ticks")} ticks '
              f'({d.get("base_attack_speed_src")}). Cadence: {d["cadence_summary"].get("attacks_per_minute")} attacks/min, '
              f'action-tick share {d["cadence_summary"].get("action_tick_share")} (estimate: {d["cadence_summary"].get("how_computed")}).', '',
              '**Phases**', '']
        for p in d.get('phases', []):
            L.append(f'- {p["name"]}: {p["trigger"]} -> {p["changes"]} ({p["src"]})')
        L += ['', '|Mechanic|Kind|Trigger|Tell (ticks)|Solve|Learnable|Punish (max % of 99)|Solved|Boss-side / other|Cadence|Sources|Inferred|',
              '|---|---|---|---|---|---|---|---|---|---|---|---|']
        for m in d['mechanics']:
            pct = 'lethal' if m['punish_max'] >= 999 else f'{round(100 * m["punish_max"] / HP)}%'
            srcs = '; '.join(dict.fromkeys([m.get('trigger_src', ''), m.get('tell_src', ''), m.get('punish_src', ''), m.get('cadence_src', '')]))
            cell = lambda s: str(s).replace('|', '/').replace('\n', ' ')
            L.append('|' + '|'.join(cell(x) for x in [m['name'], m['kind'], m['trigger'], f'{m["tell_ticks"]}: {m["tell"]}', f'{m["solve"]}: {m["solve_text"]}',
                     m['learnable'], f'{m["punish_min"]}-{m["punish_max"]} ({pct})', f'{m["solved_min"]}-{m["solved_max"]}', m.get('punish_other', ''),
                     f'{m.get("cadence")} ({m.get("cadence_ticks")})', srcs, ', '.join(m.get('inferred', [])) or '-']) + '|')
        L += ['', '**Overlaps**', '']
        for o in d.get('overlaps', []):
            L.append(f'- {o["a"]} + {o["b"]}: {"CONFLICTING answers" if o.get("conflicting_answers") else "compatible"}. {o["note"]} ({o.get("src")})')
        L += ['', f'Notes: {d.get("notes", "")}', '']
    return '\n'.join(L)


def main():
    ds = load()
    M = [metrics(d) for d in ds]
    (HERE / 'metrics.json').write_text(json.dumps(M, indent=1), encoding='utf-8')
    (HERE / 'TABLE.md').write_text(table(ds), encoding='utf-8')
    keys = [k for k in M[0] if k not in ('boss', 'slug')]
    print('slug'.ljust(17) + ' '.join(k[:9].rjust(9) for k in keys))
    for m in M:
        print(m['slug'][:16].ljust(17) + ' '.join(str(m[k]).rjust(9) for k in keys))


if __name__ == '__main__':
    main()
