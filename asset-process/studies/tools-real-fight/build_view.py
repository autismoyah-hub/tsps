"""Route arena_view's packet inputs to its selected plane and add filmed NPC state changes."""
from pathlib import Path
import os
import subprocess

HERE = Path(__file__).resolve().parent
CLIENT = HERE.parents[1] / "client/rust-client"
RUST = Path(os.environ["USERPROFILE"]) / ".rustup/toolchains/stable-x86_64-pc-windows-gnu"

def build():
    source = (CLIENT / "examples/arena_view.rs").read_text(encoding="utf-8")
    for old, new in [
        ("let mut pl = Player::at([0,", "let mut pl = Player::at([pl as i32,"),
        ("position: [0, l[2]", "position: [pl as i32, l[2]"),
        ("=> [0, t[0]", "=> [pl as i32, t[0]"),
        ("; [0, q[", "; [pl as i32, q["),
        ("let key = [0, tile[0]", "let key = [pl as i32, tile[0]"),
        ('"plane" => {}', '"plane" | "form" | "step" => {}'),
        ("for (at, who, seq) in &schedule {", """
        for l in &lines {
            if !matches!(l[0], "form" | "step") || l[1].parse::<i32>()? != c { continue; }
            let npc = client.npc_state.npcs.get_mut(&npc_index[0]).unwrap();
            if l[0] == "form" {
                let id: u16 = l[2].parse()?;
                let def = client.npcs.get(&(id as u32)).ok_or("unknown form")?;
                npc.id = id;
                npc.animations = def.animations;
            } else {
                let tile = [l[2].parse()?, l[3].parse()?];
                npc.position = tile;
                npc.path.push(tile, 1);
                npc.angle = l[4].parse()?;
                npc.turn.angle = npc.angle;
                npc.turn.target = npc.angle;
            }
        }
        for (at, who, seq) in &schedule {"""),
    ]:
        assert old in source, old
        source = source.replace(old, new)
    path = HERE / "arena_film.rs"
    path.write_text(source, encoding="utf-8")
    libroot = CLIENT / "target/release/build"
    libs = sorted(libroot.glob("*/*/out/*.rlib"))
    flags = []
    for folder in sorted({lib.parent for lib in libs}):
        flags.extend(["-L", f"dependency={folder}"])
    env = os.environ.copy()
    env.update(TMP=str(HERE), TEMP=str(HERE))
    for lib in (p for p in libs if p.name.startswith("libgielinor_client")):
        result = subprocess.run([str(RUST / "bin/rustc.exe"), "--edition=2021", "-C", "opt-level=3", "-C", "overflow-checks=yes", str(path), *flags, "--extern", f"gielinor_client={lib}", "-o", str(HERE / "arena_film.exe")], env=env)
        if result.returncode == 0:
            return
    raise SystemExit("Could not link renderer")

if __name__ == "__main__":
    build()
