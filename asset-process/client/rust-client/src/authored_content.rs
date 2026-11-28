//! Explicit local content declaration for an opt-in authored live launcher.
use crate::{
    asset_package::Package, authored_floor::Floor, disk_cache::DiskCache, environment_motion::Mode,
    environment_style::Bindings, Error, Result,
};
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
};
struct Resource {
    path: PathBuf,
    pin: [u8; 32],
}
pub struct Content {
    package: Resource,
    floor: Resource,
    bindings: Resource,
    cutaway: Option<Resource>,
    map: Option<Resource>,
    motion: Mode,
}
impl Content {
    pub fn read(path: &Path) -> Result<Self> {
        let info = std::fs::symlink_metadata(path)?;
        if !info.is_file() || info.file_type().is_symlink() || info.len() > 65536 {
            return Err(Error::Invalid("content declaration bounds/type"));
        }
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(65537)
            .read_to_end(&mut bytes)?;
        Self::decode(&bytes, path.parent().unwrap_or(Path::new(".")))
    }
    pub fn decode(bytes: &[u8], base: &Path) -> Result<Self> {
        if bytes.len() > 65536 {
            return Err(Error::Invalid("content declaration size"));
        }
        let text =
            std::str::from_utf8(bytes).map_err(|_| Error::Invalid("content declaration UTF8"))?;
        let mut lines = text.lines();
        if lines.next() != Some("GIELINOR_AUTHORED_CONTENT 1") {
            return Err(Error::Invalid("content declaration version"));
        }
        let mut fields = BTreeMap::new();
        for line in lines {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .ok_or(Error::Invalid("content declaration field"))?;
            if !matches!(
                key,
                "package"
                    | "package_sha256"
                    | "floor"
                    | "floor_sha256"
                    | "bindings"
                    | "bindings_sha256"
                    | "cutaway"
                    | "cutaway_sha256"
                    | "map"
                    | "map_sha256"
                    | "motion"
            ) || value.is_empty()
                || fields.insert(key, value).is_some()
            {
                return Err(Error::Invalid("content declaration field"));
            }
        }
        fn resource(
            fields: &BTreeMap<&str, &str>,
            base: &Path,
            key: &str,
            pin_key: &str,
            required: bool,
        ) -> Result<Option<Resource>> {
            match (fields.get(key), fields.get(pin_key)) {
                (None, None) if !required => Ok(None),
                (Some(path), Some(pin)) => {
                    if pin.len() != 64 || !pin.bytes().all(|b| b.is_ascii_hexdigit()) {
                        return Err(Error::Invalid("content SHA256"));
                    }
                    let mut digest = [0; 32];
                    for (i, v) in digest.iter_mut().enumerate() {
                        *v = u8::from_str_radix(&pin[i * 2..i * 2 + 2], 16)
                            .map_err(|_| Error::Invalid("content SHA256"))?;
                    }
                    Ok(Some(Resource {
                        path: base.join(path),
                        pin: digest,
                    }))
                }
                _ => Err(Error::Invalid("content requires path and SHA256")),
            }
        }
        Ok(Self {
            package: resource(&fields, base, "package", "package_sha256", true)?.unwrap(),
            floor: resource(&fields, base, "floor", "floor_sha256", true)?.unwrap(),
            bindings: resource(&fields, base, "bindings", "bindings_sha256", true)?.unwrap(),
            cutaway: resource(&fields, base, "cutaway", "cutaway_sha256", false)?,
            map: resource(&fields, base, "map", "map_sha256", false)?,
            motion: fields.get("motion").copied().unwrap_or("off").parse()?,
        })
    }
    /// Consume the cache so an admission failure cannot escape as a partial live mount.
    pub fn mount(self, mut cache: DiskCache) -> Result<DiskCache> {
        let package = Package::read(&self.package.path)?;
        if package.identity() != self.package.pin {
            return Err(Error::Invalid("content package identity"));
        }
        let binding = Bindings::read(&self.bindings.path, self.bindings.pin, &package)?;
        let mut floor = Floor::read(&self.floor.path, self.floor.pin)?.with_bindings(binding)?;
        if self.motion != Mode::Off {
            floor = floor.with_motion(self.motion)?;
        }
        if let Some(r) = self.cutaway {
            floor = floor.with_cutaway(crate::environment_cutaway::Metadata::read(
                &r.path, r.pin, &package,
            )?)?;
        }
        if let Some(r) = self.map {
            floor = floor.with_map_functions(crate::environment_minimap::MapFunctions::read(
                &r.path, r.pin, &package,
            )?)?;
        }
        cache.mount_package(package, self.package.pin)?;
        cache.mount_floor(floor, self.floor.pin)?;
        Ok(cache)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> String {
        let pin = "01".repeat(32);
        format!("GIELINOR_AUTHORED_CONTENT 1\npackage=package dir\npackage_sha256={pin}\nfloor=floor.bwr\nfloor_sha256={pin}\nbindings=bindings.txt\nbindings_sha256={pin}\n")
    }
    #[test]
    fn declarations_resolve_paths_and_default_to_disabled_motion() {
        let c = Content::decode(config().as_bytes(), Path::new("fixture")).unwrap();
        assert_eq!(c.package.path, Path::new("fixture").join("package dir"));
        assert_eq!(c.motion, Mode::Off);
        assert!(c.cutaway.is_none() && c.map.is_none());
        let c = Content::decode(
            (config() + "motion=reduced\n").as_bytes(),
            Path::new("fixture"),
        )
        .unwrap();
        assert_eq!(c.motion, Mode::Reduced);
    }
    #[test]
    fn declarations_reject_partial_unknown_duplicate_and_unbounded_input() {
        for suffix in [
            "map=icons.txt\n",
            "cutaway_sha256=00\n",
            "password=secret\n",
            "motion=all\nmotion=off\n",
            "motion=invalid\n",
            "package=second\n",
        ] {
            assert!(Content::decode((config() + suffix).as_bytes(), Path::new(".")).is_err());
        }
        assert!(Content::decode(
            config().replace(&"01".repeat(32), "short").as_bytes(),
            Path::new(".")
        )
        .is_err());
        assert!(Content::decode(&vec![b'x'; 65537], Path::new(".")).is_err());
        assert!(Content::decode(b"GIELINOR_AUTHORED_CONTENT 2\n", Path::new(".")).is_err());
    }
}
