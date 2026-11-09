"""Bounded read-only named-artifact inventory; no provider or credentials."""
import fnmatch,json,os,stat,time
from pathlib import Path
ROOTS=('/tmp/zogre-hf-case','/home/ubuntu/.zogre-recovery-42689c','/workspace/repo-fix-zogre-uglug-sale')
PATTERNS=('case-collection-once','supervisor-once','collection-intent.json','collection-readback.json','collection-unknown.json','collection-partial*','status-*.json','logs-*.txt','records-*.jsonl','decoded-*','collect-launch-stdout.txt','collect-launch-stderr.txt','collect-launch-closure.json','collect-controller-*','recovery-controller-*','recovery-collection-*','recovery-launch-stdout.txt','recovery-launch-stderr.txt','recovery-launch-closure.json')
SKIP={'.git','node_modules','caches','dist','.yarn','build'}
MAX_DEPTH=4;MAX_ENTRIES=100000

def main():
    result={'utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'roots':[],'scope':'named artifacts only, max depth 4; symlink directories and listed large trees excluded; other roots/hosts unknown'}
    for value in ROOTS:
        root=Path(value);row={'root':value,'hits':[],'visited':0,'skipped':[],'errors':[],'truncated':False}
        stack=[(root,0)]
        while stack:
            path,depth=stack.pop()
            try:
                with os.scandir(path) as it:
                    for item in it:
                        row['visited']+=1
                        if row['visited']>MAX_ENTRIES:row['truncated']=True;break
                        if any(fnmatch.fnmatchcase(item.name,p) for p in PATTERNS):row['hits'].append(os.path.relpath(item.path,root))
                        try:
                            if item.is_dir(follow_symlinks=False):
                                if depth>=MAX_DEPTH or item.name in SKIP:row['skipped'].append(os.path.relpath(item.path,root))
                                else:stack.append((Path(item.path),depth+1))
                        except OSError:row['errors'].append('ENTRY_STAT_ERROR')
            except OSError:row['errors'].append('DIRECTORY_READ_ERROR')
            if row['truncated']:break
        row['hits'].sort();row['skippedCount']=len(row.pop('skipped'));result['roots'].append(row)
    print(json.dumps(result,sort_keys=True))
if __name__=='__main__':main()
