"""[quoted words withheld]"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
from pathlib import Path
import json
root=Path(__file__).resolve().parent
results=(root/'proof.log').read_text().splitlines()
checks=json.loads((root/'verification.json').read_text())
assert len(results)==29 and sum(row.endswith(' PASS') for row in results)==5
assert all(row.endswith(' FAIL') or ' ceiling VERDICT PASS' in row for row in results)
assert all(r.get('hdclassic_differing_pixels',0)==0 for r in checks)
assert (root/'damage-cli.log').read_text().splitlines()[-1]=='VERDICT FAIL'
text=_AP.fix("""# HD blind-rebuild gate

Five revision 614 conversion controls passed. All 24 damaged binaries failed. The five presentations cover the three assigned assets. The gate also passed raw-model conversion controls on revisions 561 and 601. The unchanged HD-era classic renderer and the gate's lit HD path agreed on all 1,966,080 RGB pixels across the five presentations. No blind author has run yet.

| Author packet | Presentation | Vertices | Faces | HD textures | Nonzero face alpha values |
|---|---|---:|---:|---:|---:|
| [Lamp stall](../hd-rebuild-lampstall/README.md) | Object 22772, three parts | 957 | 1,660 | 13 | 5 |
| [Diving apparatus](../hd-rebuild-diving/README.md) | Item 7535, ground | 176 | 303 | 5 | 0 |
| Same packet | Male worn | 183 | 353 | 3 | 0 |
| Same packet | Female worn | 222 | 422 | 5 | 0 |
| [Spot 2768](../hd-rebuild-spot2768/README.md) | Bind mesh and sequence 14391, 20 frames, 69 stored cycles | 441 | 798 | 0 | 16 |

## Running the tools

Use Git Bash with these absolute paths. The supplied renderer and animation bridge are already built.

```bash
P='<PYTHON>'
B='<ROOT>/studies/tools-hd-gate'
A='<ROOT>/builds'
"$P" -I -B "$B/hdsheet.py" 614 object 22772 "$A/hd-rebuild-lampstall/sheet"
"$P" -I -B "$B/hdsheet.py" 614 item 7535 "$A/hd-rebuild-diving/sheet"
"$P" -I -B "$B/hdsheet.py" 614 spot 2768 "$A/hd-rebuild-spot2768/sheet"
"$P" -I -B "$B/hdgate.py" rebuild 614 object 22772 "$A/hd-rebuild-lampstall/lampstall.bin" --out "$A/hd-rebuild-lampstall/teacher-object"
"$P" -I -B "$B/proof.py"
"$P" -I -B "$B/verify.py"
```

The sheet takes REV KIND ID OUTDIR. The gate takes rebuild REV KIND ID OURS.bin and an optional --out DIR for comparison.png and score.txt. Exit codes are 0 PASS, 1 BORDERLINE and 2 FAIL. Supported kinds are model, object, spot, item, item-ground, item-worn and item-female. An item sheet expands to every available equipment form plus ground. An item gate with one binary defaults to ground. The diving README gives three independent commands. The raw-model smoke controls were r561 model 44579 and r601 model 25.

## Reading the sheet

Each presentation has six orthographic views in archive order. The upper row shows front, back and left. The lower row shows right, three quarter and top. A single camera scale applies throughout that presentation. The header gives game units per pixel. lit.png and unlit.png keep textures. flat-lit.png and flat-unlit.png expose the underlying painted HSL with textures disabled. The unlit textured pass uses constant classic intensity 126. Each render retains face transparency and draw order.

ids.png colours the visible surface by HD texture and alpha.png uses grey level 255 minus the raw alpha byte. These two diagnostic passes draw opaque face IDs to expose the outer assignment. materials.png links each region colour to its labelled swatch. swatches/ contains one original 128 × 128 RGBA PNG per HD texture. The texture-free spot has no material swatches. sheet.json gives definition identity, game-space bounds, counts, part face budgets, material and HSL frequencies, actual triangle-area shares, and map keys. The texture count excludes the untextured sentinel. Nonzero alpha counts exclude opaque faces.

