from pathlib import Path
from PIL import Image, ImageDraw
root=Path(__file__).resolve().parent
page=Image.new('RGB',(1000,1000),(50,50,52))
for i,n in enumerate((12596,14147)):
    sh=root/'sheets'/str(n)
    for k,p in enumerate([sh/f'archive-{n}.png',sh/'enlargements/head.png',sh/'enlargements/hand-grip.png',sh/'enlargements/hand-weapon.png',sh/'alpha-bands.png']):
        with Image.open(p) as im:
            im.thumbnail((490,185))
            page.paste(im,(500*i,k*200))
            ImageDraw.Draw(page).text((500*i,k*200+183),p.name)
page.save(root/'teacher-only/review.png')

