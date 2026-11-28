"""Independent literal numeric contract for issue84; no exporter/cache/Blender imports.

The quarter-turn pose below is hand-declared, including native fixed-point
rounding (sine65535, cosine0). It is not generated from exported observations.
Agreement is a numeric check, never cache admission or rendered acceptance.
"""
import argparse
import copy
import json
from pathlib import Path

REST = [[0,0,0],[128,0,0],[112,0,64],[0,0,64],
        [0,-128,0],[96,-128,0],[80,-128,48],[0,-128,64]]
FACES = [[0,2,1],[0,3,2],[4,5,6],[4,6,7],[0,1,5],[0,5,4],
         [1,2,6],[1,6,5],[2,3,7],[2,7,6],[3,0,4],[3,4,7]]
TURN = [[0,0,0],[128,0,0],[112,0,64],[0,0,64],
        [-48,-49,0],[-48,-144,0],[-48,-128,48],[-48,-49,64]]
SHIFT = [[0,0,0],[128,0,0],[112,0,64],[0,0,64],
         [-40,-49,-16],[-40,-144,-16],[-40,-128,32],[-40,-49,48]]

def expected():
    frames = []
    for length, rotation, translation in [(50,0,[0,0,0]),(75,64,[0,0,0]),(100,64,[8,0,-16])]:
        frames.append(dict(length=length, types=[0,2,1], labels=[[],[1],[1]],
                           transforms=[[0,[16,-64,8]],[1,[0,0,rotation]],[2,translation]]))
    return copy.deepcopy(dict(schema=1, units_per_tile=128, axes='blender=(x,z,-y)/128',
        model=dict(vertices=REST, faces=FACES, colors=[43051,43051,8084,8084]+[43051,8084]*4,
                   vertex_skins=[0,0,0,0,1,1,1,1], priority=0),
        action=dict(frames=frames, policy=dict(loop_frames=-1,max_loops=1)),
        preview=[REST,TURN,SHIFT]))

def _compare(actual, wanted, path):
    if type(actual) is not type(wanted):
        raise ValueError(path+': type mismatch')
    if isinstance(wanted, dict):
        if actual.keys() != wanted.keys():
            raise ValueError(path+': missing or unexpected keys')
        for key in wanted:
            _compare(actual[key], wanted[key], path+'.'+key)
    elif isinstance(wanted, list):
        if len(actual) != len(wanted):
            raise ValueError(path+': length mismatch')
        for i,(value,reference) in enumerate(zip(actual,wanted)):
            _compare(value,reference,f'{path}[{i}]')
    elif actual != wanted:
        raise ValueError(path+': value mismatch')

def validate(observation):
    _compare(observation, expected(), 'observation')

def load(path):
    path=Path(path)
    with path.open('rb') as source:
        raw=source.read((1<<20)+1)
    if len(raw)>1<<20:
        raise ValueError('observation exceeds1MiB')
    def pairs(rows):
        result={}
        for key,value in rows:
            if key in result:
                raise ValueError('duplicate JSON key')
            result[key]=value
        return result
    return json.loads(raw,object_pairs_hook=pairs,
                      parse_constant=lambda value: (_ for _ in ()).throw(ValueError('nonfinite JSON')))

def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('observation',type=Path)
    args=parser.parse_args(argv)
    try:
        validate(load(args.observation))
    except (ValueError, OSError) as error:
        parser.exit(2,f'numeric check failed: {error}\n')
    print('NUMERIC_ONLY_PASS: rest, labels, channels, three poses and duration declarations; no cache/native proof')

if __name__=='__main__':
    main()
