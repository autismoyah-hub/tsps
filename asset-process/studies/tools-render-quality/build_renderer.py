"""Build the optional NPC lens adapter from the installed client. Entirely offline."""
from pathlib import Path
from tempfile import TemporaryDirectory
import json
import shutil
import subprocess

HERE = Path(__file__).resolve().parent
CLIENT = HERE.parents[1] / "client/rust-client"
BUILD = CLIENT / "target/release/build"
EXAMPLES = CLIENT / "examples"

def main():
    libs = sorted(BUILD.glob("*/*/out/*.rlib"))
    client = next(p for p in libs if p.name.startswith("libgielinor_client") and
                  json.loads(next((p.parents[1] / "fingerprint").glob("lib-*.json")).read_text())["features"] == '["default"]')
    with TemporaryDirectory(dir=HERE) as temp:
        src = Path(temp)
        for name in ("asset_npc.rs", "asset_pose.rs"):
            shutil.copyfile(EXAMPLES / name, src / name)
        helper = Path("support/actor_draw.rs")
        code = (EXAMPLES / helper).read_text()
        original = "[0.0, 0.0, 1.0], 512.0)"
        assert code.count(original) == 1
        code = code.replace(original, '[0.0, 0.0, 1.0], std::env::var("NPC_REVIEW_FOCAL")?.parse::<f32>()?)')
        (src / helper).parent.mkdir()
        (src / helper).write_text(code)
        deps = [arg for folder in sorted({p.parent for p in libs}) for arg in ("-L", "dependency=" + str(folder))]
        subprocess.run(["rustc", "+stable-x86_64-pc-windows-gnu", "--edition=2021", "-C", "opt-level=3", "-C", "overflow-checks=yes",
                        *deps, "--extern", "gielinor_client=" + str(client), str(src / "asset_npc.rs"), "-o", str(HERE / "quality_npc.exe")], check=True)
        # A separate authored snapshot accepts the review raster dimensions and lens.
        snapshot = (EXAMPLES / "authored_scene_snapshot.rs").read_text()
        substitutions = {
            "target,1000.)": 'target,std::env::var("REVIEW_FOCAL").unwrap().parse().unwrap())',
            "Frame::new(1280,900)": 'Frame::new(std::env::var("REVIEW_W")?.parse()?,std::env::var("REVIEW_H")?.parse()?)',
        }
        for before, after in substitutions.items():
            assert snapshot.count(before) == 1
            snapshot = snapshot.replace(before, after)
        (src / "snapshot.rs").write_text(snapshot)
        shutil.copyfile(EXAMPLES / "support/local_triangle_dump.rs",src / "support/local_triangle_dump.rs")
        custom = next(p for p in libs if p.name.startswith("libgielinor_client") and p != client)
        sha2 = next(p for p in libs if p.name.startswith("libsha2-"))
        subprocess.run(["rustc", "+stable-x86_64-pc-windows-gnu", "--edition=2021", "-C", "opt-level=3", "-C", "overflow-checks=yes",
                        *deps, "--cfg", 'feature="custom-assets"', "--extern", "gielinor_client="+str(custom),
                        "--extern", "sha2="+str(sha2), str(src / "snapshot.rs"), "-o", str(HERE / "quality_scene.exe")], check=True)

if __name__ == "__main__":
    main()
