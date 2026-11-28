# asset-process

The tools, rules, briefs, prompts and review method an AI agent team uses to make game assets for this server
that read as genuine Old School RuneScape assets: a harness that draws what the game draws, rules measured from
the game's own assets, blind rebuilds scored by gates, and originals judged by a person in short review sittings.

Start with [PROCESS.md](PROCESS.md) (the method and who does what), then [SETUP.md](SETUP.md) (what to install and
which game data you must supply yourself). [INVENTORY.md](INVENTORY.md) lists what was taken from the original
working folder and what was left out, and why.

| Folder | What it holds |
|---|---|
| `asset_paths.py`, `config.example.toml` | the one place every machine path is set |
| `tools/` | the harness and the core tools: cache reader, model writer, Blender bridge, line-ups, figures, scenes, the scenery and worn-gear gate |
| `author-pack/` | what an author reads: `RULES.md`, the briefs, the room, wall-decoration, player-gear and fashion rules, small author tools |
| `studies/` | one folder per study: its rules (`*.md`) and the scripts that measured them and check against them (`research-*`), and the tool builds (`tools-*`) |
| `gates/` | index of every gate and checker, with its command |
| `render/` | the renderer contract, scene format and picture presets |
| `sittings/` | the review page maker and collector, the spec format and a sample |
| `prompts/` | the agent prompts as templates (`{ROOT}`, `{BUILD}`, `{PYTHON}` ...); `authors/` and `producers/` examples |
| `teacher/` | the Teacher role (`SKILL.md`, `core/`, `templates/`, `playbooks/`, `exam/`, `check.sh`) and a project template |
| `examples/barrel/` | the worked example of a model written by script |
| `client/` | not in this branch yet: the Rust client source the harness is built on, its prebuilt executables and build notes are held back pending a licence check (it is a port of the game client). Until then, supply your own renderer executables: `SETUP.md` lists each one the tools call and its arguments |

Nothing here is game data: no cache files, keys, real models, pictures or sheets of real assets.
