"""Compare the exported-file renderer to Icons::get using the exact bytes from ten real cache models."""
import pathlib,sys,tempfile
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from finish_items import *
def main():
    log=OUT/'file-icons.txt';log.write_text(subprocess.list2cmdline([sys.executable,'-I',__file__])+'\n')
    ids=[1303,3204,1381,1127,7462,1079,4131,1147,6570,9747]
    with tempfile.TemporaryDirectory(dir=OUT) as t:
        folder=pathlib.Path(t);specs=[]
        for id in ids:
            d=L.items()[id];model=folder/f'{id}.bin';definition=folder/f'{id}.def'
            model.write_bytes(L.osrscache.unpack(L.osrscache.group(7,d['model'])))
            definition.write_bytes(C.encode(d))
            specs.extend([f'item:{id}',f'def:{definition}+model:{model}'])
        for bg in ('000000','ff00ff'):
            bmp=folder/'controls.bmp'
            run([L.RESEARCH_ITEMS,L.CACHE,'icons',bmp,'bg='+bg,*specs],log)
            im=np.array(Image.open(bmp)).reshape(32,len(ids),2,36,3)
            diff=np.any(im[:,:,0]!=im[:,:,1],axis=3).sum(axis=(0,2)).tolist()
            with log.open('a') as fp:print('background',bg,'per-item differences',diff,'total',sum(diff),'of',1152*len(ids),file=fp)
            assert not any(diff)
if __name__=='__main__':main()

