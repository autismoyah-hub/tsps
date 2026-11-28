# Prompts

The prompts the original project sent to author and Producer agents, as templates. Placeholders:

| Placeholder | Meaning |
|---|---|
| `{ROOT}` | your working folder (the package root) |
| `{BUILD}` | this job's build folder, `{ROOT}/builds/NAME` |
| `{PYTHON}`, `{BLENDER}` | the executables of `SETUP.md` |
| `{TSPS_CHECKOUT}`, `{OSRS_CACHE_239}`, `{CLIENT_SRC}`, `{REFERENCE_ARCHIVE}` ... | the keys of `asset_paths.py` |
| `{RUNNER}` | your agent runner's id |
| `{PROJECT_REPO}` | the private repository that holds your Teacher's project record (`teacher/project/`), where Producers keep `STATE.md`, `PROPOSALS.md` and their prediction counts |
| `{TSPS_WORK}`, `{WORK_DIR}`, `{EXTERNAL_DIR}`, `{ADAPTER_DIR}`, `{ASSET_ADAPTER_DIR}`, `{TEMP}` | private folders of the original machine that a prompt tells the agent not to touch; replace with yours or drop |

Files named `*-TEMPLATE.txt` (and `prompt-player-PIECE.txt`) are the forms to copy. The others are filled-in
examples of the same forms, kept because a task line is best written by copying an earlier one for the same brief.
`*-return.txt` and `*-return2.txt` show how a piece goes back to a new session with the gate's lines pasted in.
`authors/` holds the first comparison prompts given to several author models; `producers/` the prompts that start
a Producer.

Paths of the form `{ROOT}/builds/<piece>` that are not the job's own folder name pieces of the original project
(earlier sets, worked rebuilds, reference outfits of ours). Those models are not shipped: point the line at your
own earlier pieces, or drop it. Owner remarks quoted in the originals are replaced by `[quoted words withheld]`;
write your own owner's standard in their place.
