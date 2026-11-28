# research-fights: the study's report (saved by the interim Teacher from the agent's final message, 2026-11-27)

Full findings: FINDINGS 81 in {ROOT}/FINDINGS.md. The rules are FIGHTS.md; the redesign REDESIGN.md and
fight-v2.json (from build_v2.py); per-boss sources measure/*.json; the mechanic table TABLE.md.

- 16 bosses measured from the TSPS server's own logic with file:line sources (Tekton, Olm, Verzik, Sotetseg, Nex,
  Nightmare, Yama, Vardorvis, Duke, Leviathan, Whisperer, Araxxor, Vorkath, Zulrah, Hydra, Cerberus); Tekton by
  the study's author, the others by read-only agents to a fixed schema (measure/BRIEF.md); 116 fields marked
  inferred; citations spot-checked on 3 of 15.
- 17 rules (each band 14 of 16 bosses): distinct answers 3 to 7; non-prayer answers 2 to 6; mechanics beyond autos
  4 to 13; median tell 1 to 7 ticks; unprayed auto max 27 to 85% of 99 hp; median failure cost 15 to 91%; worst
  failure 51 to 100%; a failure that helps the boss in 11 of 16; the right answer removes 84 to 100%; learnable
  share 0.40 to 1.0; conflicting overlaps 1 to 4; fight states 3 to 7; every phase escalates (13 of 16); a
  last-third escalation (12 of 16); 13.3 to 30 attacks a minute; 0.10 to 0.40 of ticks need an input.
- fightcheck.py: the current Warden fight holds 8 of 17 (outside R1, R2, R3, R8, R11, R12, R13, R14, R16);
  the Tekton transcription 15 (R14, R16); fight-v2.json 17.
- Redesign: 900 hp; Orbit must be ARRESTED (or it heals him 5% and doubles the next Lance); a new Sighting
  (4-tick mark, 3 bolts of 20, prayer useless, stand behind a live dial) that conflicts with arresting the dial;
  at 50% the beat goes from 5 to 4 ticks, two dials, the Line becomes a Meridian Cross (only a diagonal step is
  safe); at 20% "Last Bearing", every Lance a double bolt; a fixed rotation per phase. No Tekton-style flinch.
- Films: a solved clip and a failed clip per mechanic, specified tick by tick; the film tools first need a moving
  player, hitsplats, heal splats, a health bar and the prayer icon.
- Not established: pace bands (R15, R16) from one cycle per boss; R13 and R14 are readings of phase text; no live
  play; new assets (ids 4187 to 4190) and the server side (Arrest, line of sight, heal) not built.
