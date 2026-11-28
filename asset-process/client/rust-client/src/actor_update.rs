//! Shared actor synchronization blocks. Delays are relative to receipt cycle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    pub kind: u16,
    pub value: u16,
    pub delay: u16,
    pub limit: u16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Headbar {
    pub kind: u16,
    pub timing: Option<[u16; 2]>,
    pub fill: [u8; 2],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FaceTarget {
    Entity {
        kind: u16,
        index: i32,
        fallback: u16,
    },
    Tile {
        x: u16,
        z: u16,
        size: [u8; 2],
    },
    Angle(u16),
    Reset,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Face {
    pub target: FaceTarget,
    pub turn_on_walk: bool,
    pub instant: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Customisation {
    pub models: Option<Vec<u32>>,
    pub recolor: Option<Vec<u16>>,
    pub retexture: Option<Vec<u16>>,
    pub use_player: bool,
    pub composition: Option<(u8, Vec<u16>)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    Body(Option<Customisation>),
    Head(Option<Customisation>),
    Combat(i32),
    HeadIcons(Option<[(i32, i32); 8]>),
    Ops(u8),
    Hits(Vec<Hit>),
    Headbars(Vec<Headbar>),
    ExactMove {
        offsets: [i8; 4],
        delays: [u16; 2],
        direction: u16,
    },
    Sequence {
        id: u16,
        delay: u8,
    },
    Say(String),
    Name(String),
    Transform(u16),
    Tint {
        delays: [u16; 2],
        hsl: [u8; 3],
        weight: u8,
    },
    Transparency {
        delays: [i16; 2],
        alpha: [i8; 2],
        use_start: bool,
    },
    BaseAnimations([Option<u16>; 15]),
    Freeze {
        delay: u16,
        duration: u16,
        cancel_sequence: bool,
    },
    SpotAnimations(Vec<(u8, u16, u16, u16)>),
    Face(Face),
    /// Fields read and discarded by the revision 239 client.
    Discarded {
        mask: u32,
        bytes: Vec<u8>,
    },
}
