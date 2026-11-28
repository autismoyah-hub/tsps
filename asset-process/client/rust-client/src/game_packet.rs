//! Revision-specific game packet payloads, separate from ISAAC framing.
use crate::{
    buffer::{Reader, Transform},
    protocol::Packet,
    vm::{Array, Object, Value},
    Error, Result,
};
use std::{cell::RefCell, rc::Rc};
pub struct ScriptCall {
    pub id: i32,
    pub arguments: Vec<Value>,
}
impl ScriptCall {
    /// GameClient's jc.ea branch reverses signature order on the wire.
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let signature = r.string()?;
        if signature.chars().count() > 1000 {
            return Err(Error::Invalid("script argument count"));
        }
        let mut arguments = Vec::new();
        for kind in signature.chars().rev() {
            arguments.push(match kind {
                's' => Value::Object(Object::String(r.string()?)),
                'Ï' => Value::Long(r.u64()? as i64),
                'W' | 'X' => {
                    let count = r.varint_le()? as usize;
                    if count > 65535 || count > r.remaining() {
                        return Err(Error::Invalid("script packet array length"));
                    }
                    let array = if kind == 'W' {
                        Array::Ints(
                            (0..count)
                                .map(|_| r.varint_le_signed())
                                .collect::<Result<Vec<_>>>()?,
                        )
                    } else {
                        Array::Strings(
                            (0..count)
                                .map(|_| r.string().map(Some))
                                .collect::<Result<Vec<_>>>()?,
                        )
                    };
                    Value::Object(Object::Array(Rc::new(RefCell::new(array))))
                }
                _ => Value::Int(r.u32()? as i32),
            });
        }
        arguments.reverse();
        let id = r.u32()? as i32;
        if r.remaining() != 0 {
            return Err(Error::Invalid("script packet trailing bytes"));
        }
        Ok(Self { id, arguments })
    }
    pub fn split(self) -> (i32, Vec<i32>, Vec<Object>, Vec<i64>) {
        let (mut ints, mut objects, mut longs) = (Vec::new(), Vec::new(), Vec::new());
        for arg in self.arguments {
            match arg {
                Value::Int(v) => ints.push(v),
                Value::Long(v) => longs.push(v),
                Value::Object(v) => objects.push(v),
            }
        }
        (self.id, ints, objects, longs)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameMessage {
    pub kind: u16,
    pub sender: Option<String>,
    pub text: String,
}
/// GameClient's jc.dx / LOGOUT_TRANSFER payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldHop {
    pub host: String,
    pub world: u16,
    pub flags: u32,
}
pub enum Update {
    OpenUrl(String),
    SiteSettings(String),
    Uid192(Option<[u8;24]>),
    AmbientStart { id: u16, fade: bool },
    AmbientStop { fade: bool },
    Skybox(i32),
    Heatmap(bool),
    GroupFull(Vec<u8>),
    GroupVariables(Vec<crate::group_state::Delta>),
    InteractionPolicy(crate::interaction_policy::Update),
    ServerString(String),
    MarketOffer { slot: u8, offer: crate::market::Offer },
    TradingPost(Option<crate::market::Snapshot>),
    NpcHeadIcon { index:u16, slot:u8, group:i32, image:i16 },
    LocationAnimation(crate::location_animation::Specific),
    NpcSequence { index: u16, id: i32, delay: u8 },
    PlayerSequence { id: i32, delay: u8 },
    Music(crate::music_request::Request),
    Camera(crate::camera::Update),
    Ping([u32; 2]),
    Logout(u8),
    WorldHop(WorldHop),
    Zone(crate::zone::Packet),
    ActorSpot(crate::actor_spot::Update),
    WorldSpot(crate::world_spot::Request),
    Projectile(crate::projectile::Request),
    ServerTick,
    ResetAnimations,
    DialogAbort,
    WorldEntities(Vec<u8>),
    WorldEntityRebuild(Vec<u8>),
    Hint(Vec<u8>),
    HideOperations {
        kind: u8,
        hidden: bool,
    },
    PlayerOperation {
        slot: u8,
        low_priority: bool,
        label: Option<String>,
    },
    ChatMessage(crate::chat_packet::Incoming),
    ClanProfileReset(bool),
    ClanProfileValue(Vec<u8>),
    ClanSettingsDelta {
        slot: i8,
        delta: crate::clan_delta::Delta,
    },
    ClanChannelDelta {
        slot: i8,
        delta: crate::clan_channel::Delta,
    },
    ClanChannel {
        slot: i8,
        channel: Option<crate::clan_channel::Channel>,
    },
    ClanSettings {
        slot: i8,
        settings: Option<crate::clan_settings::Settings>,
    },
    FriendLoading,
    FriendsChat {
        snapshot: bool,
        payload: Vec<u8>,
    },
    Friends(Vec<crate::social::FriendUpdate>),
    Ignores(Vec<crate::social::IgnoreUpdate>),
    SynthSound {
        id: u16,
        loops: u8,
        delay: u16,
    },
    ChatFilters {
        public: u8,
        trade: u8,
    },
    PrivateChatFilter(u8),
    NpcOrigin([u8; 2]),
    ActiveWorld {
        id: u16,
        plane: u8,
    },
    Message(GameMessage),
    NpcInfo {
        large: bool,
        payload: Vec<u8>,
    },
    PlayerInfo(Vec<u8>),
    Stat {
        index: u8,
        boosted: u8,
        invisible: u8,
        experience: i32,
    },
    RunEnergy(u16),
    RunWeight(i16),
    Reboot {
        message: String,
        cycles: u16,
    },
    Minimap(u8),
    MapFlag(u32),
    Inventory(crate::inventory::Update),
    Interface(crate::interface_packet::Update),
    Varp {
        id: u16,
        value: i32,
    },
    VarpSync,
    VarpReset,
    Script(ScriptCall),
}
pub fn decode(packet: &Packet) -> Result<Option<Update>> {
    if let Some(request)=crate::music_request::Request::packet(packet)? {return Ok(Some(Update::Music(request)));}
    if let Some(update) = crate::camera::decode(packet)? {
        return Ok(Some(Update::Camera(update)));
    }
    if packet.opcode==113 {return Ok(Some(Update::Uid192(crate::uid192::decode(&packet.payload)?)));}
    let mut r = Reader::new(&packet.payload);
    // Original URL parsing accepts trailing payload bytes after the first NUL.
    if packet.opcode == 39 { return Ok(Some(Update::OpenUrl(r.string()?))); }
    if packet.opcode == 67 { return Ok(Some(Update::SiteSettings(r.string()?))); }

    let update = match packet.opcode {
        145 => Update::AmbientStart { fade: r.transformed(Transform::Add)? == 1, id: r.u16()? },
        138 => Update::AmbientStop { fade: r.u8()? == 1 },
        26 => Update::Heatmap(r.u8()? & 1 != 0),
        135 => Update::Skybox(r.u32_inverse_middle()? as i32),
        132 => return Ok(Some(Update::GroupFull(packet.payload.clone()))),
        133 | 136 | 140 => {
            if (packet.opcode == 133 && packet.payload.len() != 13)
                || (packet.opcode == 140 && packet.payload.len() != 9) {
                return Err(Error::Invalid("group variable packet length"));
            }
            return Ok(Some(Update::GroupVariables(crate::group_state::Delta::decode(&packet.payload)?)));
        }
        66 | 79 => Update::InteractionPolicy(crate::interaction_policy::Update::decode(packet.opcode, &mut r)?),
        24 => Update::ServerString(r.string()?),
        53 => { let (slot, offer) = crate::market::Offer::slot(&mut r)?; Update::MarketOffer { slot, offer } },
        56 => Update::TradingPost(crate::market::Snapshot::decode(&mut r)?),
        4 => Update::NpcHeadIcon { index:r.u16_add(false)?, slot:r.transformed(Transform::Sub)?, image:r.u16_le()? as i16, group:r.u32_middle()? as i32 },
        20 => {
            let delay = r.transformed(Transform::Neg)?;
            let index = r.u16_add(false)?;
            let id = r.u16()?;
            Update::NpcSequence { index, id: if id == 65535 { -1 } else { id as i32 }, delay }
        }
        119 => {
            let delay = r.transformed(Transform::Neg)?;
            let id = r.u16_le()?;
            Update::PlayerSequence { id: if id == 65535 { -1 } else { id as i32 }, delay }
        }
        30 => Update::LocationAnimation(crate::location_animation::Specific::decode(&mut r)?),
        115 => Update::Ping([r.u32()?, r.u32()?]),
        57 => Update::Logout(0),
        58 => Update::Logout(r.u8()?),
        91 => Update::WorldHop(WorldHop { host: r.string()?, world: r.u16()?, flags: r.u32()? }),
        92 => Update::ResetAnimations,
        117 => Update::DialogAbort,
        122 => return Ok(Some(Update::WorldEntities(packet.payload.clone()))),
        109 => return Ok(Some(Update::WorldEntityRebuild(packet.payload.clone()))),
        21 | 73 | 75 => Update::HideOperations {
            kind: match packet.opcode {
                21 => 0,
                73 => 1,
                _ => 2,
            },
            hidden: r.u8()? & 1 != 0,
        },
        6 => {
            let low_priority = r.u8()? == 0;
            let slot = r.transformed(Transform::Add)?;
            let label = r.string()?;
            Update::PlayerOperation {
                slot,
                low_priority,
                label: if label.eq_ignore_ascii_case("null") {
                    None
                } else {
                    Some(label)
                },
            }
        }

        15 | 19 | 61 | 62 | 127 => {
            return Ok(Some(Update::ChatMessage(
                crate::chat_packet::Incoming::decode(packet.opcode, &packet.payload)?,
            )))
        }
        60 => Update::ClanProfileReset(false),
        88 => Update::ClanProfileReset(true),
        11 => return Ok(Some(Update::ClanProfileValue(packet.payload.clone()))),
        71 => {
            let slot = r.u8()? as i8;
            let delta = crate::clan_delta::Delta::decode(r.bytes(r.remaining())?)?;
            Update::ClanSettingsDelta { slot, delta }
        }
        95 => {
            let slot = r.u8()? as i8;
            let delta = crate::clan_channel::Delta::decode(r.bytes(r.remaining())?)?;
            Update::ClanChannelDelta { slot, delta }
        }
        69 => {
            let slot = r.u8()? as i8;
            let channel = if r.remaining() == 0 {
                None
            } else {
                Some(crate::clan_channel::Channel::decode(
                    r.bytes(r.remaining())?,
                )?)
            };
            Update::ClanChannel { slot, channel }
        }
        81 => {
            let slot = r.u8()? as i8;
            let settings = if r.remaining() == 0 {
                None
            } else {
                Some(crate::clan_settings::Settings::decode(
                    r.bytes(r.remaining())?,
                )?)
            };
            Update::ClanSettings { slot, settings }
        }
        27 | 112 => {
            return Ok(Some(Update::FriendsChat {
                snapshot: packet.opcode == 27,
                payload: packet.payload.clone(),
            }))
        }
        36 => Update::FriendLoading,
        111 => {
            return Ok(Some(Update::Friends(crate::social::decode_friends(
                &packet.payload,
            )?)))
        }
        42 => {
            return Ok(Some(Update::Ignores(crate::social::decode_ignores(
                &packet.payload,
            )?)))
        }
        77 => Update::SynthSound {
            id: r.u16()?,
            loops: r.u8()?,
            delay: r.u16()?,
        },
        124 => Update::ChatFilters {
            public: r.transformed(Transform::Add)?,
            trade: r.transformed(Transform::Sub)?,
        },
        5 => Update::PrivateChatFilter(r.u8()?),
        116 => Update::NpcOrigin([r.u8()?, r.u8()?]),
        47 => Update::ActiveWorld {
            id: r.u16()?,
            plane: r.u8()?,
        },
        74 => {
            let kind = r.unsigned_smart()?;
            let sender = if r.u8()? == 1 {
                Some(r.string()?)
            } else {
                None
            };
            Update::Message(GameMessage {
                kind,
                sender,
                text: r.string()?,
            })
        }
        31 => Update::RunWeight(r.u16()? as i16),
        37 => Update::Reboot {
            message: r.string()?,
            cycles: r.u16()?,
        },
        43 => Update::Minimap(r.u8()?),
        46 => {
            let invisible = r.u8()?;
            let boosted = r.u8()?;
            let index = r.transformed(Transform::Add)?;
            if index >= 25 {
                return Err(Error::Invalid("stat index"));
            }
            Update::Stat {
                index,
                boosted,
                invisible,
                experience: r.u32_inverse_middle()? as i32,
            }
        }
        48 => Update::Projectile(crate::projectile::Request::specific(&mut r)?),
        72 | 101 => Update::ActorSpot(crate::actor_spot::Update::decode(packet.opcode, &mut r)?),
        90 => Update::WorldSpot(crate::world_spot::Request::specific(&mut r)?),
        64 => Update::RunEnergy(r.u16()?),
        83 => Update::ServerTick,
        9 | 10 | 13 | 33 | 34 | 41 | 52 | 54 | 70 | 78 | 86 | 104 | 105 | 110 => {
            return Ok(Some(Update::Zone(crate::zone::Packet::decode(
                packet.opcode,
                &packet.payload,
            )?)))
        }
        50 => Update::Hint(r.bytes(6)?.to_vec()),
        143 => Update::MapFlag(r.u32()?),
        28 => return Ok(Some(Update::PlayerInfo(packet.payload.clone()))),
        2 | 85 => {
            return Ok(Some(Update::NpcInfo {
                large: packet.opcode == 2,
                payload: packet.payload.clone(),
            }))
        }
        12 => Update::Varp {
            id: r.u16_add(false)?,
            value: r.u32_le()? as i32,
        },
        97 => {
            let value = r.transformed(Transform::Add)? as i8 as i32;
            Update::Varp {
                id: r.u16_add(true)?,
                value,
            }
        }
        44 => Update::VarpReset,
        76 => Update::VarpSync,
        114 => return Ok(Some(Update::Script(ScriptCall::decode(&packet.payload)?))),
        _ => {
            if let Some(u) = crate::inventory::decode(packet)? {
                return Ok(Some(Update::Inventory(u)));
            }
            return Ok(crate::interface_packet::decode(packet)?.map(Update::Interface));
        }
    };
    if r.remaining() != 0 {
        return Err(Error::Invalid("game packet trailing bytes"));
    }
    Ok(Some(update))
}
