import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
from patchlib import Patch

# ---- asset_pose.rs: aim= --------------------------------------------------------------------------------------
P = Patch(_AP.fix(r'<CLIENT_SRC>\examples\asset_pose.rs'))
rep = P.rep
rep("""//!                   [under=SEQ:FRAME] [center=body|model] [draw=actor|depth|auto|widget] ENTRY [ENTRY...]
//!        asset_pose CACHE info""", """//!                   [under=SEQ:FRAME] [center=body|model] [aim=head|body|legs|feet|hands|HEIGHT]
//!                   [draw=actor|depth|auto|widget] ENTRY [ENTRY...]
//!        asset_pose CACHE info""")
rep("""//! pixel; center=model frames each cell on its own rest mesh, which is what asset_figure does.
""", """//! pixel; center=model frames each cell on its own rest mesh, which is what asset_figure does.
//! aim=HEIGHT (worn-space height in units, feet 0, crown 196) or aim=head (180) | body (140) | hands (100) |
//! legs (65) | feet (10) points the camera at that height of the figure instead of the middle of the bare figure
//! (98). Every cell still shares the one camera, so columns and rows register pixel for pixel; with a small
//! zoom= this gives a close view of a helm or of boots. aim= overrides center=.
""")
rep("""fn half_height(m: &Model) -> i32 {""", """/// aim=: a named region of the standard figure or a height in worn space (units above the feet).
fn aim_height(v: &str) -> R<i32> {
    Ok(match v {
        "head" => 180,
        "body" | "torso" | "chest" => 140,
        "hands" => 100,
        "legs" => 65,
        "feet" => 10,
        _ => v.parse::<i32>().map_err(|_| format!("aim={v}: use head, body, hands, legs, feet or a height in units"))?,
    })
}
fn half_height(m: &Model) -> i32 {""")
rep("""[under=SEQ:FRAME] [center=body|model] [draw=actor|depth|auto|widget] ENTRY [ENTRY...]".into());""",
    """[under=SEQ:FRAME] [center=body|model] [aim=head|body|legs|feet|hands|HEIGHT] [draw=actor|depth|auto|widget] ENTRY [ENTRY...]".into());""")
rep("""    let mut center_model = false;
""", """    let mut center_model = false;
    let mut aim: Option<i32> = None;
""")
rep("""        else if let Some(v) = e.strip_prefix("draw=") { draw = actor_draw::Draw::parse(v)?; }
        else { entries.push(e.clone()); }""", """        else if let Some(v) = e.strip_prefix("aim=") { aim = Some(aim_height(v)?); }
        else if let Some(v) = e.strip_prefix("draw=") { draw = actor_draw::Draw::parse(v)?; }
        else { entries.push(e.clone()); }""")
rep("""    println!("draw={draw:?}");
    let mut painter = actor_draw::Painter::load(&mut cache)?;
    if entries.is_empty()""", """    println!("draw={draw:?}");
    if let Some(h) = aim { println!("aim: camera pointed at height {h} (every cell shares it)"); }
    let mut painter = actor_draw::Painter::load(&mut cache)?;
    if entries.is_empty()""")
rep("""            let rest_half = half_height(if center_model { &m } else { &bare });""",
    """            let rest_half = aim.unwrap_or_else(|| half_height(if center_model { &m } else { &bare }));""")
P.save()

# ---- pose.py: --aim, --tone -----------------------------------------------------------------------------------
P = Patch(_AP.fix(r'<ROOT>\tools\pose.py'))
rep = P.rep
rep("""               [--cell WxH] [--under SEQ:FRAME] [--center body|model] [--overlap A:B[,A:C...]]
               [--diff A:B[,A:C...]] [--keep-bmp] [--draw actor|depth|auto|widget]""",
    """               [--cell WxH] [--under SEQ:FRAME] [--center body|model] [--aim head|body|legs|feet|hands|HEIGHT]
               [--overlap A:B[,A:C...]] [--diff A:B[,A:C...]] [--tone A:B[,A:C...]] [--keep-bmp]
               [--draw actor|depth|auto|widget]""")
