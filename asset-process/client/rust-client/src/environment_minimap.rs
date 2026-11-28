//! Authored mesh geography composed into the ordinary minimap.
use crate::{scene::Scene, Error, Result};
pub fn paint_minimap(
    scene: &Scene,
    mesh: &crate::world_mesh::WorldMesh,
    image: &mut crate::raster::Raster,
) -> Result<usize> {
    if !scene.objects.iter().any(|o|scene.minimap_geometry_id(o.id)) {
        return Ok(0);
    }
    let palette = crate::floor::palette(0.8)?;
    let w = image.width();
    let h = image.height();
    let mut height = vec![f32::INFINITY; w * h];
    let mut painted = 0;
    for t in &mesh.triangles {
        if t.view != 0
            || t.alpha >= 128
            || !scene.minimap_geometry_id((t.tag >> 32).wrapping_sub(1) as u32)
        {
            continue;
        }
        let p = t
            .vertices
            .map(|v| [v[0] / 32., h as f32 - v[2] / 32., v[1]]);
        let edge = |a: [f32; 3], b: [f32; 3], x: f32, y: f32| {
            (b[0] - a[0]) * (y - a[1]) - (b[1] - a[1]) * (x - a[0])
        };
        let area = edge(p[0], p[1], p[2][0], p[2][1]);
        if area.abs() < 0.0001 {
            continue;
        }
        let lo = [
            p.iter()
                .map(|v| v[0])
                .fold(f32::INFINITY, f32::min)
                .floor()
                .max(0.) as usize,
            p.iter()
                .map(|v| v[1])
                .fold(f32::INFINITY, f32::min)
                .floor()
                .max(0.) as usize,
        ];
        let hi = [
            p.iter()
                .map(|v| v[0])
                .fold(f32::NEG_INFINITY, f32::max)
                .ceil()
                .max(0.)
                .min(w as f32) as usize,
            p.iter()
                .map(|v| v[1])
                .fold(f32::NEG_INFINITY, f32::max)
                .ceil()
                .max(0.)
                .min(h as f32) as usize,
        ];
        let color = palette[t.colors[0].clamp(0, 65535) as usize];
        for y in lo[1]..hi[1] {
            for x in lo[0]..hi[0] {
                let a = edge(p[1], p[2], x as f32 + 0.5, y as f32 + 0.5) / area;
                let b = edge(p[2], p[0], x as f32 + 0.5, y as f32 + 0.5) / area;
                let c = 1. - a - b;
                if a < -0.0001 || b < -0.0001 || c < -0.0001 {
                    continue;
                }
                let up = a * p[0][2] + b * p[1][2] + c * p[2][2];
                let index = y * w + x;
                if up <= height[index] {
                    height[index] = up;
                    image.fill_rect(x as i32, y as i32, 1, 1, color, 256);
                    painted += 1;
                }
            }
        }
    }
    Ok(painted)
}

pub fn scene_geography(
    scene: &Scene,
    cache: &mut crate::disk_cache::DiskCache,
) -> Result<Option<Vec<u32>>> {
    let Some(source) = scene.authored_minimap_scene()? else {
        return Ok(None);
    };
    let mesh = crate::world_mesh::WorldMesh::build(&source, cache, 3)?;
    let mut raster = crate::raster::Raster::new(scene.width * 4, scene.height * 4, 16 << 20)?;
    paint_minimap(&source, &mesh, &mut raster)?;
    Ok(Some(raster.pixels().to_vec()))
}
pub fn compose_geography(image: &mut crate::raster::Raster, source: Option<&[u32]>) -> Result<()> {
    if let Some(source) = source {
        if source.len() != image.pixels().len() {
            return Err(Error::Invalid("authored map geography size"));
        }
        for (dst, &pixel) in image.pixels_mut().iter_mut().zip(source) {
            if pixel != 0 {
                *dst = pixel;
            }
        }
    }
    Ok(())
}