The spot packet includes vertex- and face-label guides and a fixed-camera, common-crop strip of every stored frame with its hold. The trusted local client decodes the referenced frames and their bases, poses the model, and supplies posed coordinates and alpha to the classic viewer. The strip samples frame boundaries. The author directories contain only PNG, aggregate JSON and README files. All converted and damaged meshes live in private-proof under the Teacher's tool directory.

## Scoring

Every scored line votes. The worst line determines the verdict. These are provisional rebuild thresholds. Positive controls measure attainable conversion fidelity. An independent blind rebuild is still needed to test how well the thresholds serve an author.

- **Geometry and paint.** game-unit span error per axis ≤1 to pass or ≤2 for borderline. Face-count error ≤10% or ≤15%. Distinct packed paint colours ≤max(real + 1, ceiling(1.5 × real)) or ≤2 × real. compare.py's triangle-area-weighted HSL transport distance ≤2.5 or ≤3.5. Vertex count, duplicate-position count and nonzero shading-type count are informational.
- **Outline.** compare.py rasterizes all triangles in its three independent orthographic projections at 400 px on a common scale. Objects, ground items and spots use centred X/Z and base-aligned Y. Worn forms retain body coordinates. Each projection needs intersection over union ≥0.97 or ≥0.95, reduced to ≥0.93 or ≥0.90 for worn equipment.
- **Material area.** Compute half the magnitude of the triangle-edge cross product in game space, including hidden surfaces. Compare the fraction of all area textured, translucent, and both. Absolute errors ≤0.04 or ≤0.07. Losing a present channel completely fails, as does introducing more than 1% on a channel absent from the target. Compare the entire area-weighted alpha distributions by cumulative transport over 0–255, ≤5 or ≤8 byte steps.
- **Material placement.** Within each HD material region, evaluate texture presence on the nearest rebuild surface at four interior samples per triangle, weighting triangles by area. Coordinates and unit normals enter the nearest-neighbour metric, so opposite sides of a thin surface can differ. At least 83% must agree to pass, or 72% for borderline. This is sampled correspondence and permits different triangulation. Independently check the same texture presence on visible pixels in the six classic views at the same thresholds. Visible alpha mean absolute error must be ≤10 or ≤18 bytes per region.
- **Material appearance.** The HD view uses hdview's procedural bake and complex per-corner UVs. The rebuild goes through the classic Rust rasterizer with its own binary, mapping selectors, 242 definition and indexed sprite. Lighting, background and perspective camera are shared. RGB mean absolute error is the mean absolute channel difference on the HD region's pixels, with values in 0–255. All six views contribute. Untextured regions are subdivided by visible alpha value. For an occluded texture, isolate corresponding faces and inspect six additional angles at a tighter shared scale. This supplied a measured look for the lamp's otherwise hidden material 57.
- **Cross-catalogue tolerance.** For each HD material the conversion compares its swatch with all 216 installed 242 textures. It minimizes mean absolute distance across RGBA channel quantiles (10, 25, 50, 75 and 90%) and double-weighted channel means after compositing RGB on grey 40. This defines “best” for the scripted selector. It does not optimize UV layout or rendered-pixel error globally. The control retains simple texture bases and substitutes face-local planar coordinates for HD projected bases. Each RGB line reports its own measured conversion residual. PASS allows that residual plus 10 levels, BORDERLINE plus 18. For example the lamp canopy region 122 has a 15.50-level residual. Replacing its chosen texture with 242 texture 88 gives 47.52 and fails despite identical geometry and texture coverage. Different HD materials can select the same 242 texture.
- **Motion.** The spot gate drives the submitted mesh labels with the HD reference sequence. It compares foreground IoU in the front and three-quarter views at every stored frame. Both empty is an exact match. The mean must reach 0.78 or 0.70 and the worst cell 0.50 or 0.40. Swapping labels 0 and 14 preserves the entire rest mesh but takes the worst posed IoU to zero. This tests compatibility with the supplied motion. A separately authored sequence would require an additional input.

