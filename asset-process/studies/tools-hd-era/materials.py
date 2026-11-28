"""Read evaluated HD materials from the local procedural-client bake."""
from hdcache import ROOT,lru_cache
from PIL import Image
import zipfile,io,json
@lru_cache(maxsize=3)
def atlas(rev):return zipfile.ZipFile(ROOT/f'textures-{rev}.zip')
@lru_cache(maxsize=3)
def metadata(rev):return json.loads(atlas(rev).read('materials.json'))
def preview(c,tid):
 data=atlas(c.rev).read(f'{tid}.png')
 return Image.open(io.BytesIO(data)).convert('RGBA'),None