/// Original map-element identities; positions always come from scene objects.
pub struct MapFunctions {
    identity: [u8; 32],
    package: [u8; 32],
    bindings: std::collections::BTreeMap<u32, u32>,
}
impl MapFunctions {
    pub fn read(
        path: &std::path::Path,
        expected: [u8; 32],
        package: &crate::asset_package::Package,
    ) -> Result<Self> {
        use std::io::Read;
        let info = std::fs::symlink_metadata(path)?;
        if !info.is_file() || info.file_type().is_symlink() || info.len() > 4096 {
            return Err(Error::Invalid("map function bounds/type"));
        }
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(4097)
            .read_to_end(&mut bytes)?;
        Self::decode(&bytes, expected, package)
    }
    pub fn decode(
        bytes: &[u8],
        expected: [u8; 32],
        package: &crate::asset_package::Package,
    ) -> Result<Self> {
        use sha2::{Digest, Sha256};
        if bytes.len() > 4096 || <[u8; 32]>::from(Sha256::digest(bytes)) != expected {
            return Err(Error::Invalid("map function admission"));
        }
        let text = std::str::from_utf8(bytes).map_err(|_| Error::Invalid("map function UTF8"))?;
        let mut lines = text.lines();
        if lines.next() != Some("CINDERWORKS_MAP_FUNCTIONS 1") {
            return Err(Error::Invalid("map function header"));
        }
        let mut bindings = std::collections::BTreeMap::new();
        for line in lines.filter(|l| !l.is_empty() && !l.starts_with('#')) {
            let v = line
                .split_whitespace()
                .map(str::parse::<u32>)
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|_| Error::Invalid("map function number"))?;
            if v.len() != 2
                || !matches!(
                    v[0],
                    18491 | 409 | 37947 | 18525 | 18526 | 2097 | 9736 | 57692 | 10943 | 11360
                )
                || v[1] > 65535
                || bindings.insert(v[0], v[1]).is_some()
            {
                return Err(Error::Invalid("map function binding"));
            }
        }
        Ok(Self {
            identity: expected,
            package: package.identity(),
            bindings,
        })
    }
    pub fn bindings(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.bindings
            .iter()
            .map(|(&object, &element)| (object, element))
    }
    pub fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub fn package_identity(&self) -> [u8; 32] {
        self.package
    }
    pub fn validate(&self, cache: &mut crate::disk_cache::DiskCache) -> Result<()> {
        let defs = crate::object::Definition::load_all(cache)?;
        let elements = crate::map_element::Definition::load_all(cache)?;
        for (&object, &id) in &self.bindings {
            let element = elements
                .get(&id)
                .ok_or(Error::Invalid("map element absent"))?;
            if !defs.contains_key(&object) || !element.minimap || element.sprites[0].is_none() {
                return Err(Error::Invalid("map function original definition"));
            }
        }
        Ok(())
    }
}
fn map_icon_candidates(scene: &Scene, functions: &MapFunctions) -> Vec<([i32; 2], u32)> {
    scene
        .objects
        .iter()
        .filter_map(|obj| {
            if obj.rejected || obj.plane != 0 {
                return None;
            }
            let position = [
                scene.base_x + i32::from(obj.x),
                scene.base_y + i32::from(obj.y),
            ];
            if !(0..2).all(|axis| {
                (0..128).contains(
                    &(i64::from(position[axis]) - i64::from(crate::authored_floor::ORIGIN[axis])),
                )
            }) {
                return None;
            }
            functions.bindings.get(&obj.id).map(|&id| (position, id))
        })
        .collect()
}
pub fn append_map_icons(
    scene: &Scene,
    cache: &mut crate::disk_cache::DiskCache,
    plane: usize,
    out: &mut Vec<([i32; 2], std::sync::Arc<crate::sprite_pixels::Pixels>)>,
) -> Result<()> {
    if plane != 0 {
        return Ok(());
    }
    let Some(functions) = scene.authored_map_functions() else {
        return Ok(());
    };
    let candidates = map_icon_candidates(scene, &functions);
    if candidates.is_empty() {
        out.sort_by_key(|(p, _)| *p);
        return Ok(());
    }
    let elements = crate::map_element::Definition::load_all(cache)?;
    let mut sprites = std::collections::HashMap::new();
    for (position, id) in candidates {
        let element = elements
            .get(&id)
            .ok_or(Error::Invalid("map element absent"))?;
        if !element.minimap {
            return Err(Error::Invalid("map element not enabled"));
        }
        let sprite = element.sprites[0].ok_or(Error::Invalid("map element sprite absent"))?;
        if let std::collections::hash_map::Entry::Vacant(entry) = sprites.entry(sprite) {
            entry.insert(std::sync::Arc::new(crate::sprite_pixels::Pixels::load(
                cache, sprite,
            )?));
        }
        if !out.iter().any(|(p, _)| *p == position) {
            out.push((position, sprites[&sprite].clone()));
        }
    }
    out.sort_by_key(|(p, _)| *p);
    Ok(())
}

#[cfg(test)]
mod footprint_tests {
    use super::*;
    #[test]
    fn icon_candidates_preserve_order_and_exclude_both_footprint_edges_and_upper_planes() {
        let functions = MapFunctions {
            identity: [0; 32],
            package: [0; 32],
            bindings: std::collections::BTreeMap::from([(409, 26), (18491, 5)]),
        };
        let origin = crate::authored_floor::ORIGIN;
        for (offset, inside, outside) in [(-4, 8, 2), (120, 2, 8)] {
            let mut scene =
                Scene::empty_sized(origin[0] + offset, origin[1] + offset, 16, 16).unwrap();
            for (id, plane, x, y, rejected) in [
                (409, 0, inside, inside, false),
                (409, 0, outside, inside, false),
                (409, 0, inside, outside, false),
                (409, 1, inside, inside, false),
                (409, 0, inside + 2, inside, true),
                (999, 0, inside, inside + 2, false),
                (18491, 0, inside + 1, inside + 1, false),
            ] {
                scene.objects.push(crate::scene::PlacedObject {
                    initial_index: None,
                    rejected,
                    runtime: true,
                    id,
                    plane,
                    x,
                    y,
                    shape: 10,
                    rotation: 0,
                });
            }
            let p = [
                scene.base_x + i32::from(inside),
                scene.base_y + i32::from(inside),
            ];
            assert_eq!(
                map_icon_candidates(&scene, &functions),
                vec![(p, 26), ([p[0] + 1, p[1] + 1], 5)]
            );
            assert_eq!(scene.objects.len(), 7);
        }
    }
}
