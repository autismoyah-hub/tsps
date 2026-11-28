# The measured structure of an end-game fight

The owner's words were [quoted words withheld] and [quoted words withheld]. This file turns them into numbers measured on 16 real bosses whose server logic is in the TSPS checkout (read only):

- Tekton
- Great Olm
- Verzik Vitur
- Sotetseg
- Nex
- The Nightmare
- Yama
- Vardorvis
- Duke Sucellus
- The Leviathan
- The Whisperer
- Araxxor
- Vorkath
- Zulrah
- Alchemical Hydra
- Cerberus

The last four are high-level slayer and solo bosses rather than raid or DT2 bosses. They are kept in because they widen the bands at the low end, and they are named wherever they fall outside a band.

- **Per-mechanic table, with a source file and line for every number:** [TABLE.md](TABLE.md), built by [analyze.py](analyze.py) from [measure/*.json](measure/).
- **Per-boss numbers:** [metrics.json](metrics.json).
- **Checker:** [fightcheck.py](fightcheck.py) reads a fight file and reports each rule below as holds or OUTSIDE.

**How a band is made.** A band is the range of the 16 bosses after dropping the single lowest and the single highest (14 of 16). The two dropped bosses are named. Damage is given as a percentage of a 99-hitpoint player. Tell is counted in ticks from the first thing the player can see or hear to the tick the damage lands.

## The rules

| # | Rule | Band (14 of 16) | Median | Outside the band |
|---|---|---|---|---|
| R1 | Distinct answers the fight asks for (prayer, move off tiles, flinch, line of sight, kill adds, safe spot, run, interact, gear) | 3-7 | 5 | Cerberus 2, Verzik 9 |
| R2 | Distinct answers that are not a protection prayer | 2-6 | 4 | Cerberus 1, Verzik 8 |
| R3 | Mechanics beyond the auto-attacks (specials, hazards, adds, phase events, enrage) | 4-13 | 5 | Duke 3 (preparation hazards excluded), Olm 14 |
| R4 | Median tell of the specials, hazards and adds, in ticks | 1-7 | 4.75 | Sotetseg 0, Verzik 9 |
| R5 | Unprayed auto-attack max hit, % of 99 | 27-85 | 51 | Cerberus 23, Verzik 138 |
| R6 | Median punishment for failing one mechanic, % of 99 | 15-91 | 50 | Duke 124 |
| R7 | Worst single failed mechanic, % of 99 (100 = lethal) | 51-100 | 79 | Zulrah 30 |
| R8 | At least one mechanic whose failure helps the boss (heal, enrage, reset, immunity) | 11 of 16 have one | - | Araxxor, Cerberus, Leviathan, Sotetseg, Zulrah have none |
| R9 | Share of the punishment the right answer removes (median per boss) | 0.84-1.0 | 1.0 | Nex 0.81 |
| R10 | Share of mechanics that are learnable (fixed pattern) | 0.40-1.0 | 0.65 | Sotetseg 0.29 |
| R11 | Overlaps where two live mechanics demand different answers | 1-4 | 2 | Vorkath 0, Nex 5 |
| R12 | Fight states (the opening plus each phase change) | 3-7 | 4.5 | Araxxor 2 |
| R13 | Every phase change adds a mechanic or speeds one up | 13 of 16 do | - | Sotetseg (phase 3 repeats phase 1), Vorkath and Zulrah (cyclic) |
| R14 | An escalation in the last third of the boss's hitpoints (speed-up, enrage or added mechanic) | 12 of 16 do | - | Sotetseg, Tekton (his enrage follows each repair, not his hitpoints), Vorkath, Zulrah |
| R15 | Attacks per minute | 13.3-30 | 17.5 | Yama 12.5, Leviathan 38 |
| R16 | Share of ticks in which the player must input something | 0.10-0.40 | 0.24 | Nex 0.05, Tekton 0.55 |
| R17 | Ticks between occurrences of one mechanic (median) | 7-110 | 48 | Whisperer 6, Verzik 160 |

## What the rules say, in the owner's terms

**"If you don't do x, y, z you get punished."**

- A real fight asks for about five distinct answers (R1), and about four of them are not a prayer (R2).
- Answers by how often they appear across all 154 measured mechanics:

  | Answer | Mechanics |
  |---|---|
  | Move off tiles | 47 |
  | Prayer | 40 |
  | Kill adds | 16 |
  | Interact | 13 |
  | Run away | 11 |
  | Safe spot | 10 |
  | Gear or consumable | 7 |
  | Flinch or step under | 4 |
  | Line of sight | 3 |
  | Unavoidable | 3 |

- A fight with only "pray" and [quoted words withheld] sits below every measured boss except Cerberus.

**[quoted words withheld]**

- The right answer removes all or nearly all of the damage (R9).
- The wrong answer costs about half a 99-hitpoint player per mechanic (R6). Across the 93 non-auto mechanics that deal damage, the quartiles are 22, 49 and 69 damage.
- 14 of 16 bosses have a single failure that takes at least 60% of a player, and 6 have one that is lethal (R7).
- Punishment is not only damage. 11 of 16 make a failure feed the boss (R8). Tekton heals 4-8% and comes back enraged when sent to the anvil. Yama heals from flares. Vardorvis heals from his slash. The Whisperer heals 100 from souls.

**[quoted words withheld]**

- About two thirds of mechanics are fixed patterns that can be learned (R10).
- Tells are short. Across the 106 avoidable non-auto mechanics, half warn 3 to 7 ticks ahead, and 11% give no warning at all (they are answered by knowing the pattern).
- So the fights reward a learned answer, because there is too little time to work it out on the spot.

**"A real fight structure."**

- Two live mechanics demand different answers at once in 15 of 16 bosses (R11). Tekton's core is one of these: be out of reach on the landing tick, but in reach on the swing tick, or he goes to the anvil.
- Fights have three to seven states (R12). Each phase change adds or speeds something (R13).
- Most have a final escalation in the last fifth to third of their hitpoints. Hydra 6 to 4 ticks, Araxxor 6 to 4, Duke 5 to 4, Verzik 7 to 5, Vardorvis axes 13 to 8 ticks, Yama 8 to 7 (R14).

**Cadence.**

- About 17 attacks a minute (R15).
- The player must input something on about a quarter of ticks (R16).
- Each mechanic comes back roughly every 30 to 60 ticks (R17).
- Tekton is the most demanding at 0.55, because the flinch costs two inputs every three ticks. Nex is the least at 0.05, because her prayer stays the same through a whole phase.

**Learning against punishing.** The measured balance is:

- About 65% of mechanics are fixed.
- The answer removes about 100% of the damage.
- The failure costs about 50% of a player.
- About one mechanic in five (30 of 154) feeds the boss rather than hurting the player.

## What I could not establish, and what is inferred

- **Who measured what.** Fifteen of the bosses were measured by reading agents to a fixed schema. I measured Tekton myself. I spot-checked the cited lines for Vardorvis axes, the Duke's gaze and the Nightmare's claws. They point at the logic named, sometimes a few lines off.
- **Inferred fields.** 116 fields across the files are marked inferred, listed per mechanic in TABLE.md. These are mostly projectile flight times worked out from launch values and an assumed distance, and cadences of random events.
- **Estimated cadence figures.** The attacks-per-minute and input-share figures are estimates from one representative cycle per boss, with the method recorded in each file. R15 and R16 are therefore the least certain bands.
- **Escalation is my reading.** R13 and R14 are my judgement of each boss's phase text, listed in analyze.py.
- **Values from outside the boss folders.** Several hitpoint values come from `server/data/definitions/monsters-complete.json` rather than the boss plugins.
- **Not checked against live play.** Every timing is what the server schedules. The client-side readability of a tell was not measured.
