"""Play the authored timeline with fight.py's scripts, actors, clocks and cache renderer."""
from pathlib import Path
import tempfile,sys
from PIL import Image,ImageDraw
sys.dont_write_bytecode = True
D=Path(__file__).resolve().parent
sys.path.insert(0,str(D.parent/'research-effects2/fight'))
import fight
F=fight.load(D/'fight-arena.json')
out=Path(F['output'])
out.mkdir(exist_ok=True)
tempfile.tempdir=str(out)
assert not fight.check(F),fight.check(F)
def panel(timeline,cycles,name,probe=False):
    stage=F["stage"]
    body=[]; animations=[[],[]]; actors=[]
    for tick,attack in timeline:
        lines,events,end=fight.script(F,[attack],origin=stage["place"],start=tick*30)
        actors=[l.split() for l in lines[:2]]
        for i,a in enumerate(actors):animations[i]+= [v for v in a if v.startswith('seq=')]
        for line in lines[2:]:
            if line.startswith('fproj ') and attack=='star_lance':
                shot=next(b for b in F['attacks'][attack]['beats'] if b['do']=='projectile')
                dx,dy=shot['wristTileFromNpcBase']
                words=line.split()
                words[2]=f"{int(actors[0][2])+dx},{int(actors[0][3])+dy}"
                line=' '.join(words)
            body.append(line)
        if attack=='phase_change':
            phase=F['phase']
            for c in range(tick*30+48+90,cycles,90):
                a=F['assets'][phase['attachment']]
                body.append(f"faspot n1 {fight.apath(F,a)} {phase['height']} {c}")
    for i,a in enumerate(actors):
        a=[v for v in a if not v.startswith('seq=')]
        a[4]='0' if i==0 else '1024'
        body.insert(i,' '.join(a+animations[i]))
    head=['size '+' '.join(map(str,stage['size'])),f"focal {stage['focal']}",'camera '+' '.join(map(str,stage['camera'])),'pin']
    with tempfile.TemporaryDirectory(dir=out) as tmp:
        times=list(range(0,cycles,5)) if not probe else [0,30,60,90,115]
        script=head+body+[f'render {",".join(map(str,times))} {Path(tmp).as_posix()}/f']
        log=fight.fx.run(script,str(D/F['cache']))
        assert 'animated map objects started' in log
        frames=fight.fx.frames(Path(tmp).as_posix()+'/f',times)
    labels={'star_lance':'Star Lance','meridian_line':'Meridian Line','orbit':'Orbit','phase_change':'Half health'}
    for frame,c in zip(frames,times):
        t,attack=next((t,a) for t,a in reversed(timeline) if t*30<=c)
        elapsed=c-t*30
        detail={'star_lance':('draw' if elapsed<30 else 'flight' if elapsed<90 else 'impact' if elapsed<132 else ''),'meridian_line':('leave the marked row' if elapsed<90 else 'strike' if elapsed<138 else ''),'orbit':('setting the dial' if elapsed<66 else 'avoid its tile' if elapsed<300 else 'release'),'phase_change':('ignition' if elapsed<48 else 'the guiding star stays lit')}[attack]
        d=ImageDraw.Draw(frame);d.rectangle((0,0,608,23),fill=(18,24,36))
        d.text((9,4),f'{c/50:04.1f}s    {labels[attack]}    {detail}',fill=(242,226,183),font=fight.fx.font(14))
    if probe:
        strip=Image.new('RGB',(3*304,2*216))
        for i,im in enumerate(frames):strip.paste(im.resize((304,216)),(304*(i%3),216*(i//3)))
        strip.save(out/(name+'-probe.jpg'));return
    dest=out/(name+'.gif')
    frames[0].save(dest,save_all=True,append_images=frames[1:],duration=100,loop=0)
    (out/(name+'.log')).write_text('\n'.join(script[:-1])+f'\n{len(frames)} frames; 100 ms per frame.\n')
    sheet=Image.new('RGB',(4*304,3*216))
    for i in range(12):sheet.paste(frames[round((len(frames)-1)*i/11)].resize((304,216)),(i%4*304,i//4*216))
    sheet.save(out/(name+'.png'))
    with Image.open(dest) as check:
        duration=0
        for i in range(check.n_frames):check.seek(i);duration+=check.info['duration']
        assert duration==cycles*20,(name,duration)
    print(name,duration,flush=True)
if __name__=='__main__':
    if 'probe' in sys.argv:panel([(0,'meridian_line')],150,'meridian',True)
    else:
        for name in sys.argv[1:] or list(F['attacks'])+['encounter']:
            if name=='encounter':panel(F['timeline'],1500,name)
            else:panel([(0,name)],dict(star_lance=155,meridian_line=155,orbit=350,phase_change=190)[name],name)