rep("""--overlap A:B measures""", """--aim points the camera at a part of the figure instead of its middle (height 98): a height in worn-space units
  (feet 0, crown 196) or a name: head = 180, body = 140, hands = 100, legs = 65, feet = 10. All columns and rows
  still share the one camera, so --diff, --overlap and --tone work as before. Use it with a small --zoom for a
  close view of a helm or boots, which otherwise leave the cell (--zoom alone keeps aiming at the body):
     pose.py helm.png "item:1163,model:path/to/helm.bin@head+hair+jaw,none" --aim head --zoom 170
     pose.py boots.png "item:11840,model:path/to/boots.bin@feet,none" --aim feet --zoom 170
  A zoom under 400 gives 600 x 800 cells; at zoom 170 a unit is about 3 pixels (about 1 at the default 520).
--overlap A:B measures""")
rep("""  the number of differing pixels, their share of the cell, and the largest channel difference. 0 = identical.
""", """  the number of differing pixels, their share of the cell, and the largest channel difference. 0 = identical.
  It counts ANY difference, so it does not move between a wrong tone and a nearly right one; for that use --tone.
--tone A:B measures, per row, how far apart two columns are in TONE. It needs a `none` column, like --overlap.
  The pixels measured are those where column A or column B differs from the `none` column (the worn piece in
  either column; the bare figure and the background are left out). Over them it prints
     the count of such pixels,
     the mean absolute grey difference (grey = (r + g + b) / 3, 0 to 255): 0 = the same picture,
     the signed mean difference, B minus A: positive = column B is lighter than column A, negative = darker,
     and how many of the pixels differ at all.
  A right shape in a slightly wrong tone gives a small mean (one lightness step of paint is about 1 grey level
  on a lit surface, less on an unlit one) with a signed mean of nearly the same size; a wrong shape gives a
  large mean with a signed mean near 0. Follow the signed mean to 0 while tuning paint.
""")
rep("""for k in ('seq', 'frames', 'zoom', 'yaw', 'pitch', 'cell', 'under', 'center', 'draw') if opt('--' + k)]""",
    """for k in ('seq', 'frames', 'zoom', 'yaw', 'pitch', 'cell', 'under', 'center', 'aim', 'draw') if opt('--' + k)]""")
rep("""d = ImageDraw.Draw(im)
try: font""", """if opt('--tone'):
    if 'none' not in entries:
        raise SystemExit('--tone needs a `none` entry: the pixels measured are those differing from it')
    px = im.load(); base = entries.index('none')
    grey = lambda p: (p[0] + p[1] + p[2]) / 3
    for pair in opt('--tone').split(','):
        a, b = (int(v) - 1 for v in pair.split(':'))
        for row, f in enumerate(frames):
            n = changed = 0; total = signed = 0.0
            for y in range(ch):
                for x in range(cw):
                    p, q, o = px[a * cw + x, row * ch + y], px[b * cw + x, row * ch + y], px[base * cw + x, row * ch + y]
                    if p != o or q != o:
                        n += 1; changed += p != q
                        dd = grey(q) - grey(p); signed += dd; total += abs(dd)
            report.append(f'tone col {a + 1} vs col {b + 1} frame {f}: {n} worn pixels, mean absolute grey difference '
                          f'{total / n if n else 0:.2f}, signed mean (col {b + 1} minus col {a + 1}) {signed / n if n else 0:+.2f} '
                          f'of 255; {changed} of them differ')
d = ImageDraw.Draw(im)
try: font""")
rep("""draw {opt("--draw") or "actor"}' + (f' under {opt("--under")}' if opt('--under') else '')""",
    """draw {opt("--draw") or "actor"}' + (f' under {opt("--under")}' if opt('--under') else '') + (f' aim {opt("--aim")}' if opt('--aim') else '')""")
P.save()
