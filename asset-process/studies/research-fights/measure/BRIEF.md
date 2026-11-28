Measurement brief (input for the measuring agents; not a report)

Goal: from the TSPS server source ONLY ({TSPS_CHECKOUT}/, READ ONLY: no edits, builds, server runs, git), measure the
fight structure of each assigned boss. Every number needs a source "file:line" (path relative to {TSPS_CHECKOUT}/).
Anything not read directly in source must be listed in that mechanic's "inferred" array. No web. Do not guess from memory of
the real game unless marked inferred.

Write ONE file per boss: {ROOT}/studies/research-fights/measure/<slug>.json (use your file Write tool, not a
shell heredoc). Write nothing else anywhere. Schema:

{
 "boss": "Name", "slug": "slug", "npc_ids": [..], "hitpoints": N, "hitpoints_src": "file:line",
 "sources": ["file", ..],
 "base_attack_speed_ticks": N, "base_attack_speed_src": "file:line",
 "phases": [ {"name": "...", "trigger": "e.g. hp<=66% / timer / adds dead", "changes": "what is added/removed/sped up",
              "src": "file:line"} ],
 "mechanics": [ {
   "name": "...",
   "kind": "auto" | "special" | "hazard" | "add" | "phase" | "enrage",
   "trigger": "what starts it (every Nth attack, random 1/N, hp threshold, timer, player position...)", "trigger_src": "file:line",
   "tell": "what the player sees/hears (anim id, gfx id, projectile, tile graphic, overhead text, sound)",
   "tell_ticks": N,            // ticks from the FIRST visible tell to the damage tick (0 = no tell before damage)
   "tell_src": "file:line",
   "solve": "prayer" | "move_off_tiles" | "step_under_or_flinch" | "line_of_sight" | "kill_adds" | "safe_spot" |
            "run_away" | "interact" | "gear_or_consumable" | "unavoidable",
   "solve_text": [quoted words withheld],
   "learnable": "fixed" | "reactive" | "mixed",   // fixed = same pattern every time, can be pre-planned; reactive = random, must read tell
   "punish_min": N, "punish_max": N,              // damage to ONE player for one failed occurrence (sum over the occurrence)
   "solved_min": N, "solved_max": N,              // damage when solved correctly (0 if fully avoided)
   "punish_other": "heal / enrage / reset / stat drain / prayer drain / freeze / teleport / instant death / none",
   "punish_src": "file:line",
   "cadence": [quoted words withheld], "cadence_ticks": N, "cadence_src": "file:line",
   "inferred": [[quoted words withheld]]
 } ],
 "overlaps": [ {"a": "mech", "b": "mech", "conflicting_answers": true|false,
                "note": [quoted words withheld],
                "src": "file:line"} ],
 "cadence_summary": {"attacks_per_minute": N, "action_tick_share": 0..1,
                     "how_computed": "e.g. 100 ticks = 1 minute; count ticks in which the player MUST input something (prayer switch, move, click) to avoid punishment, over one representative cycle"},
 "max_typical_hit": N, "max_typical_hit_src": "file:line",
 "notes": [quoted words withheld]
}

Include the boss's ordinary auto-attacks as mechanics too (their solve is usually prayer; punishment = max hit unprayed,
solved = damage through prayer if any). Include adds, hazards, phase transitions, enrage/heal/reset behaviour. One server tick
= 600 ms, 100 ticks = one minute. Be exhaustive but terse. When the code has normal/hard modes, measure the normal mode and
note the hard mode differences in "notes". Finish by returning a 10-line summary to your caller.
