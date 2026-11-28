# Where the asset system stands, domain by domain (2026-11-30)

Written for the owner at their request and for whoever holds the Teacher's seat next. "Proven" means the owner judged
it in a sitting. Evidence for every line is in `docs/verdicts/COUNT.md` and `{ROOT}/FINDINGS.md`.

## The scale used

1. **Harness**: our tools draw what the game client draws (0 differing pixels).
2. **Rules**: real assets measured, rules written with numbers.
3. **Rebuild**: real assets rebuilt blind from sheets; a gate scores them; the owner cannot tell them (or can, and says how).
4. **Originals**: new designs the owner grades Strong, from a written brief.
5. **Transfers**: less expensive authors do it from the brief alone, more than once.
6. **In the game**: packed, loaded by id, seen in the client the game runs on.

| Domain | Level reached | Proven by | Open fault or next ceiling |
|---|---|---|---|
| Scenery objects, furniture | 5 | 11 blind rebuilds not found; three sets Strong; all by Astra | none known; at 6 only in our tooling client |
| Rooms | 5 | four rooms Strong; a Producer finished the armoury with ordered pieces and the owner preferred it to an already Strong room | no room yet in a real map of the game the owner runs |
| Wall and floor decorations | 5 | a shield rebuild not told from the real one; ten kinds of originals in a Strong room | transparency in the exporter; two-tile pieces |
| Weapons | 5 | rebuilds not told; four originals Strong | icons, female grip and attack animations exist but are untested in TSPS |
| Armour, rebuilds | 5 | chainbody, platebody, legs, five reference helms; found only close up, all Strong | none known |
| Player gear, original (fashionscape) | 4 for ONE set; a second set's helm FAILED | Meridian helm and set pass; Umbral helm from the brief alone: [quoted words withheld] | the brief does not carry to a new set; study in `studies/research-helm2/` |
| Complete items (icon, ground, female, chat head, definition) | 4 | two sets read right | not yet seen in the TSPS client (in work) |
| Capes | 4, once | one cape Strong | one piece; no brief |
| Animation on existing skeletons | 5; sets and locomotion passed once | five attacks, all Yes; the hound's own idle, walk and death: Yes | boss attacks (in work); long sequences |
| Effects: projectiles, impacts, telegraphs, hits | 5 | rebuilds not told; two creatures' sets Strong, one from the brief alone | not yet triggered by a server |
| Arena objects that animate, hazards | 3 | three rebuilds pass the gate | not shown blind; no originals judged alone |
| Fights as data | 2 | plays an attack as a film; emits the server's JSON form | never run on a server |
| NPCs from our own gear | 4, once | the warden Strong | chat heads, combat data |
| Creatures on a borrowed skeleton, original | 4, passed once after three rejections | star hound Strong; watcher Rework, Usable, Closer, then Strong by measured cleanness | face features still read as [quoted words withheld]; detail lost in cleaning |
| Creatures on their own classic skeleton | 4, once | the Armillary sentinel Strong | only rigid parts |
| Weighted (skeletal) creatures | 3, with one original Strong | encoders exact on every real file; the Zodiac ray Strong, [quoted words withheld] | weights by rule do not pass the gate on a boss; no weighted rebuild; posing still crude |
| Bosses, rebuild | 3 | Tekton passes the gate plate-first; found by [quoted words withheld]; Strong | small detail; only classic bosses; none weighted |
| Bosses, original | 4, passed once | Warden-General III Strong, [quoted words withheld] to the first; holds in motion | one boss, on a borrowed classic skeleton; a whole encounter is being assembled |
| Interfaces | 2 | harness exact on 969 groups; conventions measured; a delivery channel into TSPS proven | blind rebuilds from rules alone FAIL; rules being revised |
| Delivery into TSPS | 3 | our assets seen in the real client, all Yes; display faults traced to settings and camera | pull request 6868 awaits re-review; nothing triggered by a server |
| Running without Fable | proven for modelling; partly for orchestration | Astra authors everywhere above; Producers ran three furniture orders and a room | composition and design judgment still need the owner's sittings and measured rules |

## What the system has learned about itself

- Rebuilding real assets blind is what makes originals good. Every domain where it was skipped failed first.
- The owner's rejections are the most valuable data. Each one, turned into a measurement against the owner's own
  references, produced a rule and a tool: fit (helms), definition (sets), plates before paint (bosses), density and
  grouping (rooms), softness (effects). The same move is now being made for paint style (creatures).
- No model's eye predicts the owner's. The system works when judgment is replaced by measurement plus sittings.

## Room to grow, in order of what it unlocks

1. **Paint style and small detail on creatures** (running): the last named fault between Usable and shippable.
2. **An original end-game boss**, classic first: shape and depth by the helm's process, plates before paint, a paint
   plan, effects set, attack animations, arena objects, fight as data: every piece now exists separately. Assembling
   them into ONE encounter is the next proof.
3. **Weighted bosses**: ownership by parts file plus the ring and web rules, proven by rebuilding the Maggot King
   blind; pose tools that give weight shift; then an original.
4. **In the owner's game**: items, objects and NPCs through TSPS's existing channels; custom skeletons and large
   animations through our pull requests; one encounter playable end to end.
5. **Breadth without the Teacher**: a second player set, a second and third room, a creature family, each run by a
   Producer from briefs, with the gate, the checkers and the owner's sittings as the only judges.
6. **New ground**: interfaces; map building at scale (a whole floor, a dungeon); locomotion and full animation sets;
   NPC chat heads and dialogue; sound is out of scope.

## The far end

An encounter designer writes a page: a place, a boss, its attacks, its rewards. The system returns the arena as a map
square, the boss with its own skeleton and animations, its projectiles, telegraphs and hazards, the fight as data the
server runs, the reward set in every form an item needs, all drawn first as films and blind line-ups for the owner,
all loadable by the game. Every step of that exists today in at least a first proven form except three: a boss of our
own at the standard of the game's best, weights for soft bosses, and the last mile into the running game.
