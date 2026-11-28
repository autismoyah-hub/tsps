//! Terrain and object placement assembly for the primary 104-tile world view.
use crate::{
    collision::{CollisionMap, FLOOR, UNLOADED},
    disk_cache::DiskCache,
    rebuild::Normal,
    terrain::{self, Region, Tile},
    Error, Result,
};
use std::collections::{HashMap, HashSet};
pub const SIZE: usize = 104;
/// Own the optional scope so callers can mutate a scene while resolving definitions.
pub(crate) struct DefinitionRenderer {
    #[cfg(feature="custom-assets")] base:[i32;2],
    #[cfg(feature="custom-assets")] bindings:Option<std::sync::Arc<crate::environment_style::Bindings>>,
}
impl DefinitionRenderer {
    pub(crate) fn validate(&self,definitions:&HashMap<u32,crate::object::Definition>)->Result<()> {
        #[cfg(feature="custom-assets")]
        if let Some(bindings)=&self.bindings {return bindings.validate_definitions(definitions);}
        let _=definitions;Ok(())
    }
    pub(crate) fn get<'a>(&self,plane:usize,tile:[i32;2],definition:&'a crate::object::Definition)->std::borrow::Cow<'a,crate::object::Definition> {
        #[cfg(feature="custom-assets")]
        if plane==0 && (0..2).all(|i| {
            let position=i64::from(self.base[i])+i64::from(tile[i])-i64::from(crate::authored_floor::ORIGIN[i]);
            (0..128).contains(&position)
        }) {
            if let Some(bindings)=&self.bindings {return bindings.definition(definition);}
        }
        let _=(plane,tile);std::borrow::Cow::Borrowed(definition)
    }
}
#[derive(Clone)]
pub struct PlacedObject {
    /// Stable map-load identity for baked lighting across runtime removals.
    pub initial_index: Option<usize>,
    /// Null renderables and rejected insertions retain collision but have no scene entry.
    pub rejected: bool,
    /// Runtime packets address lowered scene planes; map records use physical planes.
    pub runtime: bool,
    pub id: u32,
    pub plane: u8,
    pub x: u8,
    pub y: u8,
    pub shape: u8,
    pub rotation: u8,
}
pub struct Scene {
    #[cfg(feature = "custom-assets")]
    authored_floor: Option<std::sync::Arc<crate::authored_floor::Floor>>,
    #[cfg(feature="custom-assets")]
    native_visuals: HashMap<u32,bool>,
    pub walk: crate::scene_walk::Selection,
    pub width: usize,
    pub height: usize,
    pub base_x: i32,
    pub base_y: i32,
    tiles: Vec<Tile>,
    heights: Vec<i32>,
    loaded: Vec<bool>,
    /// Ground decoration opcode-96 offsets, indexed by the rendered scene plane.
    ground_offsets: Vec<u8>,
    decoration_offsets: Vec<Option<[[i32; 2]; 2]>>,
    slots: Option<crate::scene_slots::Slots>,
    object_layout: Option<crate::scene_object_layout::Layout>,
    initial_objects: Option<Vec<PlacedObject>>,
    terrain_shadows: Vec<i8>,
    pub collision: Vec<CollisionMap>,
    pub objects: Vec<PlacedObject>,
    pub location_animations: HashMap<[i32; 4], [Option<crate::location_animation::Part>; 2]>,
    pub location_generations: HashMap<[i32; 4], u64>,
    animation_generation: u64,
    pub missing_regions: Vec<u32>,
}
impl Scene {
    pub(crate) fn definition_renderer(&self)->DefinitionRenderer {
        DefinitionRenderer {
            #[cfg(feature="custom-assets")] base:[self.base_x,self.base_y],
            #[cfg(feature="custom-assets")] bindings:self.authored_bindings(),
        }
    }
    #[cfg(feature = "custom-assets")]
    pub(crate) fn environment_motion(&self,triangles:&[crate::render3d::Triangle])->Result<crate::environment_motion::Motion> {
        let Some((mode,profile))=self.authored_floor.as_ref().and_then(|f|f.motion()) else{return Ok(crate::environment_motion::Motion::empty());};
        let allowed:HashSet<_>=self.objects.iter().filter(|o|!o.rejected&&o.plane==0&&o.shape==10&&o.rotation==0)
            .filter(|o|self.authored_floor.as_ref().unwrap().placements().iter().any(|p|
                p.id==o.id && p.shape==10 && p.rotation==0
                && crate::authored_floor::ORIGIN[0]+p.tile[0]as i32==self.base_x+o.x as i32
                && crate::authored_floor::ORIGIN[1]+p.tile[1]as i32==self.base_y+o.y as i32))
            .map(|o|(o.id,o.x as u16,o.y as u16)).collect();
        crate::environment_motion::Motion::bind(triangles,mode,profile,|t|
            allowed.contains(&((t.tag>>32).wrapping_sub(1)as u32,((t.tag>>14)&16383)as u16,(t.tag&16383)as u16)))
    }

    #[cfg(feature = "custom-assets")]
    pub(crate) fn authored_map_functions(&self)->Option<std::sync::Arc<crate::environment_minimap::MapFunctions>> {
        self.authored_floor.as_ref().and_then(|f|f.map_functions())
    }
    #[cfg(feature = "custom-assets")]
    pub(crate) fn minimap_geometry_id(&self,id:u32)->bool {self.native_visuals.get(&id)==Some(&true)||self.authored_floor.as_ref().is_some_and(|f|f.minimap_geometry_id(id))}
    /// A private map-only scene from the admitted placement stream. Never used for gameplay.
    #[cfg(feature = "custom-assets")]
    pub(crate) fn authored_minimap_scene(&self)->Result<Option<Self>> {
        if !self.native_visuals.is_empty(){
            let mut map=Self::empty_sized(self.base_x,self.base_y,self.width,self.height)?;
            map.tiles=self.tiles.clone();map.heights=self.heights.clone();map.loaded=self.loaded.clone();map.terrain_shadows=self.terrain_shadows.clone();map.native_visuals=self.native_visuals.clone();
            map.objects=self.objects.iter().filter(|o|!o.rejected&&self.minimap_geometry_id(o.id)).cloned().collect();return Ok(Some(map));
        }
        let Some(floor)=&self.authored_floor else{return Ok(None);};
        if floor.bindings().is_none(){return Ok(None);}
        let mut map=Self::empty_sized(self.base_x,self.base_y,self.width,self.height)?;map.apply_authored_floor(floor)?;
        for p in floor.placements() {
            if !floor.minimap_geometry_id(p.id){continue;}
            let x=crate::authored_floor::ORIGIN[0]+p.tile[0]as i32-map.base_x;
            let y=crate::authored_floor::ORIGIN[1]+p.tile[1]as i32-map.base_y;
            if x<0||y<0||x>=map.width as i32||y>=map.height as i32{continue;}
            map.objects.push(PlacedObject{initial_index:None,rejected:false,runtime:true,id:p.id,plane:0,x:x as u8,y:y as u8,shape:p.shape,rotation:p.rotation});
        }Ok(Some(map))
    }

    #[cfg(feature = "custom-assets")]
    pub(crate) fn environment_cutaway(&self)->Result<crate::environment_cutaway::Cutaway> {
        self.authored_floor.as_ref().map_or_else(||Ok(Default::default()),|f|f.cutaway([self.base_x,self.base_y]))
    }

    #[cfg(feature = "custom-assets")]
    pub(crate) fn authored_bindings(&self)->Option<std::sync::Arc<crate::environment_style::Bindings>> {
        self.authored_floor.as_ref().and_then(|f|f.bindings())
    }
    #[cfg(feature = "custom-assets")]
    pub(crate) fn visual_only_id(&self,id:u32)->bool {self.native_visuals.contains_key(&id)||self.authored_floor.as_ref().is_some_and(|f|f.visual_only_id(id))}
    /// Diagnostic omission may cover ordinary canonical scenery too. The
    /// visual-only flag governs legacy model admission, not floor membership.
    #[cfg(all(feature="custom-assets",feature="native-profiling"))]
    pub(crate) fn authored_diagnostic_placement(&self,object:&PlacedObject)->bool {
        let Some(floor)=&self.authored_floor else{return false;};
        !object.rejected && object.plane==0 && floor.placements().iter().any(|p|
            p.id==object.id && p.shape==object.shape && p.rotation==object.rotation
            && i64::from(crate::authored_floor::ORIGIN[0])+i64::from(p.tile[0])==i64::from(self.base_x)+i64::from(object.x)
            && i64::from(crate::authored_floor::ORIGIN[1])+i64::from(p.tile[1])==i64::from(self.base_y)+i64::from(object.y))
    }
    /// Only declared, noninteractive root art may bypass legacy model buckets.
    #[cfg(feature = "custom-assets")]
    pub(crate) fn authored_visual_placement(&self,object:&PlacedObject)->bool {
        if self.native_visuals.contains_key(&object.id){return !object.rejected&&!object.runtime&&object.shape==10&&object.rotation==0&&object.plane<=1;}
        let Some(floor)=&self.authored_floor else{return false;};
        !object.rejected && object.plane==0 && floor.visual_only_id(object.id)
            && floor.placements().iter().any(|p|p.id==object.id && p.shape==object.shape && p.rotation==object.rotation
                && i64::from(crate::authored_floor::ORIGIN[0])+i64::from(p.tile[0])==i64::from(self.base_x)+i64::from(object.x)
                && i64::from(crate::authored_floor::ORIGIN[1])+i64::from(p.tile[1])==i64::from(self.base_y)+i64::from(object.y))
    }
    #[cfg(feature = "custom-assets")]
    fn install_authored_visuals(&mut self,defs:&HashMap<u32,crate::object::Definition>)->Result<()> {
        let Some(floor)=self.authored_floor.clone() else {return Ok(());};
        if floor.bindings().is_none(){return Ok(());}
        for p in floor.placements() {
            if !floor.visual_only_id(p.id){continue;}
            let d=defs.get(&p.id).ok_or(Error::Invalid("authored visual definition absent"))?;
            if d.interact_type!=0||d.clipped||d.animation.is_some()||d.transforms.is_some(){return Err(Error::Invalid("authored regional visual is not static"));}
            let x=crate::authored_floor::ORIGIN[0]+p.tile[0]as i32-self.base_x;
            let y=crate::authored_floor::ORIGIN[1]+p.tile[1]as i32-self.base_y;
            if x<0||y<0||x>=self.width as i32||y>=self.height as i32{continue;}
            if !self.objects.iter().any(|old|old.id==p.id&&old.plane==0&&old.x==x as u8&&old.y==y as u8&&old.shape==p.shape) {
                self.objects.push(PlacedObject{initial_index:None,rejected:false,runtime:false,id:p.id,plane:0,x:x as u8,y:y as u8,shape:p.shape,rotation:p.rotation});
            }
        }
        Ok(())
    }

