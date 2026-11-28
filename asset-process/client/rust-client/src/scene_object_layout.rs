//! Persistent ordinary-object identities used by native scene traversal.
use crate::{
    object::Definition,
    object_traversal,
    scene::{PlacedObject, Scene},
};
use std::collections::HashMap;
/// Stable caller identity for one original scene renderable invocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Call {
    Terrain(usize),
    Ordinary(usize),
    Boundary(usize),
    Decoration(usize),
    Ground(usize),
    Item(usize),
    Temporary(usize),
    ActorSpot(usize),
}
/// A removed placement may still be owned by tiles on another logical plane.
pub struct Retained {
    pub object: PlacedObject,
    pub traversal_id: usize,
    pub original_index: usize,
    pub animation: Option<[Option<crate::location_animation::Part>; 2]>,
    pub generation: u64,
}
pub struct Layout {
    pub item_layers: crate::item_scene::Layers,
    next_ground_identity: usize,
    pub ground_identities: Vec<Option<usize>>,
    next_boundary_identity: usize,
    next_decoration_identity: usize,
    pub decoration_identities: Vec<Option<[usize; 2]>>,
    pub boundary_identities: Vec<Option<[usize; 2]>>,
    free_identities: Vec<usize>,
    pub retained: Vec<Retained>,
    pub traversal: object_traversal::Scene,
    pub identities: Vec<Option<usize>>,
}
impl Layout {
    pub fn new(scene: &Scene, definitions: &HashMap<u32, Definition>) -> Self {
        let renderer=scene.definition_renderer();
        let mut result = Self {
            item_layers: crate::item_scene::Layers::default(),
            next_ground_identity: 0,
            ground_identities: vec![None; scene.objects.len()],
            next_boundary_identity: 0,
            next_decoration_identity: 0,
            decoration_identities: vec![None; scene.objects.len()],
            boundary_identities: vec![None; scene.objects.len()],
            free_identities: Vec::new(),
            retained: Vec::new(),
            traversal: object_traversal::Scene::empty([4, scene.width, scene.height]),
            identities: vec![None; scene.objects.len()],
        };
        // Native high-detail scene construction allocates plane zero first.
        for x in 0..scene.width {
            for z in 0..scene.height {
                result.traversal.allocate(0, x, z);
            }
        }
        for p in 0..4 {
            for x in 1..scene.width.saturating_sub(1) {
                for z in 1..scene.height.saturating_sub(1) {
                    let tile = scene.tile(p, x, z).unwrap();
                    if scene.loaded(p, x, z) && (tile.underlay != 0 || tile.overlay != 0) {
                        for below in 0..=p {
                            result.traversal.allocate(below, x, z);
                        }
                        let i = result.traversal.index(p, x, z);
                        result.traversal.tiles[i].visible_surface = true;
                    }
                }
            }
        }
        for (index, object) in scene.objects.iter().enumerate().filter(|(_, o)| !o.runtime) {
            if let Some(definition) = definitions.get(&object.id) {
                let definition=renderer.get(object.plane as usize,[object.x as i32,object.y as i32],definition);
                (result.identities[index], result.boundary_identities[index]) =
                    result.place(object, &definition);
                result.decoration_identities[index] = result.place_decoration(object);
                result.ground_identities[index] = result.place_ground(object);
            }
        }
        for p in 0..4 {
            for x in 0..scene.width {
                for z in 0..scene.height {
                    let i = result.traversal.index(p, x, z);
                    result.traversal.tiles[i].force_ground =
                        scene.tile(p, x, z).unwrap().flags & 8 != 0;
                }
            }
        }
        for x in 0..scene.width {
            for z in 0..scene.height {
                if scene.tile(1, x, z).unwrap().flags & 2 != 0 {
                    result.traversal.lower_bridge(x, z);
                }
            }
        }
        for (index, object) in scene.objects.iter().enumerate().filter(|(_, o)| o.runtime) {
            if let Some(definition) = definitions.get(&object.id) {
                let definition=renderer.get(object.plane as usize,[object.x as i32,object.y as i32],definition);
                (result.identities[index], result.boundary_identities[index]) =
                    result.place(object, &definition);
                result.decoration_identities[index] = result.place_decoration(object);
                result.ground_identities[index] = result.place_ground(object);
            }
        }
        result
    }
    fn place(
        &mut self,
        object: &PlacedObject,
        definition: &Definition,
    ) -> (Option<usize>, Option<[usize; 2]>) {
        if object.rejected {
            return (None, None);
        }
        let [x, z] = [object.x as usize, object.y as usize];
        let p = object.plane as usize;
        if p >= 4 || x >= self.traversal.dimensions[1] || z >= self.traversal.dimensions[2] {
            return (None, None);
        }
        let dimensions = definition.dimensions(object.rotation).map(i32::from);
        if (9..=21).contains(&object.shape) {
            let size = if matches!(object.shape, 10 | 11) {
                dimensions
            } else {
                [1; 2]
            };
            (
                self.insert_record(
                    p,
                    object_traversal::Object {
                        start: [x as i32, z as i32],
                        end: [x as i32 + size[0] - 1, z as i32 + size[1] - 1],
                        center: crate::location_draw_position::center(
                            object.shape,
                            [x as i32, z as i32],
                            definition.dimensions(object.rotation),
                        ),
                    },
                ),
                None,
            )
        } else if object.shape <= 3 {
            let orientation =
                (if matches!(object.shape, 1 | 3) { 16 } else { 1 }) << object.rotation;
            self.traversal.insert_wall(p, x, z, orientation);
            // Distinct part identities survive placement-index shifts and shared wall tags.
            let ids = [self.next_boundary_identity, self.next_boundary_identity + 1];
            self.next_boundary_identity += 2;
            let i = self.traversal.index(p, x, z);
            self.traversal.tiles[i].boundary = Some(object_traversal::Boundary {
                models: [Some(ids[0]), (object.shape == 2).then_some(ids[1])],
                secondary: if object.shape == 2 {
                    1 << ((object.rotation + 1) & 3)
                } else {
                    0
                },
                ..Default::default()
            });
            (None, Some(ids))
        } else {
            for below in (if object.shape == 22 { p } else { 0 })..=p {
                self.traversal.allocate(below, x, z);
            }
            (None, None)
        }
    }
    pub fn insert(&mut self, object: &PlacedObject, definition: &Definition) {
        let (id, boundary) = self.place(object, definition);
        self.identities.push(id);
        self.boundary_identities.push(boundary);
        let decoration = self.place_decoration(object);
        self.decoration_identities.push(decoration);
        let ground = self.place_ground(object);
        self.ground_identities.push(ground);
        self.prune_retained();
    }
    pub(crate) fn enable_boundary_secondary(&mut self, plane: usize, x: usize, z: usize) {
        let index = self.traversal.index(plane, x, z);
        if let Some(boundary) = &mut self.traversal.tiles[index].boundary {
            // Each placement reserves two consecutive identities, even before part two exists.
            if let Some(primary) = boundary.models[0] {
                boundary.models[1] = Some(primary + 1);
            }
        }
    }
    fn place_ground(&mut self, object: &PlacedObject) -> Option<usize> {
        if object.rejected || object.shape != 22 {
            return None;
        }
        let [p, x, z] = [object.plane as usize, object.x as usize, object.y as usize];
        if p >= 4 || x >= self.traversal.dimensions[1] || z >= self.traversal.dimensions[2] {
            return None;
        }
        let id = self.next_ground_identity;
        self.next_ground_identity += 1;
        let i = self.traversal.index(p, x, z);
        self.traversal.tiles[i].ground = Some(id);
        Some(id)
    }
    fn place_decoration(&mut self, object: &PlacedObject) -> Option<[usize; 2]> {
        if object.rejected || !(4..=8).contains(&object.shape) {
            return None;
        }
        let [p, x, z] = [object.plane as usize, object.x as usize, object.y as usize];
        if p >= 4 || x >= self.traversal.dimensions[1] || z >= self.traversal.dimensions[2] {
            return None;
        }
        let ids = [
            self.next_decoration_identity,
            self.next_decoration_identity + 1,
        ];
        self.next_decoration_identity += 2;
        let i = self.traversal.index(p, x, z);
        self.traversal.tiles[i].decoration = Some(object_traversal::Decoration {
            models: [Some(ids[0]), (object.shape == 8).then_some(ids[1])],
            mask: if object.shape <= 5 {
                1 << object.rotation
            } else {
                256
            },
            orientation: if object.shape == 7 {
                (object.rotation + 2) & 3
            } else {
                object.rotation
            },
            center: [x as i32 * 128 + 64, z as i32 * 128 + 64],
            ..Default::default()
        });
        Some(ids)
    }
    pub(crate) fn enable_decoration_secondary(&mut self, plane: usize, x: usize, z: usize) {
        let index = self.traversal.index(plane, x, z);
        if let Some(boundary) = &mut self.traversal.tiles[index].decoration {
            // Each placement reserves two consecutive identities, even before part two exists.
            if let Some(primary) = boundary.models[0] {
                boundary.models[1] = Some(primary + 1);
            }
        }
    }
    fn insert_record(&mut self, plane: usize, object: object_traversal::Object) -> Option<usize> {
        self.traversal.insert(plane, object).map(|new_id| {
            let Some(id) = self.free_identities.pop() else {
                return new_id;
            };
            // Insertion appends one record and references only its own plane.
            // Reuse only identities with no surviving scene references.
            assert_eq!(new_id + 1, self.traversal.objects.len());
            let record = self.traversal.objects.pop().unwrap();
            for x in record.start[0]..=record.end[0] {
                for z in record.start[1]..=record.end[1] {
                    let tile = self.traversal.index(plane, x as usize, z as usize);
                    for reference in &mut self.traversal.tiles[tile].objects {
                        if *reference == new_id {
                            *reference = id;
                        }
                    }
                }
            }
            self.traversal.objects[id] = record;
            id
        })
    }
    fn prune_retained(&mut self) {
        let traversal = &self.traversal;
        let free = &mut self.free_identities;
        self.retained.retain(|old| {
            if has_references(traversal, old.traversal_id) {
                true
            } else {
                free.push(old.traversal_id);
                false
            }
        });
    }
    pub(crate) fn insert_temporary(
        &mut self,
        plane: usize,
        object: object_traversal::Object,
    ) -> Option<usize> {
        let id = self.insert_record(plane, object);
        self.prune_retained();
        id
    }
    pub(crate) fn insert_actor_spot(&mut self, plane: usize, fine: [i32; 2]) -> Option<usize> {
        let tile = fine.map(|v| v >> 7);
        if plane >= self.traversal.dimensions[0]
            || (0..2).any(|i| tile[i] < 0 || tile[i] >= self.traversal.dimensions[i + 1] as i32)
        {
            return None;
        }
        self.traversal
            .allocate(plane, tile[0] as usize, tile[1] as usize);
        let object = object_traversal::Object {
            start: tile,
            end: tile,
            center: fine,
        };
        let id = if let Some(id) = self.free_identities.pop() {
            self.traversal.objects[id] = object;
            id
        } else {
            let id = self.traversal.objects.len();
            self.traversal.objects.push(object);
            id
        };
        self.prune_retained();
        Some(id)
    }
    pub(crate) fn remove_actor_spot(&mut self, id: usize) {
        assert!(!has_references(&self.traversal, id));
        self.free_identities.push(id);
    }
    pub(crate) fn remove_temporary(&mut self, plane: usize, id: usize) {
        self.traversal.remove(plane, id);
        assert!(!has_references(&self.traversal, id));
        self.free_identities.push(id);
    }
    pub fn remove(
        &mut self,
        index: usize,
        plane: usize,
        object: &PlacedObject,
        animation: Option<[Option<crate::location_animation::Part>; 2]>,
        generation: u64,
    ) {
        self.boundary_identities.remove(index);
        self.decoration_identities.remove(index);
        self.ground_identities.remove(index);
        if let Some(id) = self.identities.remove(index) {
            self.traversal.remove(plane, id);
            if has_references(&self.traversal, id) {
                self.retained.push(Retained {
                    object: object.clone(),
                    traversal_id: id,
                    original_index: index,
                    animation,
                    generation,
                });
            } else {
                self.free_identities.push(id);
            }
        } else if object.shape <= 3 {
            let i = self
                .traversal
                .index(plane, object.x as usize, object.y as usize);
            self.traversal.tiles[i].wall = 0;
            self.traversal.tiles[i].boundary = None;
        } else if (4..=8).contains(&object.shape) {
            let i = self
                .traversal
                .index(plane, object.x as usize, object.y as usize);
            self.traversal.tiles[i].decoration = None;
        } else if object.shape == 22 {
            let i = self
                .traversal
                .index(plane, object.x as usize, object.y as usize);
            self.traversal.tiles[i].ground = None;
        }
    }
}

fn has_references(scene: &object_traversal::Scene, id: usize) -> bool {
    let object = &scene.objects[id];
    (0..scene.dimensions[0]).any(|p| {
        (object.start[0]..=object.end[0]).any(|x| {
            (object.start[1]..=object.end[1]).any(|z| {
                let tile = &scene.tiles[scene.index(p, x as usize, z as usize)];
                let saved = p == 3 && scene.tiles[scene.index(0, x as usize, z as usize)].bridge;
                (tile.present || saved) && tile.objects.contains(&id)
            })
        })
    })
}
