//! Independent literal issue84 oracle. No exporter/JSON expectation input.
//! ROOT MODEL SKELETON FRAMES SEQUENCE LOCATION [PRIVATE_CACHE]
//! Optional cache mode reads exact staged bytes through maintained DiskCache.
use gielinor_client::{animation::{Frame, Sequence, Skeleton}, animation_runtime::Clock,
    disk_cache::DiskCache, model::Model, object::Definition};
use std::{fs, path::PathBuf};

const REST: [[f32;3];8] = [[0.,0.,0.],[128.,0.,0.],[112.,0.,64.],[0.,0.,64.],
    [0.,-128.,0.],[96.,-128.,0.],[80.,-128.,48.],[0.,-128.,64.]];
const TURN: [[f32;3];8] = [[0.,0.,0.],[128.,0.,0.],[112.,0.,64.],[0.,0.,64.],
    [-48.,-49.,0.],[-48.,-144.,0.],[-48.,-128.,48.],[-48.,-49.,64.]];
const SHIFT: [[f32;3];8] = [[0.,0.,0.],[128.,0.,0.],[112.,0.,64.],[0.,0.,64.],
    [-40.,-49.,-16.],[-40.,-144.,-16.],[-40.,-128.,32.],[-40.,-49.,48.]];

fn main() {
    let args:Vec<_> = std::env::args().skip(1).collect();
    assert!(args.len()==6 || args.len()==7, "ROOT MODEL SKELETON FRAMES SEQUENCE LOCATION [PRIVATE_CACHE]");
    let root=PathBuf::from(&args[0]);
    let ids:Vec<u32>=args[1..6].iter().map(|v|v.parse().unwrap()).collect();
    assert!(ids.iter().all(|v|(1..=65534).contains(v)));
    let mut cache=args.get(6).map(|p|DiskCache::open(p,64<<20).unwrap());
    let mut read=|name:&str,archive:u8,group:u32,file:u32| {
        let expected=fs::read(root.join(name)).unwrap();
        assert!(expected.len()<=1<<20);
        if let Some(c)=cache.as_mut() {
            let actual=c.file(archive,group,file,[0;4]).unwrap();
            assert_eq!(actual,expected,"cache byte identity {archive}/{group}/{file}");
            actual
        } else {expected}
    };
    let model=Model::decode(&read("model.bin",7,ids[0],0)).unwrap();
    assert_eq!(model.vertices,REST);
    assert_eq!(model.faces,vec![[0,2,1],[0,3,2],[4,5,6],[4,6,7],[0,1,5],[0,5,4],
        [1,2,6],[1,6,5],[2,3,7],[2,7,6],[3,0,4],[3,4,7]]);
    assert_eq!(model.colors,vec![43051,43051,8084,8084,43051,8084,43051,8084,43051,8084,43051,8084]);
    assert_eq!(model.vertex_skins,Some(vec![0,0,0,0,1,1,1,1]));
    assert!(model.face_textures.is_none() && model.bone_weights.is_none());
    let skeleton=Skeleton::decode(ids[1] as u16,&read(&format!("skeleton-{}.bin",ids[1]),1,ids[1],0)).unwrap();
    assert_eq!(skeleton.transform_types,vec![0,2,1]);
    assert_eq!(skeleton.labels,vec![vec![],vec![1],vec![1]]);
    let sequence=Sequence::decode(ids[3],&read("sequence.bin",2,12,ids[3])).unwrap();
    assert_eq!(sequence.frames,vec![ids[2]<<16,(ids[2]<<16)|1,(ids[2]<<16)|2]);
    assert_eq!(sequence.lengths,vec![50,75,100]);
    assert_eq!((sequence.loop_frames,sequence.max_loops),(-1,1));
    assert!(sequence.cached_animation.is_none());
    for i in 0..3 {
        let raw=read(&format!("frame-{i}.bin"),0,ids[2],i);
        assert_eq!(u16::from_be_bytes([raw[0],raw[1]]),ids[1] as u16,"frame skeleton header");
        let frame=Frame::decode(&raw,&skeleton).unwrap();
        assert!(!frame.has_alpha);
        assert_eq!(frame.transforms.len(),3);
        let expected=[[16,-64,8],[0,0,if i==0 {0}else{64}],if i==2 {[8,0,-16]}else{[0;3]}];
        for (index,t) in frame.transforms.iter().enumerate() {
            assert_eq!(t.label,index); assert_eq!(t.values,expected[index]);
        }
        let mut posed=model.clone();
        posed.animate_classic(&skeleton,&frame,false).unwrap();
        assert_eq!(posed.vertices,[REST,TURN,SHIFT][i as usize],"pose{i}");
        println!("POSE {i} vertices={:?}",posed.vertices);
    }
    // Fresh clocks: strict > boundary, no floating time/wall-clock oracle.
    for (ticks,frame,elapsed,flags) in [(0,0,0,8),(50,0,50,8),(51,1,1,12),
        (125,1,75,12),(126,2,1,12),(225,2,100,12),(226,0,1,15)] {
        let mut clock=Clock::default(); let (actual,_)=clock.advance(&sequence,ticks).unwrap();
        assert_eq!((clock.frame,clock.elapsed,actual),(frame,elapsed,flags));
        println!("CLOCK ticks={ticks} frame={frame} elapsed={elapsed} flags={flags} stopped={}",clock.stopped);
    }
    let definition=Definition::decode_local(ids[4],&read("location.bin",2,6,ids[4])).unwrap();
    assert_eq!(definition.models,vec![ids[0]]);
    assert_eq!(definition.animation,Some(ids[3] as u16));
    assert!(!definition.randomize_animation);
    println!("NATIVE_DECODE_ONLY_PASS cache_readback={} no client capture or artistic acceptance",args.len()==7);
}
