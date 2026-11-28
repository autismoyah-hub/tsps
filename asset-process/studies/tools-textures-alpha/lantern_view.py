import sys
import tempfile
from pathlib import Path
from PIL import Image
here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parents[1]/'tools'))
import scene, author
board=Image.new('RGB',(960,320))
with tempfile.TemporaryDirectory(dir=here) as tmp:
    mid=1246000
    content=scene.build(Path(tmp)/'scene',None,[],[(mid,(here/'lantern.bin').read_bytes(),author.object_def(mid),20,60,0)])
    for i,yaw in enumerate((0,2048,14336)):
        p=scene.capture(content,f'view{i}',[20.5,60.5,yaw,330,170,37])
        with Image.open(p) as im:
            board.paste(im.crop((480,290,800,610)),(i*320,0))
board.save(here/'lantern-classic.png')
