//! World-entity movement state and revision 239 coordinate encoding.
use crate::{buffer::Reader, Error, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Definition {
    pub id: i32,
    pub name: String,
    pub category: i32,
    pub interactable: bool,
    pub operations: [Option<String>; 5],
    pub scene_offset: [i32; 2],
    pub default_animation: i32,
    pub flags: u8,
    pub sprite: i32,
    pub color: u16,
    pub picking_type: u8,
    pub collision_type: u8,
    /// X/Z offset and width/depth, in local-coordinate units.
    pub bounds: [i32; 4],
}
impl Definition {
    pub fn decode(id: i32, data: &[u8]) -> Result<Self> {
        let mut definition = Self {
            id,
            name: "null".into(),
            category: -1,
            interactable: false,
            operations: Default::default(),
            scene_offset: [0; 2],
            default_animation: -1,
            flags: 0,
            sprite: -1,
            color: 39188,
            picking_type: 2,
            collision_type: 2,
            bounds: [0; 4],
        };
        let mut r = Reader::new(data);
        loop {
            match r.u8()? {
                0 => break,
                2 => definition.flags = r.u8()?,
                op @ 4..=5 => definition.scene_offset[(op - 4) as usize] = r.u16()? as i16 as i32,
                op @ 6..=7 => definition.bounds[(op - 6) as usize] = r.u16()? as i16 as i32,
                op @ 8..=9 => definition.bounds[(op - 6) as usize] = r.u16()? as i32,
                12 => definition.name = r.string()?,
                14 => definition.interactable = true,
                op @ 15..=19 => {
                    let text = r.string()?;
                    definition.operations[(op - 15) as usize] =
                        (!text.eq_ignore_ascii_case("Hidden")).then_some(text);
                    definition.interactable = true;
                }
                20 => definition.category = r.u16()? as i32,
                23 => definition.picking_type = r.u8()?,
                24 => definition.collision_type = r.u8()?,
                25 => definition.default_animation = r.u16()? as i32,
                26 => {
                    definition.sprite = if r.clone().u8()? & 128 != 0 {
                        (r.u32()? & 0x7fffffff) as i32
                    } else {
                        let value = r.u16()?;
                        if value == 32767 {
                            -1
                        } else {
                            value as i32
                        }
                    };
                }
                27 => definition.color = r.u16()?,
                // The original constructor ignores unassigned one-byte tags.
                _ => {}
            }
        }
        Ok(definition)
    }
}

/// Original bj.fa opcode3339; -1 does not load a definition.
pub fn script_query(
    opcode: u16,
    stacks: &mut crate::vm::Stacks,
    mut name: impl FnMut(i32) -> Result<String>,
) -> Result<bool> {
    if opcode != 3339 {
        return Ok(false);
    }
    let id = stacks.pop_int()?;
    let value = if id == -1 { String::new() } else { name(id)? };
    stacks.push_object(crate::vm::Object::String(value))?;
    Ok(true)
}
/// Shared lazy definitions for both spawned child worlds and script lookups.
pub fn load_definition<'a>(
    cache: &mut crate::disk_cache::DiskCache,
    definitions: &'a mut std::collections::HashMap<i32, Definition>,
    id: i32,
) -> Result<&'a Definition> {
    if !definitions.contains_key(&id) {
        // GameIndexDataBase.bu rejects negative file IDs before group recovery.
        // A sentinel configuration therefore uses the Java constructor defaults.
        let bytes = if id < 0 { vec![0] } else { cache
            .group(2, 72, [0; 4])?
            .into_iter()
            .find(|(file, _)| *file as i32 == id)
            .map_or_else(|| vec![0], |(_, bytes)| bytes) };
        definitions.insert(id, Definition::decode(id, &bytes)?);
    }
    Ok(&definitions[&id])
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pose {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub angle: i32,
}
impl Pose {
    pub fn read_delta(&mut self, r: &mut Reader<'_>) -> Result<()> {
        let format = r.u8()?;
        let mut delta = [0; 4];
        for (axis, value) in delta.iter_mut().enumerate() {
            *value = match (format >> (axis * 2)) & 3 {
                0 => 0,
                1 => r.u8()? as i8 as i32,
                2 => r.u16()? as i16 as i32,
                _ => r.u32()? as i32,
            };
        }
        self.x = self.x.wrapping_add(delta[0]);
        self.y = self.y.wrapping_add(delta[1]);
        self.z = self.z.wrapping_add(delta[2]);
        self.angle = self.angle.wrapping_add(delta[3]) & 2047;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Step {
    pub pose: Pose,
    pub cycle: i32,
}

#[derive(Clone, Debug)]
pub struct Entity {
    pub animation_fraction: f32,
    pub world_spots: Vec<crate::world_spot::Effect>,
    pub zone: crate::zone::State,
    pub locations: crate::location_update::Queue,
    pub npcs: crate::npc_info::State,
    pub plane: u8,
    pub dimensions: [i32; 2],
    pub config: i32,
    pub owner_type: u8,
    pub animation: crate::actor_sequence::State,
    pub default_animation: crate::actor_sequence::State,
    pub definition: Option<Definition>,
    pub rebuild: Option<crate::rebuild::WorldEntity>,
    pub rebuild_generation: u64,
    /// Identity of this spawn, independent of subsequent terrain rebuilds.
    pub spawn_generation: u64,
    /// Global registry membership is independent of the parent entity node.
    pub registered: bool,
    pub pose: Pose,
    pub steps: [Step; 10],
    pub queued: usize,
    pub options: i32,
    pub interpolation_initialized: bool,
    pub previously_interpolated: bool,
    pub interpolation: Interpolation,
}
impl Default for Entity {
    fn default() -> Self {
        Self {
            animation_fraction: 0.0,
            world_spots: Vec::new(),
            zone: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            plane: 0,
            dimensions: [0; 2],
            config: -1,
            owner_type: 0,
            animation: Default::default(),
            default_animation: Default::default(),
            definition: None,
            rebuild: None,
            rebuild_generation: 0,
            spawn_generation: 0,
            registered: true,
            pose: Pose::default(),
            steps: [Step::default(); 10],
            queued: 0,
            options: 31,
            interpolation_initialized: false,
            previously_interpolated: false,
            interpolation: Default::default(),
        }
    }
}
impl Entity {
    /// GameClient world-view tick: action completion restarts the default clock,
    /// then the default clock advances in the same tick, even during an action.
    pub fn tick_animations(
        &mut self,
        sequences: &std::collections::HashMap<u32, crate::animation::Sequence>,
    ) -> Result<Vec<(u32, usize)>> {
        self.tick_animations_with_readiness(sequences,&mut |_|true)
    }
    pub fn scene_animation(&self,mut ready:impl FnMut(i32)->bool)->Option<&crate::actor_sequence::State> {
        let action=&self.animation;
        let selected=if action.id>=0&&action.delay==0&&ready(action.id){action}else{&self.default_animation};
        (selected.id>=0&&ready(selected.id)).then_some(selected)
    }
    pub fn tick_animations_with_readiness(
        &mut self,
        sequences:&std::collections::HashMap<u32,crate::animation::Sequence>,
        ready:&mut impl FnMut(i32)->bool,
    )->Result<Vec<(u32,usize)>> {
        fn restart(state: &mut crate::actor_sequence::State) {
            state.frame = 0;
            state.elapsed = 0;
            state.loops = 0;
        }
        fn advance(
            state: &mut crate::actor_sequence::State,
            sequences: &std::collections::HashMap<u32, crate::animation::Sequence>,
            events: &mut Vec<(u32, usize)>,
            ready: &mut impl FnMut(i32)->bool,
        ) -> Result<bool> {
            let id = state.id as u32;
            let sequence = sequences
                .get(&id)
                .ok_or(Error::Invalid("world entity animation"))?;
            let mut clock = state.clock();
            let (flags, frames) = clock.advance(sequence, 1)?;
            state.frame = clock.frame as i32;
            state.elapsed = clock.elapsed;
            state.loops = clock.loops;
            state.loading=if ready(state.id){0}else{state.loading.wrapping_add(1)};
            events.extend(frames.into_iter().map(|frame| (id, frame)));
            Ok(flags & 2 != 0)
        }
        let mut events = Vec::new();
        if self.animation.id >= 0 {
            if self.animation.delay > 0 {
                self.animation.delay -= 1;
            } else if advance(&mut self.animation, sequences, &mut events,ready)? {
                self.animation.reset();
                restart(&mut self.default_animation);
            }
        }
        if self.default_animation.id >= 0
            && advance(&mut self.default_animation, sequences, &mut events,ready)?
        {
            restart(&mut self.default_animation);
        }
        Ok(events)
    }
    pub fn shift(&mut self, delta: [i32; 2]) {
        let translate = |pose: &mut Pose| {
            pose.x = pose.x.wrapping_add(delta[0]);
            pose.z = pose.z.wrapping_add(delta[1]);
        };
        translate(&mut self.pose);
        for step in &mut self.steps {
            translate(&mut step.pose);
        }
        translate(&mut self.interpolation.from);
        translate(&mut self.interpolation.to);
    }
    pub fn geometry_transform(&self, elevation: i32) -> GeometryTransform {
        let offsets = self.definition.as_ref().map_or([0; 2], |d| d.scene_offset);
        let angle =
            ((self.pose.angle & 2047) as f32 / 2048.0) as f64 * (std::f64::consts::PI * 2.0);
        let half = angle as f32 * 0.5;
        let y = libm::sin(half as f64) as f32;
        let w = libm::cos(half as f64) as f32;
        let (ww, yy, wy) = (w * w, y * y, y * w);
        GeometryTransform {
            rotation_y: ww + yy - 0.0 - 0.0,
            rotation: [
                ww + 0.0 - 0.0 - yy,
                0.0 + 0.0 + wy + wy,
                0.0 - wy - wy + 0.0,
                ww + 0.0 - yy - 0.0,
            ],
            centre: [
                self.dimensions[0].wrapping_mul(64).wrapping_add(offsets[0]) as f32,
                self.dimensions[1].wrapping_mul(64).wrapping_add(offsets[1]) as f32,
            ],
            translation: [self.pose.x as f32, elevation as f32, self.pose.z as f32],
            vertical: [0.0, 1.0],
        }
    }
    pub fn transform_horizontal(&self, point: [i32; 2]) -> [f32; 3] {
        let offsets = self.definition.as_ref().map_or([0; 2], |d| d.scene_offset);
        let x = point[0].wrapping_sub(self.dimensions[0].wrapping_mul(64).wrapping_add(offsets[0]))
            as f32;
        let z = point[1].wrapping_sub(self.dimensions[1].wrapping_mul(64).wrapping_add(offsets[1]))
            as f32;
        let angle =
            ((self.pose.angle & 2047) as f32 / 2048.0) as f64 * (std::f64::consts::PI * 2.0);
        let half = angle as f32 * 0.5;
        let y = libm::sin(half as f64) as f32;
        let w = libm::cos(half as f64) as f32;
        // Match the quaternion-to-matrix arithmetic order in ud.ab.
        let ww = w * w;
        let yy = y * y;
        let wy = y * w;
        let xx = ww + 0.0 - 0.0 - yy;
        let xz = 0.0 + 0.0 + wy + wy;
        let zx = 0.0 - wy - wy + 0.0;
        let zz = ww + 0.0 - yy - 0.0;
        [
            self.pose.x as f32 + (xz * z + (0.0 + x * xx)),
            0.0,
            self.pose.z as f32 + (0.0 + x * zx + zz * z),
        ]
    }
    /// GameWorldEntity.qg: undo the current horizontal entity pose.
    pub fn inverse_horizontal(&self, point: [i32; 2]) -> [f32; 3] {
        let offsets = self.definition.as_ref().map_or([0; 2], |d| d.scene_offset);
        let angle = ((self.pose.angle & 2047) as f32 / 2048.0) as f64
            * (std::f64::consts::PI * 2.0);
        let half = angle as f32 * 0.5;
        let y = libm::sin(half as f64) as f32;
        let w = libm::cos(half as f64) as f32;
        let (ww, yy, wy) = (w * w, y * y, y * w);
        let xx = ww + 0.0 - 0.0 - yy;
        let zz = ww + 0.0 - yy - 0.0;
        let xz = 0.0 + 0.0 + wy + wy;
        let zx = 0.0 - wy - wy + 0.0;
        let vertical = ww + yy - 0.0 - 0.0;
        // Preserve the original matrix inverse's float evaluation order.
        // Its rounded quaternion matrix is not exactly orthonormal.
        let inverse = 1.0 / (zz * (xx * vertical) - vertical * zx * xz);
        let a = vertical * zz * inverse;
        let b = inverse * (-vertical * xz);
        let c = inverse * (-vertical * zx);
        let d = vertical * xx * inverse;
        let tx = self.pose.x as f32;
        let tz = self.pose.z as f32;
        let px = inverse * (xz * vertical * tz - zz * vertical * tx);
        let pz = inverse * (tz * (-xx * vertical) + tx * (vertical * zx));
        let pw = (vertical * xx * zz - vertical * zx * xz) * inverse;
        let scale = 1.0 / pw;
        let [x, z] = point.map(|v| v as f32);
        [
            (px + (b * z + (0.0 + x * a))) * scale
                + self.dimensions[0].wrapping_mul(64).wrapping_add(offsets[0]) as f32,
            0.0,
            (pz + (0.0 + x * c + d * z)) * scale
                + self.dimensions[1].wrapping_mul(64).wrapping_add(offsets[1]) as f32,
        ]
    }
    pub fn target(&self) -> Pose {
        if self.queued == 0 {
            self.pose
        } else {
            self.steps[0].pose
        }
    }
    pub fn snap(&mut self, pose: Pose) {
        self.pose = pose;
        self.steps[0].pose = pose;
        self.queued = 0;
        self.interpolation_initialized = false;
        self.previously_interpolated = false;
    }
    pub fn move_to(&mut self, pose: Pose, bounds: [i32; 2], cycle: i32) {
        let (x, z) = (pose.x >> 7, pose.z >> 7);
        if x < 0 || z < 0 || x >= bounds[0] || z >= bounds[1] {
            self.snap(pose);
        } else {
            self.queued = (self.queued + 1).min(9);
            self.steps[..=self.queued].rotate_right(1);
            self.steps[0] = Step { pose, cycle };
        }
    }
    pub fn operation_enabled(&self, operation: i32) -> bool {
        !(0..=4).contains(&operation) || self.options & (1 << operation) != 0
    }

    /// RuneLite's double-precision replacement of GameWorldEntity's movement
    /// update. Vertical position is retained during interpolation and copied
    /// only when the reference controller snaps.
    pub fn advance(&mut self, cycle: i32, fraction: f64, duration: i32) {
        self.animation_fraction = fraction as f32;
        if self.queued == 0 {
            self.snap(self.steps[0].pose);
            return;
        }
        if !self.interpolation_initialized {
            if self.previously_interpolated {
                self.interpolation
                    .sample(cycle.wrapping_sub(1) as f64, &mut self.pose);
            }
            self.interpolation = Interpolation {
                from: self.pose,
                to: self.steps[0].pose,
                start: cycle.wrapping_sub(1),
                end: self.steps[0].cycle.wrapping_add(duration),
            };
            self.interpolation_initialized = true;
        }
        if self
            .interpolation
            .sample(cycle as f64 + fraction, &mut self.pose)
        {
            self.queued -= 1;
            self.interpolation_initialized = false;
        }
        self.previously_interpolated = true;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GeometryTransform {
    pub rotation: [f32; 4],
    pub rotation_y: f32,
    pub centre: [f32; 2],
    pub translation: [f32; 3],
    pub vertical: [f32; 2],
}
impl GeometryTransform {
    pub fn matrix(&self, animation: Option<crate::skeletal::Matrix>) -> crate::skeletal::Matrix {
        let [xx, xz, zx, zz] = self.rotation;
        crate::matrix_pick::scene(
            self.centre,
            [xx, xz, zx, zz, self.rotation_y],
            self.translation,
            self.vertical,
            animation,
        )
    }
    /// GameScene.mx: collapse the scene before its animation and world transform.
    pub fn with_overlap_hidden(mut self, hidden: bool) -> Self {
        self.vertical = if hidden { [-1200.0, 0.01] } else { [0.0, 1.0] };
        self
    }
    pub fn animated_point(
        &self,
        point: [f32; 3],
        matrix: Option<&crate::skeletal::Matrix>,
    ) -> [f32; 3] {
        let Some(m) = matrix else {
            return self.point(point);
        };
        let [x, y, z] = [
            point[0] - self.centre[0],
            (point[1] + self.vertical[0]) * self.vertical[1],
            point[2] - self.centre[1],
        ];
        let px = m[0] * x + m[4] * y + m[8] * z + m[12];
        let py = m[1] * x + m[5] * y + m[9] * z + m[13];
        let pz = m[2] * x + m[6] * y + m[10] * z + m[14];
        let [xx, xz, zx, zz] = self.rotation;
        [
            self.translation[0] + (xz * pz + (0.0 + px * xx)),
            self.translation[1] + py,
            self.translation[2] + (0.0 + px * zx + zz * pz),
        ]
    }
    pub fn point(&self, point: [f32; 3]) -> [f32; 3] {
        let (x, z) = (point[0] - self.centre[0], point[2] - self.centre[1]);
        let [xx, xz, zx, zz] = self.rotation;
        [
            self.translation[0] + (xz * z + (0.0 + x * xx)),
            self.translation[1] + (point[1] + self.vertical[0]) * self.vertical[1],
            self.translation[2] + (0.0 + x * zx + zz * z),
        ]
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Interpolation {
    pub from: Pose,
    pub to: Pose,
    pub start: i32,
    pub end: i32,
}
impl Interpolation {
    pub fn sample(&self, cycle: f64, pose: &mut Pose) -> bool {
        if self.start >= self.end {
            *pose = self.to;
            return true;
        }
        let progress = (cycle - self.start as f64) / self.end.wrapping_sub(self.start) as f64;
        let amount = progress.clamp(0.0, 1.0);
        pose.x = self
            .from
            .x
            .wrapping_add((amount * self.to.x.wrapping_sub(self.from.x) as f64) as i32);
        pose.z = self
            .from
            .z
            .wrapping_add((amount * self.to.z.wrapping_sub(self.from.z) as f64) as i32);
        let mut angle = self.to.angle.wrapping_sub(self.from.angle) & 2047;
        if angle > 1024 {
            angle -= 2048;
        }
        pose.angle = self.from.angle.wrapping_add((amount * angle as f64) as i32) & 2047;
        progress >= 1.0
    }
}

#[derive(Clone, Debug, Default)]
pub struct State {
    pub visible: Vec<u16>,
    pub entities: std::collections::HashMap<u16, Entity>,
}
impl State {
    pub fn read_packet(
        &mut self,
        data: &[u8],
        bounds: [i32; 2],
        cycle: i32,
        sequences: &std::collections::HashMap<u32, crate::animation::Sequence>,
    ) -> Result<Vec<u16>> {
        Ok(self.read_packet_removed(data, bounds, cycle, sequences)?.into_iter().map(|(id, _)| id).collect())
    }
    /// Preserve removed objects for the independent global world registry.
    pub fn read_packet_removed(
        &mut self,
        data: &[u8],
        bounds: [i32; 2],
        cycle: i32,
        sequences: &std::collections::HashMap<u32, crate::animation::Sequence>,
    ) -> Result<Vec<(u16, Option<Entity>)>> {
        let mut r = Reader::new(data);
        let removed = self.read_existing_removed(&mut r, bounds, cycle, |_, entity, r, flags| {
            entity.read_mask(r, flags, sequences)
        })?;
        while r.remaining() != 0 {
            let id = r.u16()?;
            let flags = r.u8()?;
            let dimensions = r.transformed(crate::buffer::Transform::Neg)?;
            let owner_type = r.transformed(crate::buffer::Transform::Add)?;
            let config = r.u16_le()? as i16 as i32;
            let mut entity = Entity {
                dimensions: [(dimensions >> 4) as i32 * 8, (dimensions & 15) as i32 * 8],
                owner_type,
                config,
                ..Default::default()
            };
            entity.pose.read_delta(&mut r)?;
            entity.move_to(entity.pose, bounds, cycle);
            entity.read_mask(&mut r, flags, sequences)?;
            self.visible.push(id);
            self.entities.insert(id, entity);
        }
        Ok(removed)
    }
    /// Decode the retained-list portion. The caller handles animation masks and
    /// creates each trailing world view before inserting its entity.
    pub fn read_existing(
        &mut self,
        r: &mut Reader<'_>,
        bounds: [i32; 2],
        cycle: i32,
        mut mask: impl FnMut(u16, &mut Entity, &mut Reader<'_>, u8) -> Result<()>,
    ) -> Result<Vec<u16>> {
        Ok(self.read_existing_removed(r, bounds, cycle, &mut mask)?.into_iter().map(|(id, _)| id).collect())
    }
    fn read_existing_removed(
        &mut self,
        r: &mut Reader<'_>,
        bounds: [i32; 2],
        cycle: i32,
        mut mask: impl FnMut(u16, &mut Entity, &mut Reader<'_>, u8) -> Result<()>,
    ) -> Result<Vec<(u16, Option<Entity>)>> {
        let retained = r.u8()? as usize;
        if retained > self.visible.len() {
            return Err(Error::Invalid("world entity retained count"));
        }
        let old = std::mem::take(&mut self.visible);
        let mut removed: Vec<_> = old[retained..].iter().map(|&id| (id, self.entities.remove(&id))).collect();
        for id in old.into_iter().take(retained) {
            let mode = r.u8()?;
            if mode == 0 {
                removed.push((id, self.entities.remove(&id)));
                continue;
            }
            let entity = self
                .entities
                .get_mut(&id)
                .ok_or(Error::Invalid("missing retained world entity"))?;
            self.visible.push(id);
            if mode != 1 {
                let mut pose = entity.target();
                pose.read_delta(r)?;
                if mode == 2 {
                    entity.move_to(pose, bounds, cycle);
                } else {
                    entity.snap(pose);
                }
            }
            let flags = r.u8()?;
            mask(id, entity, r, flags)?;
        }
        Ok(removed)
    }
}

impl Entity {
    fn read_mask(
        &mut self,
        r: &mut Reader<'_>,
        flags: u8,
        sequences: &std::collections::HashMap<u32, crate::animation::Sequence>,
    ) -> Result<()> {
        if flags & 2 != 0 {
            self.options = r.transformed(crate::buffer::Transform::Sub)? as i32;
        }
        if flags & 1 != 0 {
            let id = r.u16()?;
            let delay = r.u8()?;
            self.animation.set(
                if id == 65535 { -1 } else { id as i32 },
                delay as i32,
                0,
                sequences,
            )?;
        }
        Ok(())
    }
}

/// Visible child views in parent-before-child order; IDs belong to one registry.
pub fn render_order(worlds: &std::collections::HashMap<u16, State>) -> Result<Vec<(u16, u16)>> {
    let mut order = Vec::new();
    let mut pending = worlds.get(&0).map_or_else(Vec::new, |world| {
        world.visible.iter().rev().map(|&id| (0, id)).collect()
    });
    let mut seen = std::collections::HashSet::from([0]);
    while let Some((parent, id)) = pending.pop() {
        if !seen.insert(id) {
            return Err(Error::Invalid("world entity hierarchy cycle or duplicate"));
        }
        if !worlds
            .get(&parent)
            .is_some_and(|world| world.entities.contains_key(&id))
        {
            return Err(Error::Invalid("visible world entity missing"));
        }
        order.push((parent, id));
        if let Some(world) = worlds.get(&id) {
            pending.extend(world.visible.iter().rev().map(|&child| (id, child)));
        }
    }
    Ok(order)
}
#[derive(Clone, Debug, Default)]
pub struct SceneTransform(pub Vec<(GeometryTransform, Option<crate::skeletal::Matrix>)>);
impl SceneTransform {
    pub fn matrix(&self) -> crate::skeletal::Matrix {
        self.0.iter().fold(
            crate::matrix_pick::identity(),
            |matrix, (transform, animation)| {
                crate::matrix_pick::then(matrix, transform.matrix(*animation))
            },
        )
    }
    pub fn point(&self, point: [f32; 3]) -> [f32; 3] {
        self.0.iter().fold(point, |point, (transform, animation)| {
            transform.animated_point(point, animation.as_ref())
        })
    }
}

/// GameClient.iw visits each registered parent's visible entity list.
pub fn simulation_order(
    registry: &crate::world_view::Registry,
    worlds: &std::collections::HashMap<u16, State>,
) -> Vec<(u16,u16)> {
    let mut order=Vec::new();
    for parent in registry.ordered() {
        if let Some(world)=worlds.get(&parent) {
            order.extend(world.visible.iter().filter(|id|world.entities.contains_key(id)).map(|&id|(parent,id)));
        }
    }
    order
}

/// Global cc.tt lookup returns the oldest still-linked matching world view.
/// Root-child selection uses the root entity table separately.
pub fn registered_entity<'a>(
    id: u16,
    worlds: &'a std::collections::HashMap<u16, State>,
    detached: &'a [State],
) -> Option<&'a Entity> {
    worlds.values().chain(detached.iter())
        .filter_map(|world| world.entities.get(&id))
        .filter(|entity| entity.registered)
        .min_by_key(|entity| entity.spawn_generation)
}

/// Mutable counterpart for global packet routing; parent-local routing stays separate.
pub fn registered_entity_mut<'a>(
    id: u16,
    worlds: &'a mut std::collections::HashMap<u16, State>,
    detached: &'a mut [State],
) -> Option<&'a mut Entity> {
    worlds.values_mut().chain(detached.iter_mut())
        .filter_map(|world| world.entities.get_mut(&id))
        .filter(|entity| entity.registered)
        .min_by_key(|entity| entity.spawn_generation)
}

/// Reconcile registered worlds by creation order without treating repeated IDs
/// in different parent tables as an entity hierarchy cycle.
pub fn registration_order(
    worlds: &std::collections::HashMap<u16, State>,
    detached: &[State],
) -> Vec<u16> {
    let mut entries: Vec<_> = worlds.values().chain(detached.iter())
        .flat_map(|world| world.entities.iter())
        .filter(|(_, entity)| entity.registered)
        .map(|(&id, entity)| (entity.spawn_generation, id)).collect();
    entries.sort_unstable();
    entries.into_iter().map(|(_, id)| id).collect()
}
