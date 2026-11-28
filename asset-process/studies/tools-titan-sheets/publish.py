"""Assemble the author instructions and Teacher's calibration record from measured output."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys
from pathlib import Path
sys.dont_write_bytecode=True
sys.path.insert(0,str(Path(__file__).resolve().parent))
from titan import *
import itertools
def read(path):return json.loads(path.read_text())
def md(path,lines):path.write_text('\n'.join(lines)+'\n',encoding='utf-8')
def spans(items):
    chunks=[]
    for _,g in itertools.groupby(enumerate(sorted(items)),lambda p:p[1]-p[0]):
        vals=[v for _,v in g];chunks.append(str(vals[0]) if len(vals)==1 else f'{vals[0]}–{vals[-1]}')
    return ', '.join(chunks)
def table(headers,rows):
    return ['| '+' | '.join(map(str,row))+' |' for row in [headers,['---']*len(headers),*rows]]
report=['# Royal Titans · blind rebuild calibration','',
'Both revision-239 combat definitions clear the stock gate at the encoder ceiling in all five attacks. The two baselines also match in bind, stand, walk and death. Stripping ordinary transparency produces BORDERLINE on Branda and FAIL on Eldric. Removing a limb label or shifting one part by a single lightness unit leaves the overall verdict at PASS. The individual lines expose both changes.','','The opaque small-mark test has 45 visible marks on Branda and zero on Eldric. A perfect Eldric detail score carries no evidence about the placement of ice markings. Inspect his layered reference and the moving lineup before judging a rebuild.','',
'[Comparison board](royal-titans-and-tekton.png) · [Branda sheet](sheets/12596/SHEET.md) · [Eldric sheet](sheets/14147/SHEET.md)','',
'The board uses the same unscaled 500 × 560 render cell for all nine pictures, with zoom 1350, aim 420, yaw 192 and pitch 128. Rows show bind, melee and area attack. Tekton shows two frames of his attack. The definitions retain their own game scale. The four author folders contain instructions only.','']
appendix=[]
for n in ATTACKS:
    base=ROOT/'teacher-only'/str(n);sh=ROOT/'sheets'/str(n)
    inv=read(base/'fixtures/inventory.json'); meta=read(sh/'sheet.json');names=read(base/'sequence-names.json');model=inv['model'];sk=inv['skeleton'];npc=inv['npc'];detailres=read(base/'detail/detail.json')
    mark_index=read(sh/'marks/index.json')
    assert meta['sheet_version']==3
    other=L.npc_def(14148 if n==12596 else 14149)
    seqrows=[]
    seqids={s['id'] for s in inv['sequences']}
    for s in inv['sequences']:
        name=names[str(s['id'])]
        seqrows.append([s['id'],name,s['frames'],s['length'],'arena object' if name.startswith('loc_') else 'NPC'])
    share=[f"{p['id']} {p['name']}" for p in L.census('npcs').values() if seqids.intersection(p['anims'])]
    hist={int(k):v for k,v in inv['alpha'].items()}
    ordinary=sum(v for k,v in hist.items() if 0<k<254)
    identity=table(['Property','Revision 239'],[
        ['Fought form',f"{n} · {npc['name']} · combat {npc['combat']} · {npc['ops']}"],
        ['Inactive form',f"{other['id']} · model {other['models'][0]} · combat {other['combat']} · {', '.join(other['ops'])}"],
        ['Model',f"{model['id']} · {model['verts']} stored vertices · {model['faces']} stored faces · {meta['counts']['faces']} drawn"],
        ['Size and scale',f"{npc['size']} tiles · {npc['scale']} / 128 · ambient 0 · contrast 0"],
        ['Rig',f"classic base {sk['id']} · {sk['groups']} transform groups · {sk['bones']} bones · weights {model['weights']}"],
        ['Base label IDs',f"{len(sk['labels'])} · {spans(sk['labels'])}"],
        ['Mesh vertex label IDs',f"{len(model['labels'])} · {spans(model['labels'])}"],
        ['NPCs referencing this base in a definition animation',', '.join(share)],
        ['Sharing beyond the NPCs','Four floor-object sequences below use the same frame base. The other titan and elemental adds have distinct bases.'],
        ['Materials',f"{model['textures']} textured faces · {len(inv['palette'])} raw paints · {len(meta['palette'])} drawn paints · {model['flat']} flat faces"],
        ['Transparency',f"{ordinary:,} ordinary translucent faces ({100*ordinary/model['faces']:.2f}% of stored faces) · {model['alpha']} nonzero bytes including special 254/255"],
        ['Visibility classes in the standard pose sample',', '.join(f'{k} {v}' for k,v in meta['faces_by_pose'].items())]
    ])
    stats=[*identity,'','### Sequences','','Frame count and total duration in client cycles come from the classic sequence census. Gameval spelling is retained. Each NPC has twelve character sequences and shares its frame base with four floor sequences.','',*table(['ID','Cache name','Frames','Cycles','Played by'],seqrows),'','### Paint and transparency','','Raw transparency byte and face count, before animation. Zero is opaque. Ordinary values 1–253 blend with the background. Byte 254 selects a special flat scanline operation and 255 suppresses the face.','',*table(['Byte','Faces'],sorted(hist.items())),'','Packed HSL paint → stored face count (including undrawn faces). The gridded palette gives the drawn surface-area shares.','',', '.join(f'{k} → {v}' for k,v in sorted(inv['palette'].items(),key=lambda x:int(x[0]))),'']
    sheetdoc=[f"# {npc['name']} · version 3",'','Begin with the lit pose set to locate the live anatomy. Use the phase panels to assign plates and marks, then use the registered rest maps to establish paint boundaries and lighting. Use the sheet images and summary measurements as your reference. Build the plates, joints and interior construction independently.','',
'## Reading the pictures','',
f'- **sheet-{n}.png** shows axis paint views, a 20-unit grid, softened vertex-label regions, centroids and the drawn palette. **archive-{n}.png** is the earlier six-view NPC portrait. The two JSON sheet files contain measurements and label summaries. **sheet.json** also registers the 18 orthographic views, enlargements, style targets and phase counts.',
'- **board-paint, board-lit and board-ink** collect the 18 rest views. In **views/** each filename starts with the camera direction. Paint gives the opaque front surface, lit applies the rest light, and light stores its number in the grey channel (128 preserves the painted lightness). Ink marks opaque paint boundaries in white, depth steps within a paint in blue and the silhouette in grey.',
'- **tpaint and tlit** show the foremost translucent surface ahead of opaque paint. PNG opacity equals 255 minus its stored byte. The orthographic boards composite this one translucent layer. Multiple ice layers and the special scanline operation require the client-lit pose for their final appearance.',
'- **alpha** gives a location map. Charcoal is background and grey is opaque-only. A coloured pixel has RGB (raw byte, 190, 255 minus raw byte). Blue-green means less transparency and yellow-green more. **alpha-bands.png** separates every stored byte, including hidden faces, in front and side views. Each panel shows that band’s paint at full visibility. Read it with the counts below.',
'- **pose/** contains 96 registered client-camera cells. Each name gives row and zero-based frame, yaw and pitch. Lit is the complete client image. Paint and light describe the opaque layer. Tpaint describes the foremost translucent layer using bind material values. ID-colour renders used to derive the diagnostic layers freeze face-alpha animation. Use lit to judge animated transparency. **pose.json** gives all rows, cameras, sequences and framing.',
'- **actions/** shows two frames from every character sequence, front and rear. For example, q11974-4-y192 denotes sequence 11974 at frame 4 and yaw 192. See the sequence table for names. **comparison-0, -1 and -2** give bind, melee and area attack from the common-scale Teacher board.',
'- **enlargements/** has two types of strips. Direction-number files show the highest-density opaque boundary windows in that direction, painted, lit and inked. Head, hand-weapon, hand-grip and the named effect strip show the front crop in composite paint, composite lighted colour and translucent paint. Closely spaced plates and the grip can sit behind the weapon in profile.',
'- **phases/** draws each visibility class separately in ten common rest projections. Both means seen at rest and live. Rest-only and live-only include surfaces strongly biased to one state. Death-only describes surfaces chiefly revealed in the death sample. Never means below the sampled visibility cutoff. These are observations from eight horizontal cameras and selected frames. They do not identify a semantic body part or prove invisibility across a whole sequence.',
'- **phase-poses-y192 and -y1216** carry the same isolated pieces through bind, stand, walk, melee and death beside the complete animated model. Read across a row to distinguish material belonging to different phases at overlapping locations.',
'- **phases/*-marks.png** registers every visible marking in the corresponding phase panel as a 16-bit grayscale mark number (0 is background). **marks/** provides a readable black-and-white footprint and surrounding paint crop per mark. The index gives mark number, camera, origin in the phase panel, paint and translucent status. A bare axis name denotes an isolated mark exposed for inspection. Phase panels use 1.5 pixels per model unit with an 8-pixel margin on the orthographic box.',
'- **game-marks/** shows the detail-gate footprints on its sixteen game-size cells. The left panel is the client image and orange in the right panel shows pixels of a mark that meets the visibility test. The matching 16-bit mark map uses the same numbers as marks/. An all-zero map means no qualifying opaque mark in that cell.',
'',f"The detail gate finds {detailres['visible']} qualifying opaque marks across the game-size sample. The mark catalogue contains {len(mark_index)} candidate regions, including translucent ones.",'','## Measurements','',*stats]
    material_reading=('Branda’s 120-byte band crosses the breastplate, brow and book. Bytes 140–220 layer progressively smaller pieces of the crown flame. Bytes 50 and 75 pick out lower ornaments and weapon accents.' if n==12596 else 'Eldric’s 1,209 byte-60 faces form much of the body and weapon. Byte 40 defines the head, byte 30 the eyes, and bytes 100 and 150 overlap in the elevated ice effect. His 2,090 ordinary translucent faces account for 67.55% of the stored mesh.')
    notes=['- **tlight and tink** follow the foremost translucent surface. Tlight stores the ordinary surface’s light number in grey. Tink shows paint boundaries in white, transparency changes within one paint in amber, and the silhouette in grey.',
           '- **marks/catalog** pages pair each cropped paint region with its binary mask. Numbers match the phase and game images. The crops are enlarged independently. Use their index and phase panel for scale.',
           '- '+material_reading]
    if n==12596:
        notes+=['- **death31-wide.png** gives the complete silhouette for the death frame at yaw 1088 that touches the edge of its registered pose cell. **wide-cameras.json** also records the wider cameras used for four summon pictures in actions/.']
    sheetdoc[sheetdoc.index('## Measurements'):sheetdoc.index('## Measurements')]=notes+['']
    md(sh/'SHEET.md',sheetdoc)
    for suffix in 'ab':
        folder=BUILDS/f'titan-rebuild-{n}-{suffix}'
        folder.mkdir(exist_ok=True)
        cmd=_AP.fix(f'<PYTHON> -I -B {ROOT.as_posix()}/teacher_gate.py rebuild-npc {n} {folder.as_posix()}/submission.json')
        md(folder/'README.md',[f"# {npc['name']} · blind rebuild {suffix.upper()}",'',f'Use the pictures and measurements in [the version 3 sheet](../tools-titan-sheets/sheets/{n}/SHEET.md). Keep the source binary, private fixtures, gate outputs and other authors’ work outside the reference set. Construct plates and markings before painting. Deliver your definition as submission.json here, with its authored model files beside it.','','The Teacher will run the following command in Git Bash. It scores all five attacks, stand, walk and death, the opaque small-mark line and the solid-body paint/light split.','','```bash',cmd,'```'])
    report+=['## '+npc['name'],'',*stats,material_reading,'','### Gate calibration','']
    checks=[]
    for attack in ATTACKS[n]:
        g=read(base/'gates'/f'ceiling-{attack}.json')
        assert g['code']==0
        assert len(g['cells'])==50 and all(c['differ']==0 and c['overlap']==1 for c in g['cells'])
        diff=max(c['differ'] for c in g['cells'])
        ds=read(base/'tone/decomp'/f'ceiling-{attack}.json')['cells']
        S=sum(row['all']['solid'] for row in ds); tot=sum(row['all']['pixels'] for row in ds)
        vals=[max(abs(row['all'].get(k,0)) for row in ds) for k in ('tone_S','paint_alone','light_alone')]
        checks.append([attack,'PASS',diff,f'{S:,} / {tot:,}',*vals])
    report+=table(['Attack','Stock gate','Max motion-row differing share','Solid / shared samples¹','Max solid tone','Max paint alone','Max light alone'],checks)
    report+=['','¹ The decomp tool skips empty shared silhouettes. A sample is one pixel of one frame and camera. The last three columns are maxima over measured cells. The stock gate uses six sampled frames per sequence and two yaws. Complete lines for every invocation follow in the appendix.','',
    f"**Small marks** · {detailres['verdict']}, {detailres['missing']:.1f} missing of {detailres['visible']} visible. Recall {detailres['recall']:.3f}, spurious {detailres['spurious']:.1f}. "+('Zero qualifying opaque marks makes this line vacuous for Eldric.' if n==14147 else 'All 45 retained marks have a qualifying footprint in at least one game-size cell.'),'','### Damaged controls','']
    def checksfile(case):
        return read(base/'ordinary-alpha/gate.json' if case=='ordinary-alpha' else base/'gates'/f'{case}-{ATTACKS[n][0]}.json')
    damages=[]
    for case,line,desc in [('labels','real labels ours lacks',f"Reassign every vertex on label {inv['part']} to the most common label"),('ordinary-alpha','standing picture: tone difference (grey levels of 255)','Remove ordinary transparency, preserve 254 and 255'),('paint','colour distance',f"Shift {inv['paint_faces']} faces attached by majority to label {inv['part']} by one packed lightness unit"),('alpha','standing picture: tone difference (grey levels of 255)','Additional stress control, also clear special byte 254')]:
        g=checksfile(case);item=next(r for r in g['checks'] if r['name']==line)
        px=next(r['value'] for r in g['checks'] if r['name']=='attack: share of pixels differing')
        damages.append([desc,gate_npc.LEVELS[g['code']],line,str(item['value']),px])
    report+=table(['Single change','Verdict','Diagnostic line','Value','Attack differing share'],damages)
    report+=['','A positive colour distance and changed pixels catch the one-step edit against the exact-copy baseline. The stock thresholds accept that edit on a rebuild. The missing-label line is informational. The extra alpha stress run changes a special lighting operation as well as ordinary blending.','']
    appendix += [f'## {npc["name"]} · full calibration output','']
    for f in [*(base/'gates').glob('ceiling-*.txt'),*(base/'gates').glob('labels-*.txt'),base/'ordinary-alpha/gate.txt',*(base/'gates').glob('paint-*.txt'),*(base/'gates').glob('alpha-*.txt')]:
        appendix += [f'### {f.parent.name} / {f.name}','','```text',f.read_text().rstrip(),'```','']
    appendix+=['### Opaque detail','','```text',(ROOT/f'detail-{n}.log').read_text().rstrip(),'```','','### Solid-body decomposition, all attacks','','```text',(ROOT/f'tone-{n}.log').read_text().rstrip(),'```','']
report+=['## Rebuild risks and evidence','',
'Both bind views contain off-body equipment or effects that the animations move into place. Branda has a separate fire-coloured piece beside her hand. Eldric carries an elevated radial ice effect, a book and layered ice body surfaces. These appear with the NPC alone. The four floor sequences per base are separate object assets. The pictures do not reconstruct the encounter’s external spot effects, projectiles or floor choreography. The phase panels separate sampled visibility classes and the action gallery shows each playable NPC sequence.','',
'Ordinary alpha covers most of Eldric’s raw mesh. A one-layer orthographic composite cannot reproduce a stack of blended surfaces. The comparison board and pose-set lit panels use the actual client compositor. The solid pixel decomposition deliberately excludes front translucency and undecodable pixels. The opaque detail line excludes translucent marks. Each measures a different subset of his appearance.','',
'The encoder path is the same meshio/native_codec path used by reencode_npc.py. Readback checked every decoded field against its intended fixture, including alpha, labels, priority and depth. The revision-239 harness drew the real cache entry beside each encoded definition. All generated bytecode, fixtures and diagnostic outputs used local storage; the asset build tree and the supplied Cuttle directory were outside Git. New build folders supplied isolation. No shared tool source needed editing.','',
'## Command failures, recovered checks and limits','',
'- Initial direct-shell inventory batches beginning with ls and cat were routed to the Windows command interpreter. They failed before reading the files. Repeated through Git Bash with an explicit login-shell invocation.',
'- The initial target-directory ls/glob and both git rev-parse probes found uncreated output directories and non-Git locations. The first census rg probes lost literal quotes in the Windows command wrapper. Subsequent structured JSON reads recovered the full inventory. One rg filter expanded its unquoted alternation and printed unrelated filename matches. No conclusions use that filter.',
'- The early sheets/*/pose.json probe ran before the two pose jobs finished. Final manifests each contain 96 cells.',
'- The preview helper initially used inspect.py and shadowed Python’s standard library inspect. Renamed it to preview_refs.py and reran successfully.',
'- The first two phase runs reached image export and Pillow rejected signed 32-bit RGB. Converted the palette image to unsigned 8-bit RGB and reran both phase builds.',
'- Visual readback showed the inherited rest lighting sampled smooth vertex normals on flat Eldric faces. Regenerated both rest sheets with light_npc’s integer flat-face rule. The first comparison camera clipped the upper ice effect. Reframed all nine cells with common settings and verified a background border on every cell.',
'- Game-mark overlay review found that the detail tool stores yaw and row in a combined view key. Regenerated the masks from that key and both visibility thresholds. The two Eldric AOE names were absent from the project extract because gameval spells the king “edric” there. The full cache table resolves both.',
'','The control results measure sensitivity at the real-asset ceiling. Human blind-lineup performance remains unmeasured until authors deliver their rebuilds.','','# Calibration transcripts','',*appendix]
report[2]=report[2].replace('a limb label','a part’s label')
report[report.index('## Rebuild risks and evidence'):report.index('## Rebuild risks and evidence')]=[
    'The ten ceiling runs cover 500 scored cells including repeated bind, stand, walk and death controls. Full-precision per-cell readback confirms zero differing RGB pixels in every cell, including bind. Overlap is 1.0 throughout. Each solid-body paint and light ceiling is zero. Across the five runs Eldric has only about 1–3% solid, decodable samples, depending on attack.','',
    'Sheet verification reopened all PNGs, checked the 18 registered views and 96 four-layer poses per titan, matched the nine board crops and source columns, and checked each binary mark mask against its measured pixel count. Branda has 68 candidate mark masks and Eldric 210. The union of game-size qualified masks reproduces 45 and zero respectively.','']
report[report.index('# Calibration transcripts'):report.index('# Calibration transcripts')]=[
    'Further readback found four summon previews and one registered Branda death cell touching the image border. The summon previews were reframed, and the sheet includes an additional wider death picture so that the original four pose layers stay registered. Per-cell data were initially omitted from the gate JSON by a mismatched dictionary key; the complete gate runs were repeated and verified against the unrounded measurements.','']
md(ROOT/'REPORT.md',report)

