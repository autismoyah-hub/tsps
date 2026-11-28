"""Verify synthetic owned exports against an explicitly selected client library."""
import argparse
from pathlib import Path
import subprocess
import tempfile

import convert_glb
import native_codec


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('library', type=Path, help='Built libgielinor_client-*.rlib')
    parser.add_argument('--rustc', default='rustc')
    args = parser.parse_args()
    library = args.library.resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix='native-owned-codecs-') as folder:
        root = Path(folder)
        geometry = dict(vertices=[[0, 0, 0], [128, -64, 0], [0, 0, 128]],
                        faces=[[0, 1, 2]], colors=[43051], render_types=[1])
        (root / 'legacy.bin').write_bytes(convert_glb.encode(
            geometry['vertices'], geometry['faces'], geometry['colors'], geometry['render_types']))
        geometry.update(vertex_skins=[0, 1, 2], face_skins=[3], alpha=[17],
                        priorities=[7], face_flags=[1], texture_types=[0],
                        texture_triangles=[[0, 1, 2]], face_textures=[23],
                        texture_coords=[0], metadata=[[1, 2, 3], 4])
        (root / 'extended.bin').write_bytes(native_codec.model(geometry))
        frames = [dict(types=[0, 1, 2, 3, 5], labels=[[0], [1], [2], [3], [4]],
                       length=length, transforms=[[i, [i+j, i-j, i*2]] for i in range(5)])
                  for j, length in enumerate([2, 5])]
        native_codec.classic(dict(frames=frames, skeleton_id=123, frame_group=456,
                                 policy=dict(loop_frames=2, interleave=[1, 3], stretch=True,
                                             priority=7, max_loops=4, precedence=1,
                                             priority_moving=2, restart=1)), root / 'action')
        executable = root / 'verify.exe'
        subprocess.run([args.rustc, '--edition=2021', str(Path(__file__).with_name('verify_roundtrip.rs')),
                        '--extern', f'gielinor_client={library}', '-L', f'dependency={library.parent}',
                        '-o', str(executable)], check=True)
        subprocess.run([str(executable), str(root)], check=True)


if __name__ == '__main__':
    main()
