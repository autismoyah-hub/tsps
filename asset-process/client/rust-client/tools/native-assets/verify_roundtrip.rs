use gielinor_client::model::Model;
fn main() {
    let root = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    for name in ["legacy", "extended"] {
        let m = Model::decode(&std::fs::read(root.join(format!("{name}.bin"))).unwrap()).unwrap();
        assert_eq!(
            m.vertices,
            vec![[0., 0., 0.], [128., -64., 0.], [0., 0., 128.]]
        );
        assert_eq!(m.faces, vec![[0, 1, 2]]);
        assert_eq!(m.colors, vec![43051]);
        assert_eq!(m.render_types, Some(vec![1]));
        if name != "legacy" {
            assert_eq!(m.vertex_skins, Some(vec![0, 1, 2]));
            assert_eq!(m.face_skins, Some(vec![3]));
            assert_eq!(m.alpha, Some(vec![17]));
            assert_eq!(m.priorities, Some(vec![7]));
            assert_eq!(m.face_flags, Some(vec![1]));
        }
        if name == "extended" {
            assert_eq!(m.face_textures, Some(vec![23]));
            assert_eq!(m.texture_coords, Some(vec![0]));
            assert_eq!(m.texture_types, vec![0]);
            assert_eq!(m.texture_triangles, vec![[0, 1, 2]]);
            assert_eq!(m.metadata, Some(([1, 2, 3], 4)));
        }
        println!("{name}: all authored channels match maintained Rust decoder");
    }
    use gielinor_client::animation::{Frame, Sequence, Skeleton};
    let root = root.join("action");
    let s = Skeleton::decode(123, &std::fs::read(root.join("skeleton-123.bin")).unwrap()).unwrap();
    assert_eq!(s.transform_types, vec![0, 1, 2, 3, 5]);
    assert_eq!(s.labels, vec![vec![0], vec![1], vec![2], vec![3], vec![4]]);
    for j in 0..2 {
        let f = Frame::decode(
            &std::fs::read(root.join(format!("frame-{j}.bin"))).unwrap(),
            &s,
        )
        .unwrap();
        assert_eq!(f.skeleton, 123);
        assert!(f.has_alpha);
        assert_eq!(f.transforms.len(), 5);
        for (i, t) in f.transforms.iter().enumerate() {
            assert_eq!(t.label, i);
            let i = i as i16;
            assert_eq!(t.values, [i + j, i - j, i * 2]);
        }
    }
    let q = Sequence::decode(999, &std::fs::read(root.join("sequence.bin")).unwrap()).unwrap();
    assert_eq!(q.frames, vec![456 << 16, (456 << 16) | 1]);
    assert_eq!(q.lengths, vec![2, 5]);
    assert_eq!(q.loop_frames, 2);
    assert_eq!(q.interleave, Some(vec![1, 3, 9999999]));
    assert!(q.stretch);
    assert_eq!(
        (
            q.priority,
            q.max_loops,
            q.precedence,
            q.priority_moving,
            q.restart
        ),
        (7, 4, 1, 2, 1)
    );
    println!("classic action: skeleton, transforms, frame timing and policy match maintained Rust decoder");
}
