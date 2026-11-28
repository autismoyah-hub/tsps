"""film.py: the whole set in one moving picture: the Armillary sentinel (NPC 16360, its own skeleton and sequences)
attacks a player with the five original effects, BY ID from the packed cache through the client's ordinary path
(asset_effect.exe: NPC and player through world_actors, their sequences by actor_sequence, the effects started as the
three packets start them).
usage: film.py [test]     -> render/research-effects-set/set-fight.gif (+ set-fight-sheet.png, film.log)
Timeline in client cycles (20 ms; 30 = one game tick):
   20  sentinel attack (sequence 20102, 56 cycles)
   44  ring-bolt 4100, sentinel -> player, 50 cycles, start height 190, end 120, slope 10, offset 110
   94  star-burst 4101 on the player (height 100); the player flinches (sequence 424)
  130  meridian-mark 4102 on the tile in front of the player: 120 cycles = 4 ticks of warning
  232  sentinel attack again, so the clamp closes as the mark runs out
  250  meridian-slam 4103 on the same tile
  262  star-pool 4104 on the same tile (240 cycles; a server would send it again for a longer hazard)
"""
import sys, pathlib
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import fx
from fxlib import *
from PIL import Image, ImageDraw

PACKED = str(HERE / 'cache-packed')
OUT = RENDER / 'research-effects-set'; OUT.mkdir(parents=True, exist_ok=True)
x, y = fx.PLACE
SY, PY = y + 3, y - 3          # the sentinel (two tiles wide) north on the road, the player south; seen from the side
TY = y - 1                      # the slam tile, in front of the player
test = len(sys.argv) > 1
W, H = (640, 440)
L = [f'size {W} {H}', 'focal 640', f'camera {x + .5} {y + .5} 4096 1250 780 80',
     f'npc 16360 {x - 1} {SY} 0 seq=20102@20 seq=20102@232',
     f'player {x - 1} {PY} 0 1024 {fx.PLAYER_ITEMS} seq=424@94',
     f'proj 4100 n1 p1 190 120 44 94 10 110',
     f'aspot p1 4101 100 94',
     f'spot 4102 {x - 1} {TY} 0 130',
     f'spot 4103 {x - 1} {TY} 0 250',
     f'spot 4104 {x - 1} {TY} 0 262']
cyc = [10, 40, 60, 80, 100, 110, 150, 200, 245, 256, 270, 290, 340, 400] if test else list(range(0, 430, 2))
L.append(f'render {",".join(map(str, cyc))} {OUT.as_posix()}/f')
out = fx.run(L, PACKED, keep_log=OUT / 'film.log')
ims = fx.frames(f'{OUT.as_posix()}/f', cyc)
EVENTS = [(0, 'the sentinel stands'), (20, 'attack: it fires'), (44, 'a  ring-bolt (projectile 4100)'), (94, 'b  star-burst (impact 4101)'), (130, 'c  meridian-mark (telegraph 4102): 4 ticks'),
          (250, 'd  meridian-slam (hit 4103)'), (262, 'e  star-pool (hazard 4104)')]


def caption(im, c):
    d = ImageDraw.Draw(im)
    text = [t for at, t in EVENTS if at <= c][-1]
    d.rectangle((0, 0, im.width, 20), fill=(16, 18, 22)); d.text((6, 3), f'cycle {c:3d}  ({c * 0.02:4.1f} s)   {text}', fill=(240, 240, 240), font=fx.font(13))
    return im


ims = [caption(im, c) for im, c in zip(ims, cyc)]
if test:
    sh = Image.new('RGB', (W * 5 // 2, (H // 2) * 3))
    for k, im in enumerate(ims[:15]): sh.paste(im.resize((W // 2, H // 2)), ((k % 5) * (W // 2), (k // 5) * (H // 2)))
    sh.save(OUT / 'set-fight-test.png'); print(OUT / 'set-fight-test.png')
else:
    ims[0].save(OUT / 'set-fight.gif', save_all=True, append_images=ims[1:] + [ims[-1]] * 12, duration=40, loop=0, optimize=False)
    pick = [10, 30, 42, 50, 62, 80, 100, 115, 124, 128, 131, 137, 150, 190]
    sh = Image.new('RGB', (W * 5 // 2, (H // 2) * 3))
    for k, i in enumerate(pick[:15]): sh.paste(ims[i].resize((W // 2, H // 2)), ((k % 5) * (W // 2), (k // 5) * (H // 2)))
    sh.save(OUT / 'set-fight-sheet.png')
    print(OUT / 'set-fight.gif', len(ims), 'frames of 2 cycles')
