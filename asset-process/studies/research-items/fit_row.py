"""Refine a view in renderer space. Test neighbourhoods of zoom and both signed offsets
together. Search direction is measured, including high-pitch views whose Y offset is inverted."""
import pathlib,sys,re,tempfile,subprocess,itertools,json,statistics
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
import itemlib as L,itemcodec as C
def fit(model):
    model=pathlib.Path(model).resolve()
    path=model.with_suffix('.view.json');v=json.loads(path.read_text())
    target=v['target_box'];x0,x1,y0,y1=target
    w,h=x1-x0+1,y1-y0+1
    def score(box):
        _,a,b,c,d=box
        edge=sum([a==0 and x0>0,b==35 and x1<35,c==0 and y0>0,d==31 and y1<31])
        scale=max((b-a+1)/w,(d-c+1)/h)
        return 3*edge+0.6*(abs(a+b-x0-x1)+abs(c+d-y0-y1))+24*abs(scale-1)+0.12*(abs(b-a+1-w)+abs(d-c+1-h))
    start=(v['zoom'],*v['offset']);best=start
    got={}
    with tempfile.TemporaryDirectory(dir=model.parent) as temp:
        t=pathlib.Path(temp)
        for factor in (0.08,0.04,0.02):
            step=max(1,round(best[0]/512*(3 if factor==0.08 else 1)))
            candidates=[(round(best[0]*(1+z*factor)),best[1]+dx*step,best[2]+dy*step) for z,dx,dy in itertools.product((-1,0,1),repeat=3)]
            new=list(dict.fromkeys(vv for vv in candidates if vv not in got))
            if new:
                specs=[]
                for i,(z,x,y) in enumerate(new):
                    f=t/f'{i}.def'
                    f.write_bytes(C.encode(dict(v,model=1000100,zoom=z,offset=[x,y])))
                    specs.append(f'def:{f.name}+model:{model}')
                args=[L.RESEARCH_ITEMS,L.CACHE,'icons','trial.bmp',*specs]
                proc=subprocess.run(args,cwd=t,capture_output=True,text=True,check=True)
                boxes=re.findall(r': (\d+) pixels drawn, box x (\d+)\.\.(\d+) y (\d+)\.\.(\d+)',proc.stdout)
                assert len(boxes)==len(new)
                got.update(zip(new,[tuple(map(int,b)) for b in boxes]))
            best=min([best,*candidates],key=lambda vv:(score(got[vv]),sum(abs(a-b) for a,b in zip(vv,start))))
        v.update(zoom=best[0],offset=list(best[1:]),final_box=list(got[best][1:]),final_pixels=got[best][0])
        print(f'{len(got)} renderer trials: {start} {got[start]} -> {best} {got[best]}, target {target}')
    path.write_text(json.dumps(v,indent=1))
if __name__=='__main__':fit(sys.argv[1])

