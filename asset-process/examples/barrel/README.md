# Worked example: a model written by script

`build.py` writes a barrel in the game's model format with `tools/author.py`: one vertex per position, smooth
shading, faces turned outward, 16-bit HSL face colours, edges made by tight rings and a tone step, shading painted
with lightness steps inside one hue, open underneath. Its design is our own; its proportions follow the rules
measured from real barrels. Run `python -I examples/barrel/build.py`; it writes `barrel.bin` beside itself, which
`tools/lineup.py` can stand beside real barrels. The Blender route (`tools/blender_kit.py`, then
`tools/blender_export.py`) gives the same model face for face.