    #[cfg(feature = "custom-assets")]
    fn apply_authored_floor(&mut self, floor: &std::sync::Arc<crate::authored_floor::Floor>) -> Result<()> {
        let mut affected=false;
        for x in 0..=self.width {for z in 0..=self.height {
            let fx=self.base_x as i64+x as i64-crate::authored_floor::ORIGIN[0] as i64;
            let fz=self.base_y as i64+z as i64-crate::authored_floor::ORIGIN[1] as i64;
            if fx<0 || fz<0 {continue;}
            let Some(v)=floor.vertex(fx as usize,fz as usize) else {continue;};
            let h=-(v.height as i32);
            self.heights[x*(self.height+1)+z]=h;
            if x<self.width && z<self.height {
                affected=true;
                let i=x*self.height+z;
                self.tiles[i]=Tile{height:h,underlay:v.underlay,flags:if v.walkable{0}else{1},..Tile::default()};
                self.loaded[i]=true;
                self.collision[0].change(x as i32,z as i32,UNLOADED,false)?;
                self.terrain_shadows[x*(self.height+1)+z]=0;
            }
        }}
        if affected {self.authored_floor=Some(floor.clone());}
        Ok(())
    }
    /// Offline terrain inspection, without installing functional placements.
    #[cfg(feature = "custom-assets")]
    pub fn authored_floor_study(cache: &mut DiskCache) -> Result<Self> {
        Self::authored_floor_study_at(cache,crate::authored_floor::ORIGIN)
    }
    /// Offline terrain-only study at an explicit origin. Does not load chunks,
    /// install functional objects, or represent a child-world rebuild.
    #[cfg(feature = "custom-assets")]
    pub fn authored_floor_study_at(cache: &mut DiskCache, base:[i32;2]) -> Result<Self> {
        cache.group(2,1,[0;4])?; // Force refreshed namespace admission before geometry use.
        let floor=cache.authored_floor().ok_or(Error::Invalid("no authored floor mounted"))?;
        let mut scene=Self::empty(base[0],base[1])?;
        scene.apply_authored_floor(&floor)?;
        scene.add_floor_collision()?;
        Ok(scene)
    }
    /// Offline declared placements, never a server-driven world or child rebuild.
    #[cfg(feature="custom-assets")]
    pub fn authored_placement_study_at(cache:&mut DiskCache,base:[i32;2])->Result<Self> {
        let mut scene=Self::authored_floor_study_at(cache,base)?;
        let floor=cache.authored_floor().ok_or(Error::Invalid("no authored floor mounted"))?;
        for p in floor.placements() {
            let x=i64::from(crate::authored_floor::ORIGIN[0])+i64::from(p.tile[0])-i64::from(base[0]);
            let y=i64::from(crate::authored_floor::ORIGIN[1])+i64::from(p.tile[1])-i64::from(base[1]);
            if x<0||y<0||x>=scene.width as i64||y>=scene.height as i64 {continue;}
            scene.objects.push(PlacedObject{initial_index:Some(scene.objects.len()),rejected:false,runtime:false,id:p.id,plane:0,x:x as u8,y:y as u8,shape:p.shape,rotation:p.rotation});
        }
        let definitions=crate::object::Definition::load_all(cache)?;
        scene.add_object_collision(&definitions)?;
        Ok(scene)
    }
    pub fn empty(base_x: i32, base_y: i32) -> Result<Self> {
        Self::empty_sized(base_x, base_y, SIZE, SIZE)
    }
    pub fn empty_sized(base_x: i32, base_y: i32, width: usize, height: usize) -> Result<Self> {
        // World-entity packets encode each dimension in units of eight in four bits.
        if width > 120 || height > 120 {
            return Err(Error::Invalid("scene dimensions"));
        }
        Ok(Self {
            walk: Default::default(),
            #[cfg(feature = "custom-assets")]
            authored_floor: None,
            #[cfg(feature="custom-assets")]
            native_visuals: HashMap::new(),
            width,
            height,
            base_x,
            base_y,
            tiles: vec![Tile::default(); 4 * width * height],
            heights: vec![0; 4 * (width + 1) * (height + 1)],
            loaded: vec![false; 4 * width * height],
            ground_offsets: vec![0; 4 * width * height],
            decoration_offsets: Vec::new(),
            slots: None,
            object_layout: None,
            initial_objects: None,
            terrain_shadows: vec![0; (width + 1) * (height + 1)],
            collision: (0..4)
                .map(|_| CollisionMap::new(width, height, true))
                .collect::<Result<Vec<_>>>()?,
            objects: Vec::new(),
            location_animations: HashMap::new(),
            location_generations: HashMap::new(),
            animation_generation: 0,
            missing_regions: Vec::new(),
        })
    }
    /// Assemble instance terrain in destination-plane order, like GameNPCComposition.as.
    pub fn add_chunk(
        &mut self,
        region: &Region,
        chunk: crate::rebuild::Chunk,
        plane: usize,
        x: usize,
        y: usize,
    ) -> Result<()> {
        if plane >= 4
            || x + 8 > self.width
            || y + 8 > self.height
            || chunk.plane >= 4
            || chunk.rotation > 3
        {
            return Err(Error::Invalid("instance chunk bounds"));
        }
        let sx = usize::from(chunk.zone_x & 7) * 8;
        let sy = usize::from(chunk.zone_y & 7) * 8;
        for dx in 0..8 {
            for dy in 0..8 {
                let source = usize::from(chunk.plane) * 4096 + (sx + dx) * 64 + sy + dy;
                let (rx, ry) = crate::rebuild::rotate_tile(dx as u8, dy as u8, chunk.rotation)?;
                let (tx, ty) = (x + rx as usize, y + ry as usize);
                let below = if plane == 0 {
                    0
                } else {
                    self.height(plane - 1, tx, ty).unwrap()
                };
                let height = match region.height_rules[source] {
                    Some(delta) => below - i32::from(delta) * 8,
                    None if plane > 0 => below - 240,
                    None => terrain::procedural_height(
                        i32::from(chunk.zone_x) * 8 + dx as i32,
                        i32::from(chunk.zone_y) * 8 + dy as i32,
                    ),
                };
                let mut tile = region.tiles[source].clone();
                tile.height = height;
                tile.overlay_rotation = (tile.overlay_rotation + chunk.rotation) & 3;
                let i = (plane * self.width + tx) * self.height + ty;
                self.tiles[i] = tile;
                self.loaded[i] = true;
                self.heights[(plane * (self.width + 1) + tx) * (self.height + 1) + ty] = height;
                self.collision[plane].change(tx as i32, ty as i32, UNLOADED, false)?;
            }
        }
        Ok(())
    }
    pub fn load_instance(
        cache: &mut DiskCache,
        rebuild: &crate::rebuild::Instance,
    ) -> Result<Self> {
        let (bx, by) = rebuild.base();
        Self::load_chunks(
            cache,
            [bx, by],
            [SIZE, SIZE],
            &rebuild.regions,
            &rebuild.chunks,
            true,
        )
    }
    pub fn load_world_entity(
        cache: &mut DiskCache,
        rebuild: &crate::rebuild::WorldEntity,
    ) -> Result<Self> {
        Self::load_chunks(
            cache,
            rebuild.base,
            rebuild.dimensions,
            &rebuild.regions,
            &rebuild.chunks,
            false,
        )
    }
    fn load_chunks(
        cache: &mut DiskCache,
        base: [i32; 2],
        dimensions: [usize; 2],
        region_ids: &[u32],
        chunks: &[Option<crate::rebuild::Chunk>],
        _root: bool,
    ) -> Result<Self> {
        let [width, height] = dimensions;
        if width % 8 != 0 || height % 8 != 0 || chunks.len() != 4 * (width / 8) * (height / 8) {
            return Err(Error::Invalid("scene chunk dimensions"));
        }
        let mut scene = Self::empty_sized(base[0], base[1], width, height)?;
        let mut known = cache
            .reference(5)?
            .groups
            .iter()
            .map(|g| g.id)
            .collect::<HashSet<_>>();
        #[cfg(feature="custom-assets")]
        known.extend(cache.admitted_terrain_ids());
        let mut regions = std::collections::HashMap::new();
        for &id in region_ids {
            if !known.contains(&id) {
                scene.missing_regions.push(id);
                continue;
            }
            let files = cache.group(5, id, [0; 4])?;
            let terrain = files
                .iter()
                .find(|(n, _)| *n == 0)
                .ok_or(Error::Invalid("region terrain file"))?;
            let region = Region::decode(
                &terrain.1,
                ((id >> 8) * 64) as i32,
                ((id & 255) * 64) as i32,
            )?;
            let objects = match files.iter().find(|(n, _)| *n == 1) {
                Some((_, b)) => terrain::locations(b, 1_000_000)?,
                None => Vec::new(),
            };
            regions.insert(id, (region, objects));
        }
        let definitions = crate::object::Definition::load_all(cache)?;
        for plane in 0..4 {
            for x in 0..width / 8 {
                for y in 0..height / 8 {
                    let Some(chunk) = chunks[(plane * (width / 8) + x) * (height / 8) + y] else {
                        scene.clear_chunk(plane, x * 8, y * 8);
                        continue;
                    };
                    let Some((region, objects)) = regions.get(&chunk.region()) else {
                        scene.clear_chunk(plane, x * 8, y * 8);
                        continue;
                    };
                    scene.add_chunk(region, chunk, plane, x * 8, y * 8)?;
                    for obj in objects {
                        if obj.plane != chunk.plane
                            || obj.x / 8 != (chunk.zone_x & 7) as u8
                            || obj.y / 8 != (chunk.zone_y & 7) as u8
                        {
                            continue;
                        }
                        let def = definitions
                            .get(&obj.object)
                            .ok_or(Error::Invalid("instance object definition"))?;
                        let (rx, ry) = crate::rebuild::rotate_object(
                            obj.x & 7,
                            obj.y & 7,
                            chunk.rotation,
                            def.size,
                            obj.rotation,
                        )?;
                        let (tx, ty) = (x as i32 * 8 + rx, y as i32 * 8 + ry);
                        if tx > 0 && ty > 0 && tx < (width - 1) as i32 && ty < (height - 1) as i32 {
                            scene.objects.push(PlacedObject { initial_index: None, rejected: false,
                                runtime: false,
                                id: obj.object,
                                plane: plane as u8,
                                x: tx as u8,
                                y: ty as u8,
                                shape: obj.shape,
                                rotation: (obj.rotation + chunk.rotation) & 3,
                            });
                        }
                    }
                }
            }
        }
        // Only absent base-plane templates receive fa's dark edge fill; a
        // referenced but unavailable region follows the separate zs path above.
        for x in 0..width / 8 {
            for y in 0..height / 8 {
                if chunks[x * (height / 8) + y].is_none() {
                    scene.fill_void([x as i32 * 8, y as i32 * 8, 8, 8]);
                }
            }
        }
        #[cfg(feature = "custom-assets")]
        if _root {if let Some(floor)=cache.authored_floor() {scene.apply_authored_floor(&floor)?;}}
        #[cfg(feature = "custom-assets")]
        scene.definition_renderer().validate(&definitions)?;
        #[cfg(feature = "custom-assets")]
        scene.install_authored_visuals(&definitions)?;
        scene.add_floor_collision()?;
        scene.add_object_collision(&definitions)?;
        Ok(scene)
    }
    fn clear_chunk(&mut self, plane: usize, x: usize, y: usize) {
        let (width, height) = (self.width, self.height);
        let index = |x, y| (plane * (width + 1) + x) * (height + 1) + y;
        for dx in 0..8 {
            for dy in 0..8 {
                self.heights[index(x + dx, y + dy)] = 0;
            }
        }
        if x > 0 {
            for dy in 1..8 {
                self.heights[index(x, y + dy)] = self.heights[index(x - 1, y + dy)];
            }
        }
        if y > 0 {
            for dx in 1..8 {
                self.heights[index(x + dx, y)] = self.heights[index(x + dx, y - 1)];
            }
        }
        for (px, py) in [
            (x.checked_sub(1), Some(y)),
            (Some(x), y.checked_sub(1)),
            (x.checked_sub(1), y.checked_sub(1)),
        ] {
            if let (Some(px), Some(py)) = (px, py) {
                let h = self.heights[index(px, py)];
                if h != 0 {
                    self.heights[index(x, y)] = h;
                    break;
                }
            }
        }
    }
    pub fn load_normal(cache: &mut DiskCache, rebuild: &Normal) -> Result<Self> {
        let (base_x, base_y) = rebuild.base();
        let mut scene = Self::empty(base_x, base_y)?;
        #[cfg(feature="custom-assets")]
        if rebuild.regions().iter().any(|id|cache.admitted_terrain_ids().contains(id)){
            scene.native_visuals=cache.admitted_map_visuals()?.into_iter().collect();
        }
        let mut known = cache
            .reference(5)?
            .groups
            .iter()
            .map(|g| g.id)
            .collect::<HashSet<_>>();
        #[cfg(feature="custom-assets")]
        known.extend(cache.admitted_terrain_ids());
        for id in rebuild.regions() {
            if !known.contains(&id) {
                scene.missing_regions.push(id);
                continue;
            }
            let files = cache.group(5, id, [0; 4])?;
            let data = files
                .iter()
                .find(|(n, _)| *n == 0)
                .ok_or(Error::Invalid("region terrain file"))?;
            let rx = (id >> 8) as i32 * 64;
            let ry = (id & 255) as i32 * 64;
            let region = Region::decode(&data.1, rx, ry)?;
            scene.add_region(&region, rx - base_x, ry - base_y)?;
            if let Some((_, bytes)) = files.iter().find(|(n, _)| *n == 1) {
                for object in terrain::locations(bytes, 1_000_000)? {
                    let x = rx - base_x + object.x as i32;
                    let y = ry - base_y + object.y as i32;
                    if (0..SIZE as i32).contains(&x) && (0..SIZE as i32).contains(&y) {
                        scene.objects.push(PlacedObject { initial_index: None, rejected: false,
                            runtime: false,
                            id: object.object,
                            plane: object.plane,
                            x: x as u8,
                            y: y as u8,
                            shape: object.shape,
                            rotation: object.rotation,
                        });
                    }
                }
            }
        }
        if rebuild.zone_y < 800 {
            for id in scene.missing_regions.clone() {
                scene.fill_void([(id >> 8) as i32 * 64 - base_x, (id & 255) as i32 * 64 - base_y, 64, 64]);
            }
        }
        #[cfg(feature = "custom-assets")]
        if let Some(floor)=cache.authored_floor() {scene.apply_authored_floor(&floor)?;}
        scene.add_floor_collision()?;
        let definitions = crate::object::Definition::load_all(cache)?;
        #[cfg(feature = "custom-assets")]
        scene.definition_renderer().validate(&definitions)?;
        #[cfg(feature = "custom-assets")]
        scene.install_authored_visuals(&definitions)?;
        scene.add_object_collision(&definitions)?;
        Ok(scene)
    }
    fn add_floor_collision(&mut self) -> Result<()> {
        for plane in 0..4 {
            for x in 0..self.width {
                for y in 0..self.height {
                    if self.tile(plane, x, y).unwrap().flags & 1 != 0 {
                        let target = plane as i32
                            - if self.tile(1, x, y).unwrap().flags & 2 != 0 {
                                1
                            } else {
                                0
                            };
                        if target >= 0 {
                            self.collision[target as usize]
                                .change(x as i32, y as i32, FLOOR, true)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn fill_void(&mut self, rect: [i32; 4]) {
        crate::terrain_light::fill_void(&mut self.heights, &mut self.terrain_shadows, [self.width, self.height], rect);
    }
    pub(crate) fn terrain_shadows(&self) -> &[i8] { &self.terrain_shadows }
    /// Map-load shadows stay baked when runtime locations change.
    pub(crate) fn initial_objects(&self) -> &[PlacedObject] {
        self.initial_objects.as_deref().unwrap_or(&self.objects)
    }
    pub fn add_object_collision(
        &mut self,
        definitions: &std::collections::HashMap<u32, crate::object::Definition>,
    ) -> Result<()> {
        self.ensure_slots(definitions);
        if self.initial_objects.is_none() {
            let mut initial = Vec::new();
            for object in self.objects.iter_mut().filter(|o| !o.runtime) {
                object.initial_index = Some(initial.len());
                initial.push(object.clone());
            }
            self.initial_objects = Some(initial);
        }
        self.ground_offsets.fill(0);
        for obj in &self.objects {
            let d = definitions
                .get(&obj.id)
                .ok_or(Error::Invalid("placed object definition missing"))?;
            let Some(plane) = self.object_scene_plane(obj) else {
                continue;
            };
            let map = &mut self.collision[plane];
            let (x, y) = (obj.x as i32, obj.y as i32);
            if obj.shape == 22 {
                if !obj.rejected { self.ground_offsets[(plane as usize * self.width + obj.x as usize) * self.height
                    + obj.y as usize] = d.extra; }
                if d.interact_type == 1 {
                    map.change(x, y, crate::collision::FLOOR_DECORATION, true)?;
                }
            } else if obj.shape <= 3 && d.interact_type != 0 {
                map.wall(x, y, obj.shape, obj.rotation, d.blocks_projectiles, true)?;
            } else if obj.shape >= 9 && d.interact_type != 0 {
                let size = d.dimensions(obj.rotation);
                map.rectangle(
                    x,
                    y,
                    size[0] as u32,
                    size[1] as u32,
                    d.blocks_projectiles,
                    true,
                );
            }
        }
        Ok(())
    }
    fn add_region(&mut self, region: &Region, offset_x: i32, offset_y: i32) -> Result<()> {
        for plane in 0..4 {
            for x in 0..64 {
                for y in 0..64 {
                    let sx = offset_x + x as i32;
                    let sy = offset_y + y as i32;
                    if sx < 0 || sy < 0 || sx > self.width as i32 || sy > self.height as i32 {
                        continue;
                    }
                    let (sx, sy) = (sx as usize, sy as usize);
                    let tile = region.tile(plane, x, y).unwrap();
                    self.heights[(plane * (self.width + 1) + sx) * (self.height + 1) + sy] =
                        tile.height;
                    if sx < self.width && sy < self.height {
                        let i = (plane * self.width + sx) * self.height + sy;
                        self.tiles[i] = tile.clone();
                        self.loaded[i] = true;
                        self.collision[plane].change(sx as i32, sy as i32, UNLOADED, false)?;
                    }
                }
            }
        }
        Ok(())
    }
    pub fn tile(&self, plane: usize, x: usize, y: usize) -> Option<&Tile> {
        if plane >= 4 || x >= self.width || y >= self.height {
            None
        } else {
            Some(&self.tiles[(plane * self.width + x) * self.height + y])
        }
    }
    pub fn height(&self, plane: usize, x: usize, y: usize) -> Option<i32> {
        if plane >= 4 || x > self.width || y > self.height {
            None
        } else {
            Some(self.heights[(plane * (self.width + 1) + x) * (self.height + 1) + y])
        }
    }
    pub fn height_fine(&self, plane: usize, x: i32, y: i32) -> Option<i32> {
        if x < 0 || y < 0 {
            return None;
        }
        let (tx, ty) = ((x >> 7) as usize, (y >> 7) as usize);
        Some(crate::tile_mesh::bilinear_height(
            [
                self.height(plane, tx, ty)?,
                self.height(plane, tx + 1, ty)?,
                self.height(plane, tx + 1, ty + 1)?,
                self.height(plane, tx, ty + 1)?,
            ],
            [x, y],
        ))
    }
    pub fn loaded(&self, plane: usize, x: usize, y: usize) -> bool {
        plane < 4
            && x < self.width
            && y < self.height
            && self.loaded[(plane * self.width + x) * self.height + y]
    }
    /// Original ns.az / ds.ch minimum visible level for a physical terrain tile.
    pub fn tile_render_plane(&self, plane: usize, x: usize, y: usize) -> Option<usize> {
        if self.tile(plane, x, y)?.flags & 8 != 0 {
            Some(0)
        } else if plane > 0 && self.tile(1, x, y)?.flags & 2 != 0 {
            Some(plane - 1)
        } else {
            Some(plane)
        }
    }
    /// Rendered surface height (GameWorldView.jt), with bilinear fallback.
    pub fn elevation(&self, mut plane: usize, x: i32, y: i32) -> Option<i32> {
        if plane >= 4 || x < 0 || y < 0 {
            return None;
        }
        let (tx, ty) = ((x >> 7) as usize, (y >> 7) as usize);
        if tx >= self.width || ty >= self.height {
            return None;
        }
        let offset = i32::from(self.ground_offsets[(plane * self.width + tx) * self.height + ty]);
        let bridge = self.tile(1, tx, ty)?.flags & 2 != 0;
        let surface_present = !(bridge && plane == 3);
        if plane < 3 && bridge {
            plane += 1;
        }
        let corners = [
            self.height(plane, tx, ty)?,
            self.height(plane, tx + 1, ty)?,
            self.height(plane, tx + 1, ty + 1)?,
            self.height(plane, tx, ty + 1)?,
        ];
        let tile = self.tile(plane, tx, ty)?;
        // Scene construction excludes the outer border. Bridge lowering moves
        // the rendered surface down one plane, leaving plane three empty.
        if surface_present
            && tx > 0
            && ty > 0
            && tx + 1 < self.width
            && ty + 1 < self.height
            && self.loaded(plane, tx, ty)
            && (tile.underlay & 32767 != 0 || tile.overlay as u16 & 32767 != 0)
        {
            let shape = if tile.overlay as u16 & 32767 == 0 {
                0
            } else {
                tile.overlay_shape + 1
            };
            let point = [x & 127, y & 127];
            let surface = if shape <= 1 {
                crate::tile_mesh::paint_height(
                    corners,
                    shape == 1 && corners.iter().all(|&h| h == corners[0]),
                    point,
                )
            } else {
                crate::tile_mesh::surface_height(shape, tile.overlay_rotation, corners, point)
            };
            if surface.is_some() {
                return surface.map(|height| height.wrapping_sub(offset));
            }
        }
        Some(crate::tile_mesh::bilinear_height(corners, [x, y]).wrapping_sub(offset))
    }
}

impl Scene {
    pub fn animate_location(
        &mut self,
        key: [i32; 4],
        request: crate::location_animation::Request,
        defs: &HashMap<u32, crate::object::Definition>,
    ) -> Result<bool> {
        let object = self.location(key);
        if object.id < 0 {
            return Ok(false);
        }
        let def = defs
            .get(&(object.id as u32))
            .ok_or(Error::Invalid("location animation definition"))?;
        let def=self.definition_renderer().get(key[0] as usize,[key[1]-self.base_x,key[2]-self.base_y],def);
        let initial = crate::location_animation::parts(
            object.shape,
            object.rotation,
            def.animation.map(u32::from),
            0,
        )?;
        self.animation_generation = self.animation_generation.wrapping_add(1);
        let incoming = crate::location_animation::parts(
            request.shape,
            request.rotation,
            (request.sequence >= 0).then_some(request.sequence as u32),
            self.animation_generation,
        )?;
        let parts = self.location_animations.entry(key).or_insert(initial);
        for (old, new) in parts.iter_mut().zip(incoming) {
            if new.is_some() {
                *old = new;
            }
        }
        if key[3]==1 && incoming[1].is_some() {
            if let Some(layout)=&mut self.object_layout {
                layout.enable_decoration_secondary(key[0] as usize,(key[1]-self.base_x) as usize,(key[2]-self.base_y) as usize);
            }
        }
        // cj.dl can create a second wall renderable without changing its orientation mask.
        if key[3]==0 && incoming[1].is_some() {
            if let Some(layout)=&mut self.object_layout {
                layout.enable_boundary_secondary(key[0] as usize,(key[1]-self.base_x) as usize,(key[2]-self.base_y) as usize);
            }
        }
        Ok(true)
    }
    pub fn location(&self, key: [i32; 4]) -> crate::location_update::Object {
        let (x, y) = (key[1] - self.base_x, key[2] - self.base_y);
        self.objects
            .iter()
            .find(|o| {
                !o.rejected && self.object_scene_plane(o).map(|p| p as i32) == Some(key[0])
                    && o.x as i32 == x
                    && o.y as i32 == y
                    && crate::location_update::category(o.shape).ok() == Some(key[3] as u8)
            })
            .map_or(
                crate::location_update::Object {
                    id: -1,
                    shape: 0,
                    rotation: 0,
                },
                |o| crate::location_update::Object {
                    id: o.id as i32,
                    shape: o.shape,
                    rotation: o.rotation,
                },
            )
    }
    /// Runtime placements retain their packet plane but sample above bridges
    /// (yk.eh -> ny.ah). Map placements already carry their physical height plane.
    pub fn object_height_plane(&self, object: &PlacedObject) -> usize {
        let plane = object.plane as usize;
        if plane < 3
            && object.runtime
            && self
                .tile(1, object.x as usize, object.y as usize)
                .is_some_and(|tile| tile.flags & 2 != 0)
        {
            plane + 1
        } else {
            plane
        }
    }
    /// GameScene.at lowers static slots, retaining plane-zero objects below the bridge.
    pub fn object_scene_plane(&self, object: &PlacedObject) -> Option<usize> {
        let plane = object.plane as usize;
        if plane >= 4 {
            return None;
        }
        if !object.runtime
            && self
                .tile(1, object.x as usize, object.y as usize)
                .is_some_and(|tile| tile.flags & 2 != 0)
        {
            plane.checked_sub(1)
        } else {
            Some(plane)
        }
    }
    pub fn object_render_plane(&self, object: &PlacedObject) -> Option<usize> {
        if object.rejected { return None; }
        if object.runtime {
            self.slots.as_ref().map_or_else(
                || (object.plane < 4).then_some(object.plane as usize),
                |slots| slots.render_plane(object.plane as usize, object.x as usize, object.y as usize))
        } else {
            self.tile_render_plane(object.plane as usize, object.x as usize, object.y as usize)
        }
    }
    /// Runtime location replacement uses the packet plane's collision map (yk.eh).
    pub fn replace_location(
        &mut self,
        key: [i32; 4],
        object: &crate::location_update::Object,
        defs: &std::collections::HashMap<u32, crate::object::Definition>,
    ) -> Result<()> {
        let renderer=self.definition_renderer();
        let (x, y) = (key[1] - self.base_x, key[2] - self.base_y);
        if !(0..4).contains(&key[0])
            || !(1..self.width as i32 - 1).contains(&x)
            || !(1..self.height as i32 - 1).contains(&y)
        {
            return Ok(());
        }
        if object.id >= 0 && !defs.contains_key(&(object.id as u32)) {
            return Err(Error::Invalid("dynamic location definition"));
        }
        self.ensure_slots(defs);
        let previous_animation = self.location_animations.remove(&key);
        let previous_generation = self.location_generations.get(&key).copied().unwrap_or(0);
        self.animation_generation = self.animation_generation.wrapping_add(1);
        self.location_generations
            .insert(key, self.animation_generation);
        if let Some(index) = self.objects.iter().position(|o| {
            !o.rejected && self.object_scene_plane(o).map(|p| p as i32) == Some(key[0])
                && o.x as i32 == x
                && o.y as i32 == y
                && crate::location_update::category(o.shape).ok() == Some(key[3] as u8)
        }) {
            let old = self.objects.remove(index);
            self.decoration_offsets.remove(index);
            self.object_layout.as_mut().unwrap().remove(index, key[0] as usize, &old, previous_animation, previous_generation);
            self.location_collision(&old, defs, false)?;
            if (9..=21).contains(&old.shape) {
                if let Some((size, _)) = Self::slot_footprint(&old, &renderer.get(old.plane as usize,[old.x as i32,old.y as i32],&defs[&old.id])) {
                    // Removal touches only references still present on the addressed logical plane.
                    let layout = &self.object_layout.as_ref().unwrap().traversal;
                    for tx in x as usize..(x as usize+size[0]).min(self.width) {
                        for tz in y as usize..(y as usize+size[1]).min(self.height) {
                            let tile = &layout.tiles[layout.index(key[0] as usize,tx,tz)];
                            self.slots.as_mut().unwrap().set_object_count(key[0] as usize,tx,tz,tile.objects.len());
                        }
                    }
                }
            }
        }
        if object.id >= 0 {
            let mut placed = PlacedObject { initial_index: None, rejected: false,
                runtime: true,
                id: object.id as u32,
                plane: key[0] as u8,
                x: x as u8,
                y: y as u8,
                shape: object.shape,
                rotation: object.rotation,
            };
            let definition=renderer.get(placed.plane as usize,[placed.x as i32,placed.y as i32],&defs[&placed.id]);
            if let Some((size, below)) = Self::slot_footprint(&placed, &definition) {
                let slots = self.slots.as_mut().unwrap();
                if (9..=21).contains(&placed.shape) {
                    placed.rejected = !slots.insert_object(placed.plane as usize, placed.x as usize, placed.y as usize, size, true);
                } else { slots.insert(placed.plane as usize, placed.x as usize, placed.y as usize, size, below); }
            } else { placed.rejected = true; }
            self.location_collision(&placed, defs, true)?;
            if !placed.rejected {
                self.object_layout.as_mut().unwrap().insert(&placed, &definition);
                let offsets = if (4..=8).contains(&placed.shape) {
                    let width = |category| {
                        let entry = self.location([key[0], key[1], key[2], category]);
                        (entry.id >= 0).then(|| defs.get(&(entry.id as u32))).flatten().map(|d| d.decor_displacement)
                    };
                    Some(crate::decoration_offset::placement(placed.shape, placed.rotation, width(0), width(2)))
                } else { None };
                self.decoration_offsets.push(offsets);
                self.objects.push(placed);
            }
        }
        Ok(())
    }
    /// Compute both ordinary-object passes while preserving cross-frame pending tiles.
    pub fn draw_object_calls(&mut self, view: &crate::terrain_submission::View,
        visibility: &mut crate::scene_visibility::Visibility, fine: [i32; 2]) -> Option<[Vec<usize>; 2]> {
        self.draw_object_calls_with_temporary(view, visibility, fine, &[])
    }
    /// Temporary renderables constrain ordinary-object traversal until end-of-frame cleanup.
    pub fn draw_object_calls_with_temporary(&mut self, view: &crate::terrain_submission::View,
        visibility: &mut crate::scene_visibility::Visibility, fine: [i32; 2],
        temporary: &[(usize, crate::object_traversal::Object)]) -> Option<[Vec<usize>; 2]> {
        self.draw_render_calls(view,visibility,fine,temporary).map(|calls| calls.map(|pass|
            pass.into_iter().filter_map(|call|match call {
                crate::scene_object_layout::Call::Ordinary(id)=>Some(id),
                crate::scene_object_layout::Call::Temporary(_)|crate::scene_object_layout::Call::ActorSpot(_)|crate::scene_object_layout::Call::Boundary(_)|crate::scene_object_layout::Call::Decoration(_)|crate::scene_object_layout::Call::Ground(_)|crate::scene_object_layout::Call::Item(_)|crate::scene_object_layout::Call::Terrain(_)=>None,
            }).collect()))
    }
    /// Preserve temporary input identities through allocation, traversal and cleanup.
    /// Preserve item-layer occupancy independently of model/cache readiness.
    /// Replay item tile allocation independently of currently surviving piles.
    pub fn sync_item_allocations(&mut self, positions:impl IntoIterator<Item=[i32;3]>) {
        if self.object_layout.is_none() && self.objects.is_empty() {self.ensure_slots(&HashMap::new());}
        let Some(layout)=self.object_layout.as_mut() else {return;};
        for [plane,x,z] in positions {
            let Some(x)=x.checked_sub(self.base_x) else {continue;};
            let Some(z)=z.checked_sub(self.base_y) else {continue;};
            if !(0..4).contains(&plane)||x<0||z<0||x as usize>=self.width||z as usize>=self.height {continue;}
            layout.traversal.allocate(plane as usize,x as usize,z as usize);
            if let Some(slots)=self.slots.as_mut() {slots.insert(plane as usize,x as usize,z as usize,[1,1],false);}
        }
    }
    pub fn sync_item_piles(&mut self, piles: &[crate::item_scene::Pile]) {
        // Newly created child views may contain items before any location packet.
        if self.object_layout.is_none() && self.objects.is_empty() {self.ensure_slots(&HashMap::new());}
        let Some(layout)=self.object_layout.as_mut() else {return;};
        layout.item_layers.replace(&mut layout.traversal,piles);
        if let Some(slots)=self.slots.as_mut() {
            for pile in piles {
                if pile.plane<4 && pile.tile[0]<self.width && pile.tile[1]<self.height {
                    slots.insert(pile.plane,pile.tile[0],pile.tile[1],[1,1],false);
                }
            }
        }
    }
    pub fn sync_terrain_calls(&mut self, submissions:&[crate::terrain_scene::Submission]) {
        if self.object_layout.is_none() && self.objects.is_empty() {self.ensure_slots(&HashMap::new());}
        if let Some(layout)=self.object_layout.as_mut() {
            crate::terrain_scene::sync(&mut layout.traversal,submissions,self.walk.plane);
        }
    }
    pub fn item_identity(&self,plane:usize,tile:[usize;2],part:usize)->Option<usize> {
        crate::item_scene::identity(&self.object_layout.as_ref()?.traversal,plane,tile,part)
    }
    pub fn draw_render_calls(&mut self, view: &crate::terrain_submission::View,
        visibility: &mut crate::scene_visibility::Visibility, fine: [i32; 2],
        temporary: &[(usize, crate::object_traversal::Object)]) -> Option<[Vec<crate::scene_object_layout::Call>; 2]> {
        self.draw_render_calls_with_spots(view,visibility,fine,temporary,&[])
    }
    pub fn draw_render_calls_with_spots(&mut self,view:&crate::terrain_submission::View,
        visibility:&mut crate::scene_visibility::Visibility,fine:[i32;2],
        temporary:&[(usize,crate::object_traversal::Object)],spots:&[(usize,[i32;2])]) -> Option<[Vec<crate::scene_object_layout::Call>;2]> {
        let layout = self.object_layout.as_mut()?;
        let mut inserted = Vec::new();
        for (input_index,(plane, object)) in temporary.iter().enumerate() {
            if let Some(id) = layout.insert_temporary(*plane, object.clone()) {
                // Java retains newly allocated tiles after temporary cleanup.
                self.slots.as_mut().unwrap().insert(*plane, object.start[0] as usize, object.start[1] as usize,
                    [(object.end[0]-object.start[0]+1) as usize,(object.end[1]-object.start[1]+1) as usize], true);
                inserted.push((*plane,id,input_index));
            }
        }
        let mut spot_ids=HashMap::new();let mut spot_records=Vec::new();
        for (input_index,&(plane,position)) in spots.iter().enumerate() {
            if let Some(id)=layout.insert_actor_spot(plane,position) {
                self.slots.as_mut().unwrap().insert(plane,(position[0]>>7) as usize,(position[1]>>7) as usize,[1,1],false);
                spot_ids.insert(id,input_index);spot_records.push((plane,id));
            }
        }
        let heights = &self.heights;
        let width = self.width + 1;
        let height = self.height + 1;
        layout.traversal.prepare(view, visibility, fine, |p,x,z| heights[(p*width+x)*height+z]);
        let first = layout.traversal.first_render_calls_with_spots(&spot_records);
        let later = layout.traversal.render_calls_with_spots(&spot_records);
        let temporary_ids = inserted.iter().map(|&(_,id,input_index)|(id,input_index)).collect::<HashMap<_,_>>();
        let calls=[first,later].map(|pass|pass.into_iter().filter_map(|call|Some(match call {
            crate::object_traversal::RenderCall::Terrain(id) => crate::scene_object_layout::Call::Terrain(id),
            crate::object_traversal::RenderCall::Item(id)=>crate::scene_object_layout::Call::Item(id),
            crate::object_traversal::RenderCall::Ground(id)=>crate::scene_object_layout::Call::Ground(id),
            crate::object_traversal::RenderCall::Boundary(id)=>crate::scene_object_layout::Call::Boundary(id),
            crate::object_traversal::RenderCall::Decoration(id)=>crate::scene_object_layout::Call::Decoration(id),
            crate::object_traversal::RenderCall::Object(id)=>
                if let Some(&input_index)=spot_ids.get(&id) {crate::scene_object_layout::Call::ActorSpot(input_index)}
                else {temporary_ids.get(&id).map_or(crate::scene_object_layout::Call::Ordinary(id),
                    |&input_index|crate::scene_object_layout::Call::Temporary(input_index))}
        })).collect());
        for (plane,id,_) in inserted { layout.remove_temporary(plane,id); }
        for (_,id) in spot_records {layout.remove_actor_spot(id);}
        Some(calls)
    }
    /// Render-only retained placements do not re-enter collision or location lookup.
    pub(crate) fn retained_objects(&self) -> &[crate::scene_object_layout::Retained] {
        self.object_layout.as_ref().map_or(&[], |layout| &layout.retained)
    }
    /// GameScene.bo inspects only references in the actual present logical tile.
    pub(crate) fn object_support_tiles(&self, id: usize) -> Vec<[u8; 3]> {
        let Some(layout) = &self.object_layout else { return Vec::new(); };
        let traversal = &layout.traversal;
        let object = &traversal.objects[id];
        let mut tiles = Vec::new();
        for p in 0..4 {
            for x in object.start[0]..=object.end[0] {
                for z in object.start[1]..=object.end[1] {
                    let tile = &traversal.tiles[traversal.index(p, x as usize, z as usize)];
                    if tile.present && tile.objects.contains(&id) {
                        tiles.push([p as u8, x as u8, z as u8]);
                    }
                }
            }
        }
        tiles
    }
    pub fn ground_identity(&self,index:usize)->Option<usize> {
        self.object_layout.as_ref()?.ground_identities.get(index).copied().flatten()
    }
    pub fn decoration_part_identity(&self,index:usize,part:usize)->Option<usize> {
        self.object_layout.as_ref()?.decoration_identities.get(index).copied().flatten()?.get(part).copied()
    }
    pub fn boundary_part_identity(&self,index:usize,part:usize)->Option<usize> {
        self.object_layout.as_ref()?.boundary_identities.get(index).copied().flatten()?.get(part).copied()
    }
    pub fn object_traversal_id(&self, index: usize) -> Option<usize> {
        self.object_layout.as_ref()?.identities.get(index).copied().flatten()
    }
    /// Start a render frame with static occupancy; temporary actors never mutate it.
    pub fn temporary_slots(&self) -> crate::scene_slots::Slots {
        self.slots.clone().unwrap_or_else(|| crate::scene_slots::Slots::new(self.width, self.height))
    }

    fn slot_footprint(object: &PlacedObject, definition: &crate::object::Definition) -> Option<([usize; 2], bool)> {
        let shape = match object.shape { 4..=8 => 4, 11 => 10, shape => shape };
        if definition.animation.is_none() && definition.transforms.is_none() && definition.model_ids(shape).is_empty() { return None; }
        let size = if matches!(object.shape, 10 | 11) { definition.dimensions(object.rotation).map(usize::from) } else { [1, 1] };
        Some((size, object.shape != 22))
    }
    pub fn decoration_part_offsets(&self, index: usize) -> Option<[[i32; 2]; 2]> {
        self.decoration_offsets.get(index).copied().flatten()
    }
    fn ensure_slots(&mut self, definitions: &HashMap<u32, crate::object::Definition>) {
        if self.slots.is_some() { return; }
        let renderer=self.definition_renderer();
        let mut slots = crate::scene_slots::Slots::new(self.width, self.height);
        // The native high-detail scene initializes plane zero, including empty terrain.
        slots.insert(0, 0, 0, [self.width, self.height], false);
        for plane in 0..4 {
            for x in 1..self.width.saturating_sub(1) { for y in 1..self.height.saturating_sub(1) {
                let tile = self.tile(plane, x, y).unwrap();
                if self.loaded(plane, x, y) && (tile.underlay != 0 || tile.overlay != 0) {
                    slots.insert(plane, x, y, [1, 1], true);
                }
            }}
        }
        self.decoration_offsets = vec![None; self.objects.len()];
        let mut decoration_state = crate::decoration_offset::Construction::default();
        // Walls and decorations have one entry per physical slot; game objects keep a list.
        let mut singleton_slots = HashMap::new();
        let mut overwritten = Vec::new();
        for (index, object) in self.objects.iter_mut().enumerate().filter(|(_, o)| !o.runtime) {
            if let Some((size, below)) = definitions.get(&object.id).and_then(|d| Self::slot_footprint(object, &renderer.get(object.plane as usize,[object.x as i32,object.y as i32],d))) {
                if (9..=21).contains(&object.shape) {
                    object.rejected = !slots.insert_object(object.plane as usize, object.x as usize, object.y as usize, size, true);
                } else { slots.insert(object.plane as usize, object.x as usize, object.y as usize, size, below); }
            } else { object.rejected = true; }
            if let Some(definition) = definitions.get(&object.id) {
                decoration_state.place(object, definition.decor_displacement, index, &mut self.decoration_offsets);
            }
            let category = crate::location_update::category(object.shape).ok();
            if !object.rejected && category.is_some_and(|kind| kind != 2) {
                let key = (object.plane, object.x, object.y, category.unwrap());
                if let Some(previous) = singleton_slots.insert(key, index) { overwritten.push(previous); }
            }
        }
        for index in overwritten { self.objects[index].rejected = true; }
        for plane in 0..4 { for x in 0..self.width { for y in 0..self.height {
            slots.mark_force_ground(plane, x, y, self.tile(plane, x, y).unwrap().flags & 8 != 0);
        }}}
        for x in 0..self.width { for y in 0..self.height {
            if self.tile(1, x, y).unwrap().flags & 2 != 0 {
                slots.lower_bridge(x, y);
                decoration_state.lower_bridge(x, y);
            }
        }}
        for (index, object) in self.objects.iter_mut().enumerate().filter(|(_, o)| o.runtime) {
            if let Some((size, below)) = definitions.get(&object.id).and_then(|d| Self::slot_footprint(object, &renderer.get(object.plane as usize,[object.x as i32,object.y as i32],d))) {
                if (9..=21).contains(&object.shape) {
                    object.rejected = !slots.insert_object(object.plane as usize, object.x as usize, object.y as usize, size, true);
                } else { slots.insert(object.plane as usize, object.x as usize, object.y as usize, size, below); }
            } else { object.rejected = true; }
            if let Some(definition) = definitions.get(&object.id) {
                decoration_state.place(object, definition.decor_displacement, index, &mut self.decoration_offsets);
            }
        }
        self.slots = Some(slots);
        self.object_layout = Some(crate::scene_object_layout::Layout::new(self, definitions));
    }

    fn location_collision(
        &mut self,
        o: &PlacedObject,
        defs: &std::collections::HashMap<u32, crate::object::Definition>,
        add: bool,
    ) -> Result<()> {
        let d = defs
            .get(&o.id)
            .ok_or(Error::Invalid("dynamic location collision definition"))?;
        let Some(plane) = self.object_scene_plane(o) else {
            return Ok(());
        };
        let map = &mut self.collision[plane];
        let (x, y) = (o.x as i32, o.y as i32);
        if o.shape == 22 {
            if !add || !o.rejected {
                self.ground_offsets[(plane * self.width + o.x as usize) * self.height + o.y as usize] = if add { d.extra } else { 0 };
            }
            if d.interact_type == 1 {
                map.change(x, y, crate::collision::FLOOR_DECORATION, add)?;
            }
        } else if o.shape <= 3 && d.interact_type != 0 {
            map.wall(x, y, o.shape, o.rotation, d.blocks_projectiles, add)?;
        } else if o.shape >= 9 && d.interact_type != 0 {
            let [w, h] = d.dimensions(o.rotation);
            map.rectangle(x, y, w as u32, h as u32, d.blocks_projectiles, add);
        }
        Ok(())
    }
}

#[cfg(test)]
mod elevation_tests {
    use super::*;
    use crate::buffer::Reader;

    #[test]
    fn tile_render_levels_match_original_java() {
        let mut r = Reader::new(include_bytes!(
            "../tests/fixtures/tile-render-plane-jvm.bin"
        ));
        let mut scene = Scene::empty_sized(0, 0, 16, 16).unwrap();
        for case in 0..r.u32().unwrap() {
            let plane = r.u8().unwrap() as usize;
            let flags = r.u8().unwrap();
            let bridge_flags = r.u8().unwrap();
            let expected = r.u8().unwrap() as usize;
            scene.tiles[(16 + 4) * 16 + 4].flags = bridge_flags;
            scene.tiles[(plane * 16 + 4) * 16 + 4].flags = flags;
            assert_eq!(
                scene.tile_render_plane(plane, 4, 4),
                Some(expected),
                "case {case}"
            );
            let object = PlacedObject { initial_index: None, rejected: false,
                runtime: false,
                id: 1,
                plane: plane as u8,
                x: 4,
                y: 4,
                shape: 10,
                rotation: 0,
            };
            assert_eq!(scene.object_render_plane(&object), Some(expected));
        }
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn item_support_slots_match_original_bridge_scene() {
        let mut r = Reader::new(include_bytes!("../tests/fixtures/bridge-support-jvm.bin"));
        for case in 0..r.u32().unwrap() {
            let bridge = r.u8().unwrap() != 0;
            let mut scene = Scene::empty_sized(0, 0, 16, 16).unwrap();
            scene.tiles[(16 + 4) * 16 + 4].flags = if bridge { 2 } else { 0 };
            let mut heights = [0; 4];
            for plane in 0..4 {
                for _ in 0..r.u8().unwrap() {
                    let supports = r.u8().unwrap() != 0;
                    let is_model = r.u8().unwrap() != 0;
                    let vertices = (0..r.u8().unwrap())
                        .map(|_| std::array::from_fn(|_| f32::from_bits(r.u32().unwrap())))
                        .collect::<Vec<_>>();
                    let object = PlacedObject { initial_index: None, rejected: false,
                        runtime: false,
                        id: 1,
                        plane,
                        x: 4,
                        y: 4,
                        shape: 10,
                        rotation: 0,
                    };
                    if let Some(slot) = scene
                        .object_scene_plane(&object)
                        .filter(|_| supports && is_model)
                    {
                        heights[slot] =
                            heights[slot].max(crate::ground_model::support_height(&vertices));
                    }
                }
            }
            let expected = std::array::from_fn(|_| r.u32().unwrap() as i32);
            assert_eq!(heights, expected, "case {case}, bridge {bridge}");
        }
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn bridge_location_lookup_and_removal_match_original_scene() {
        let mut r = Reader::new(include_bytes!("../tests/fixtures/bridge-location-jvm.bin"));
        for case in 0..r.u32().unwrap() {
            let bridge = r.u8().unwrap() != 0;
            let mask = r.u8().unwrap();
            let target = r.u8().unwrap();
            let mut scene = Scene::empty_sized(0, 0, 16, 16).unwrap();
            scene.tiles[(16 + 4) * 16 + 7].flags = if bridge { 2 } else { 0 };
            let mut defs = HashMap::new();
            for plane in 0..4u8 {
                let id = 1000 + case * 4 + plane as u32;
                defs.insert(
                    id,
                    crate::object::Definition::decode_local(id, &[1, 1, 0, 1, 22, 27, 96, plane + 1, 0]).unwrap(),
                );
                if mask & (1 << plane) != 0 {
                    scene.objects.push(PlacedObject { initial_index: None, rejected: false,
                        runtime: false,
                        id,
                        plane,
                        x: 4,
                        y: 7,
                        shape: 22,
                        rotation: plane,
                    });
                }
            }
            scene.add_object_collision(&defs).unwrap();
            for phase in 0..2 {
                for plane in 0..4 {
                    let tag = r.u64().unwrap();
                    let height = r.u32().unwrap() as i32;
                    let actual = scene.location([plane, 4, 7, 3]);
                    assert_eq!(
                        actual.id,
                        if tag == 0 { -1 } else { (tag >> 20) as i32 },
                        "case {case}, phase {phase}, plane {plane}"
                    );
                    if tag != 0 {
                        assert_eq!(actual.rotation, ((tag >> 14) & 3) as u8);
                    }
                    assert_eq!(scene.elevation(plane as usize, 576, 960), Some(height));
                    assert_eq!(
                        scene.collision[plane as usize].flag_at(4, 7).unwrap()
                            & crate::collision::FLOOR_DECORATION
                            != 0,
                        tag != 0
                    );
                }
                if phase == 0 {
                    scene
                        .replace_location(
                            [target as i32, 4, 7, 3],
                            &crate::location_update::Object {
                                id: -1,
                                shape: 22,
                                rotation: 0,
                            },
                            &defs,
                        )
                        .unwrap();
                }
            }
            // Replacing a lowered slot must coexist with the next static slot,
            // even if both records carry the same raw plane or object ID.
            let id = 1000 + case * 4 + target as u32;
            let replacement = crate::location_update::Object {
                id: id as i32,
                shape: 22,
                rotation: 2,
            };
            let before = (0..4)
                .map(|p| scene.location([p, 4, 7, 3]).id)
                .collect::<Vec<_>>();
            scene
                .replace_location([target as i32, 4, 7, 3], &replacement, &defs)
                .unwrap();
            assert_eq!(scene.location([target as i32, 4, 7, 3]).id, id as i32);
            for p in 0..4 {
                if p != target as usize {
                    assert_eq!(scene.location([p as i32, 4, 7, 3]).id, before[p]);
                }
            }
        }
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn runtime_location_height_planes_match_original_placements() {
        let mut r = Reader::new(include_bytes!("../tests/fixtures/location-height-jvm.bin"));
        for case in 0..r.u32().unwrap() {
            let plane = r.u8().unwrap();
            let bridge = r.u8().unwrap() != 0;
            let x = r.u8().unwrap();
            let z = r.u8().unwrap();
            let sx = r.u8().unwrap();
            let sz = r.u8().unwrap();
            let rotation = r.u8().unwrap();
            let expected_plane = r.u32().unwrap();
            let expected_height = r.u32().unwrap() as i32;
            let expected_tag = r.u64().unwrap();
            let mut scene = Scene::empty_sized(0, 0, 16, 16).unwrap();
            for p in 0..4 {
                for xx in 0..17 {
                    for zz in 0..17 {
                        scene.heights[(p * 17 + xx) * 17 + zz] =
                            p as i32 * 1000 - xx as i32 * 17 + zz as i32 * 9;
                    }
                }
            }
            scene.tiles[(16 + x as usize) * 16 + z as usize].flags = if bridge { 2 } else { 0 };
            let id = 1000 + case;
            let definition =
                crate::object::Definition::decode_local(id, &[1, 1, 0, 1, 22, 17, 14, sx, 15, sz, 0]).unwrap();
            let size = definition.dimensions(rotation);
            let defs = [(id, definition)].into_iter().collect();
            scene
                .replace_location(
                    [plane as i32, x as i32, z as i32, 3],
                    &crate::location_update::Object {
                        id: id as i32,
                        shape: 22,
                        rotation,
                    },
                    &defs,
                )
                .unwrap();
            let object = &scene.objects[0];
            assert_eq!(object.plane as u32, expected_plane);
            assert_eq!((expected_tag >> 14) & 3, expected_plane as u64);
            let hp = scene.object_height_plane(object);
            let axes = [
                (x as usize, size[0] as usize),
                (z as usize, size[1] as usize),
            ]
            .map(|(v, size)| {
                if v + size <= 16 {
                    [v + size / 2, v + (size + 1) / 2]
                } else {
                    [v, v + 1]
                }
            });
            let [[x1, x2], [z1, z2]] = axes;
            let height = (scene.height(hp, x1, z1).unwrap()
                + scene.height(hp, x2, z1).unwrap()
                + scene.height(hp, x1, z2).unwrap()
                + scene.height(hp, x2, z2).unwrap())
                >> 2;
            assert_eq!(
                height, expected_height,
                "case {case}, plane {plane}, bridge {bridge}"
            );
            // Static map objects already carry their physical plane.
            scene.objects[0].runtime = false;
            assert_eq!(scene.object_height_plane(&scene.objects[0]), plane as usize);
        }
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn scene_surfaces_and_bridges_match_original_java() {
        check_surfaces(
            include_bytes!("../tests/fixtures/scene-elevation-jvm.bin"),
            false,
        );
    }

    #[test]
    fn ground_decoration_offsets_match_original_java() {
        check_surfaces(
            include_bytes!("../tests/fixtures/scene-decoration-jvm.bin"),
            true,
        );
    }

    fn check_surfaces(data: &[u8], decorations: bool) {
        let mut r = Reader::new(data);
        for case in 0..r.u32().unwrap() {
            let shape = r.u8().unwrap();
            let rotation = r.u8().unwrap();
            let bridge = r.u8().unwrap() != 0;
            let plane = r.u8().unwrap() as usize;
            let mut scene = Scene::empty_sized(0, 0, 16, 16).unwrap();
            let mut defs = HashMap::new();
            for p in 0..4 {
                for (x, y) in [(4, 7), (5, 7), (5, 8), (4, 8)] {
                    scene.heights[(p * 17 + x) * 17 + y] = r.u32().unwrap() as i32;
                }
                if decorations {
                    let offset = r.u8().unwrap();
                    defs.insert(
                        p as u32,
                        crate::object::Definition::decode_local(p as u32, &[1, 1, 0, 1, 22, 96, offset, 0])
                            .unwrap(),
                    );
                    scene.objects.push(PlacedObject { initial_index: None, rejected: false,
                        runtime: false,
                        id: p as u32,
                        plane: p as u8,
                        x: 4,
                        y: 7,
                        shape: 22,
                        rotation: 0,
                    });
                }
                let i = (p * 16 + 4) * 16 + 7;
                scene.loaded[i] = true;
                let s = (shape + p as u8) % 14;
                if s == 0 {
                    scene.tiles[i].underlay = 1;
                } else if s < 13 {
                    scene.tiles[i].overlay = 1;
                    scene.tiles[i].overlay_shape = s - 1;
                    scene.tiles[i].overlay_rotation = rotation;
                }
            }
            if bridge {
                scene.tiles[(16 + 4) * 16 + 7].flags = 2;
            }
            if decorations {
                scene.add_object_collision(&defs).unwrap();
            }
            let x = r.u32().unwrap() as i32;
            let y = r.u32().unwrap() as i32;
            let expected = r.u32().unwrap() as i32;
            assert_eq!(
                scene.elevation(plane, 512 + x, 896 + y),
                Some(expected),
                "case {case}, shape {shape}, rotation {rotation}, bridge {bridge}, plane {plane}"
            );
        }
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn dynamic_ground_offsets_replace_remove_and_do_not_leak_to_other_layers() {
        use crate::location_update::Object;
        let mut scene = Scene::empty_sized(3200, 3200, 16, 16).unwrap();
        let defs = [
            (
                1,
                crate::object::Definition::decode_local(1, &[1, 1, 0, 1, 22, 96, 17, 0]).unwrap(),
            ),
            (
                2,
                crate::object::Definition::decode_local(2, &[1, 1, 0, 1, 22, 96, 255, 0]).unwrap(),
            ),
        ]
        .into_iter()
        .collect();
        let key = [0, 3204, 3207, 3];
        let mut object = Object {
            id: 1,
            shape: 22,
            rotation: 0,
        };
        scene.replace_location(key, &object, &defs).unwrap();
        assert_eq!(scene.elevation(0, 576, 960), Some(-17));
        assert_eq!(scene.elevation(1, 576, 960), Some(0));
        assert_eq!(scene.height_fine(0, 576, 960), Some(0));
        object.id = 2;
        scene.replace_location(key, &object, &defs).unwrap();
        assert_eq!(scene.elevation(0, 576, 960), Some(-255));
        object.id = -1;
        scene.replace_location(key, &object, &defs).unwrap();
        assert_eq!(scene.elevation(0, 576, 960), Some(0));
        object.id = 2;
        object.shape = 10;
        scene
            .replace_location([0, 3204, 3207, 2], &object, &defs)
            .unwrap();
        assert_eq!(scene.elevation(0, 576, 960), Some(0));
    }
}
#[cfg(test)]
mod slot_tests {
    use super::*;
    fn definition() -> crate::object::Definition {
        let mut d = crate::object::Definition::decode(1, &[0], false, false).unwrap();
        d.models = vec![1];
        d.model_shapes = Some(vec![22]);
        d
    }
    fn replace(scene: &mut Scene, plane: i32, id: i32, shape: u8, definitions: &HashMap<u32, crate::object::Definition>) {
        scene.replace_location([plane, 4, 4, crate::location_update::category(shape).unwrap() as i32], &crate::location_update::Object { id, shape, rotation: 0 }, definitions).unwrap();
    }
    #[test]
    fn replacement_preserves_static_slot_force_ground() {
        for occupied in [false, true] {
            for bridge in [false, true] {
                let mut scene = Scene::empty_sized(0, 0, 16, 16).unwrap();
                let physical = if bridge { 3 } else { 2 };
                scene.tiles[(physical * 16 + 4) * 16 + 4].flags = 8;
                if bridge { scene.tiles[(16 + 4) * 16 + 4].flags = 2; }
                let definitions = HashMap::from([(1, definition())]);
                if occupied { scene.objects.push(PlacedObject { initial_index: None, rejected: false, runtime: false, id: 1, plane: physical as u8, x: 4, y: 4, shape: 22, rotation: 0 }); }
                scene.add_object_collision(&definitions).unwrap();
                // Removal preserves the slot even though no placement remains.
                replace(&mut scene, 2, -1, 22, &definitions);
                replace(&mut scene, 2, 1, 22, &definitions);
                assert_eq!(scene.object_render_plane(&scene.objects[0]), Some(if occupied { 0 } else { 2 }));
                replace(&mut scene, 2, -1, 22, &definitions);
                replace(&mut scene, 2, 1, 22, &definitions);
                assert_eq!(scene.object_render_plane(&scene.objects[0]), Some(if occupied { 0 } else { 2 }));
            }
        }
    }
    #[test]
    fn upper_terrain_allocates_lower_slots_but_missing_static_model_does_not() {
        for floor in [false, true] {
            let mut scene = Scene::empty_sized(0, 0, 16, 16).unwrap();
            scene.tiles[(2 * 16 + 4) * 16 + 4].flags = 8;
            if floor { scene.tiles[(3 * 16 + 4) * 16 + 4].underlay = 1; scene.loaded[(3 * 16 + 4) * 16 + 4] = true; }
            let mut missing = definition(); missing.id = 2; missing.models.clear(); missing.model_shapes = None;
            let definitions = HashMap::from([(1, definition()), (2, missing)]);
            scene.objects.push(PlacedObject { initial_index: None, rejected: false, runtime: false, id: 2, plane: 2, x: 4, y: 4, shape: 22, rotation: 0 });
            replace(&mut scene, 2, 1, 22, &definitions);
            assert!(scene.objects[0].rejected);
            assert_eq!(scene.object_render_plane(scene.objects.last().unwrap()), Some(if floor { 0 } else { 2 }));
        }
    }
}

#[cfg(test)]
mod null_location_tests {
 use super::*;
 use crate::{buffer::Reader,location_update::Object};
 #[test]
 fn original_null_location_lifecycle() {
  let mut r=Reader::new(include_bytes!("../tests/fixtures/null-location-jvm.bin"));
  for case in 0..r.u32().unwrap() {
   let plane=r.u8().unwrap();let category=r.u8().unwrap();let x=r.u8().unwrap();let y=r.u8().unwrap();let first=r.u8().unwrap()!=0;let second=r.u8().unwrap()!=0;
   let shape=[0,4,10,22][category as usize];let mut present=crate::object::Definition::decode(1,&[0],false,false).unwrap();present.extra=23;present.models=vec![1];present.model_shapes=Some(vec![shape]);
   let missing=crate::object::Definition::decode(2,&[96,23,0],false,false).unwrap();let defs=HashMap::from([(1,present),(2,missing)]);
   let mut scene=Scene::empty_sized(0,0,16,16).unwrap();let key=[plane as i32,x as i32,y as i32,category as i32];
   scene.objects.push(PlacedObject{initial_index: None, rejected:false,runtime:false,id:if first {1} else {2},plane,x,y,shape,rotation:0});scene.add_object_collision(&defs).unwrap();
   for phase in 0..2 {
    if phase==1 {scene.replace_location(key,&Object{id:if second {1} else {2},shape,rotation:0},&defs).unwrap();}
    let exists=r.u8().unwrap()!=0;let occupied=r.u8().unwrap()!=0;let height=r.u32().unwrap() as i32;
    assert_eq!(scene.elevation(plane as usize,x as i32*128+64,y as i32*128+64),Some(height),"height {case} phase {phase}");
    assert_eq!(scene.location(key).id>=0,exists,"case {case} phase {phase} kind {category}");
    assert_eq!(scene.slots.as_ref().unwrap().occupied(plane as usize,x as usize,y as usize),occupied || plane==0,"slot {case} phase {phase}");
    assert_eq!(scene.objects.iter().any(|o|scene.object_render_plane(o).is_some()),exists,"render {case} phase {phase}");
   }
  }
  assert_eq!(r.remaining(),0);
 }
 #[test]
 fn dynamic_model_less_definition_still_has_scene_entry() {
  for animated in [false,true] {
   let mut def=crate::object::Definition::decode(1,&[0],false,false).unwrap();
   if animated {def.animation=Some(1);} else {def.transforms=Some(vec![None]);}
   let defs=HashMap::from([(1,def)]);let mut scene=Scene::empty_sized(0,0,16,16).unwrap();
   scene.replace_location([2,4,4,2],&Object{id:1,shape:10,rotation:0},&defs).unwrap();
   assert_eq!(scene.location([2,4,4,2]).id,1);assert_eq!(scene.object_render_plane(&scene.objects[0]),Some(2));
  }
 }
}

#[cfg(test)]
mod shadow_tests {
    use super::*;
    #[test]
    fn identical_static_objects_keep_distinct_baked_identities() {
        let mut scene = Scene::empty_sized(0, 0, 16, 16).unwrap();
        let mut definition = crate::object::Definition::decode(1, &[0], false, false).unwrap();
        definition.models = vec![1];
        definition.model_shapes = Some(vec![10]);
        let definitions = HashMap::from([(1, definition)]);
        for _ in 0..2 {
            scene.objects.push(PlacedObject { initial_index: None, rejected: false, runtime: false, id: 1, plane: 0, x: 4, y: 4, shape: 10, rotation: 0 });
        }
        scene.add_object_collision(&definitions).unwrap();
        assert_eq!(scene.objects[0].initial_index, Some(0));
        assert_eq!(scene.objects[1].initial_index, Some(1));
        scene.replace_location([0, 4, 4, 2], &crate::location_update::Object { id: -1, shape: 10, rotation: 0 }, &definitions).unwrap();
        assert_eq!(scene.objects.len(), 1);
        assert_eq!(scene.objects[0].initial_index, Some(1));
        assert_eq!(scene.initial_objects().len(), 2);
    }
    #[test]
    fn baked_casters_survive_runtime_removal_and_collision_rebuild() {
        let mut scene = Scene::empty_sized(0, 0, 16, 16).unwrap();
        let mut def = crate::object::Definition::decode(1, &[0], false, false).unwrap();
        def.models = vec![1];
        def.model_shapes = Some(vec![0]);
        let defs = HashMap::from([(1, def)]);
        scene.objects.push(PlacedObject { initial_index: None, rejected: false, runtime: false, id: 1, plane: 0, x: 4, y: 4, shape: 0, rotation: 0 });
        scene.add_object_collision(&defs).unwrap();
        scene.replace_location([0, 4, 4, 0], &crate::location_update::Object { id: -1, shape: 0, rotation: 0 }, &defs).unwrap();
        assert!(scene.objects.is_empty());
        scene.add_object_collision(&defs).unwrap();
        assert_eq!(scene.initial_objects().len(), 1);
        assert_eq!(scene.initial_objects()[0].rotation, 0);
        scene.replace_location([0, 4, 4, 0], &crate::location_update::Object { id: 1, shape: 0, rotation: 2 }, &defs).unwrap();
        scene.add_object_collision(&defs).unwrap();
        assert_eq!(scene.initial_objects().len(), 1);
        assert_eq!(scene.initial_objects()[0].rotation, 0);
    }
}

#[cfg(test)]
#[path = "../tests/support/native_object_layout.rs"]
mod native_object_layout_tests;

#[cfg(test)]
#[path = "../tests/support/native_object_churn.rs"]
mod native_object_churn_tests;

#[cfg(test)]
#[path = "../tests/support/native_temporary_objects.rs"]
mod native_temporary_object_tests;

#[cfg(all(test,feature="custom-assets",feature="native-profiling"))]
mod diagnostic_placement_tests {
    use super::*;
    use sha2::{Digest,Sha256};
    #[test]
    fn canonical_placement_without_visual_only_flag_is_admitted_only_at_exact_root_site() {
        let mut text=String::from("BROKEN_WATER_RETURN 1\n");
        for x in 0..128 {for y in 0..128 {text.push_str(&format!("V {x} {y} 0 5802 1\n"));}}
        text.push_str("O 65100 40 40 10 2\n");
        let floor=crate::authored_floor::Floor::decode(text.as_bytes(),Sha256::digest(text.as_bytes()).into()).unwrap();
        let mut scene=Scene::empty_sized(crate::authored_floor::ORIGIN[0]+8,crate::authored_floor::ORIGIN[1]+16,64,64).unwrap();
        let object=PlacedObject{initial_index:None,rejected:false,runtime:true,id:65100,plane:0,x:32,y:24,shape:10,rotation:2};
        assert!(!scene.authored_diagnostic_placement(&object));
        scene.authored_floor=Some(std::sync::Arc::new(floor));
        assert!(!scene.authored_visual_placement(&object));
        assert!(scene.authored_diagnostic_placement(&object));
        macro_rules! reject {($field:ident,$value:expr)=>{{let mut other=object.clone();other.$field=$value;assert!(!scene.authored_diagnostic_placement(&other),stringify!($field));}}}
        reject!(id,65101);reject!(plane,1);reject!(x,33);reject!(y,25);reject!(shape,22);reject!(rotation,3);reject!(rejected,true);
        let mut static_object=object.clone();static_object.runtime=false;
        assert!(scene.authored_diagnostic_placement(&static_object));
    }
}