## Proof and scope

proof.py obtains each part from hdconvert.convert, independently reapplies the definition transforms and material overrides, checks the result against HD assembly, chooses target materials and packs the converted parts. Each fixture is encoded to an ordinary model binary and decoded again for scoring. The wrong-texture fixtures preserve texture coverage. Single-part equipment substitutions exchange the ground and worn mesh. The equipment has zero source face alpha to remove, so its transparency control introduces alpha 160. Spot 2768 has zero source textures, so its material-removal test strips the largest nonzero alpha band. Its part swap exchanges animation groups. The lamp swap exchanges physical positions of two constituent models.

verify.py opened all 63 author PNGs, validated the packet file types and swatch sizes, ran all five ceiling binaries through the public CLI and compared its complete output with the saved score transcripts. A separate public CLI run on the stripped lamp material returned exit code 2 and wrote a FAIL transcript. Reference-raster parity covers the lit HD path on the selected presentations. Material animation, NPCs, morph-state variants, historical GPU shaders, runtime particles and independent authored animation clips are outside this demonstration. Offscreen surfaces can contribute to the 3D area scores even when no visible pixel depicts them. Motion was verified for the named identity-scale spot. General definition-scale or rotation interactions during playback have not been validated across all HD sequences.

## Command failures and recovery

- The initial shell override dispatched the Git Bash discovery string through cmd, which rejected pwd. A quoted Bash path without a PowerShell call operator then failed to parse -lc. An explicit PowerShell call operator invoking Git Bash worked for all subsequent shell commands.
- git rev-parse --show-toplevel found no repository in the working folder or the assigned Cuttle directory. The tools and their build target use a new isolated directory in the specified builds tree.
- The initial Python -c target probe lost its inner quotes at the Windows argument boundary. Running inspect_target.py with the specified isolated interpreter produced the five definitions and full hold list.
- The first proof.py run rejected NumPy scalar coordinates as signed-smart overflow while packing the moved-geometry fixture. Converting the array to Python integers fixed it.
- The initial six-view visibility audit and first isolated view could not see HD material 57. The final complementary-angle cutaway measured it on 79,771 pixels. An rg probe for residual “no front-facing” lines subsequently returned no matches.
- The first verify.py import resolved proof to the older HD-era proof.py because of its path setup. That script executed its 22-pair verification and regenerated tools-hd-era/pixel-proof.json and proof-sheet.png. It then raised ImportError for TARGETS. Their final lengths equal the initial lengths (8,367 and 53,844 bytes). No before-run hashes are available for a byte-identity claim. The verifier now declares its target list locally. A head probe of the still-absent CLI score and an ls of verification.json before its generating run completed also reported missing files.
- The stripped-material CLI test intentionally returns nonzero on a damaged binary. All fixture verdicts follow below. The source tools' default behaviour was not edited. No author was started, cache content was treated as data, and no assets were uploaded or committed.

## Complete gate transcripts

The following are the saved gate lines for each converted binary and every damaged copy. Each subsection names the changed property. Mapping keys are HD texture IDs and values are installed 242 texture IDs.
""")
for kind in ('object','item-ground','item-worn','item-female','spot'):
    folder=root/'private-proof'/kind
    text+='\n### '+kind+'\n\nHD to 242 mapping `'+json.dumps(json.loads((folder/'mapping.json').read_text()),sort_keys=True)+'`.\n'
    for line in results:
        if not line.startswith(kind+' '):continue
        name,verdict=line[len(kind)+1:].rsplit(' VERDICT ',1)
        score=(folder/(name+'.txt')).read_text().strip()
        assert score.endswith('VERDICT '+verdict)
        text+='\n#### '+name+'\n\n```text\n'+score+'\n```\n'
(root/'REPORT.md').write_text(text,encoding='utf-8')

