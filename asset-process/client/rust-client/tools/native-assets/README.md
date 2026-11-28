# Owned asset conversion

These optional Python tools export original, owned geometry to formats accepted
by the maintained Rust decoder. They do not access the game cache, reserve IDs,
install packages or run in the player client. Python's standard library is enough.

- `convert_glb.py`: embedded glTF 2 GLB with static indexed triangles, opaque
  constant colors and closed, consistently oriented geometry. Node transforms,
  reflected winding and explicit material `extras.native_render_type` (0 or 1)
  are preserved. Textures, skins, animation and morph targets are rejected.
- `native_codec.py`: explicit extended model channels, simple texture mappings,
  labels, depth flags, and classic skeleton/frame/sequence exports. Weighted bones
  and complex texture mappings are unsupported. Invalid action inputs are
  validated before any output files are written.
- `mesh_validation.py`: explicitly requested removal of zero-area owned faces,
  preserving vertex identities. It is not an automatic repair step.

The GLB conversion uses 128 native units per tile, maps glTF XYZ to `[x,-y,-z]`,
and rounds to integer coordinates. Exporting evaluated fractional vertices through
the native model API is rejected rather than silently quantized. Closed winding
does not establish absence of self-intersections or globally outward normals.

Run `python convert_glb.py owned.glb output-directory`. The historical output
filename remains `archway.model`; the other outputs record expected geometry and
hashes. No object/model namespace is assigned. Use a separate output directory:
successful exports replace their named output files. Filesystem errors during
writing can still leave an incomplete export; this is not a transactional package
installer.

Run `python -m unittest discover -s tools/native-assets -v` from the repository.
For a decoder roundtrip, first build the client library, then pass its exact
`target/release/deps/libgielinor_client-*.rlib` to
`python tools/native-assets/verify_roundtrip.py LIBRARY`. The matching `rustc`
must be on PATH, or supplied using `--rustc`. This verifies synthetic original
model channels and classic animation bytes with the maintained Rust decoders;
it does not require Blender, authentic cache assets or reference gamepacks.

Source identities and acceptance scope are in
`docs/consolidation/native-asset-tools-evidence.json`. Blender interchange and
historical fixed-ID package recipes remain cataloged separately; they have not
been installed into this tooling directory.

## Numeric terrain observation audit

`terrain_audit.py` compares explicit JSON corner/deck data with complete CSV
observations. It accepts only plain SE–NW paint triangles and flat linked decks;
it neither creates native terrain nor launches a client. Its report does not
certify collision, the origin of supplied observations or gameplay acceptance.

From the workspace root:

```text
python rust-client/tools/native-assets/terrain_audit.py --source sample-grid.json --corners decoded-corners.csv --samples native-samples.csv --output new-audit.json
python -m unittest discover -s rust-client/tools/native-assets -p test_terrain_audit.py -v
```

Each input is limited to 1 MiB, dimensions to 1–64 cells per axis. Missing,
duplicate, unexpected or malformed observations fail; an empty audit cannot
pass. Ground/deck corners and upper flags compare exactly; signed surface
elevation permits one native rounding unit. Valid mismatches produce a report
and exit 1; malformed evidence or publication failure exits 2; agreement exits 0.
The report includes raw input SHA-256 values, counts and sorted mismatches, with
no timestamps or input paths. It is published without overwriting an existing
output; the output filesystem must support hard links (otherwise it fails closed).

See [the map pipeline method](../../../docs/tower/map-pipeline.md) for the full
schema, synthetic sample, height conventions, attribution and deferred server
atlas work. The model codec's decoder roundtrip above does not validate terrain.
