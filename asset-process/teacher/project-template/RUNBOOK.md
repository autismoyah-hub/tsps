# Runbook

How the Teacher's work is carried out on the asset machine, command by command. Paths are relative to the package
root; the keys are in SETUP.md.

1. **Start a session**: update the checkout; read `teacher/SKILL.md`, the six `core/` files one at a time, then
   `project/PROFILE.md`, `STATE.md`, `DECISIONS.md`, then `PROCESS.md`. Check the anchors in STATE.md against the machine.
2. **Start an author**: write the prompt to a file (first line names the brief in `author-pack/` or a study, then the
   task line, then where the report goes), start it with the agent runner, wait, read the report. A started session
   takes no changed assignment: a return is a new session naming the build folder and pasting the gate's lines.
   Close every session you started once its report is read.
3. **Gate** with the command for the piece's kind (`gates/README.md`). Read the VERDICT line.
4. **A real asset in Blender**: `python -I tools/real_dump.py KIND ID OUT.json`, then
   `blender -b -P tools/real_to_blender.py -- OUT.json OUT.blend`; export with
   `blender -b FILE.blend -P tools/blender_export.py -- OUT.bin` (welds, smooths, prints size and colour shares).
5. **Line-ups**: `python -I tools/lineup.py OUT "entry,entry,..." [--shuffle SEED] [--views N]`; the key stays away
   from whoever judges. Worn gear: `tools/figure.py`, `tools/pose.py`, `tools/assemble.py`. Creatures:
   `studies/research-creatures/lineup_npc.py`.
6. **Sittings**: `sittings/README.md`.
7. **Records**: findings (what was measured), state (time only via `teacher/tools/stamp_state.sh`), decisions.
