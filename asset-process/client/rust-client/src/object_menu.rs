use crate::{
    context_menu::{Action, Entry},
    location_update::Pending,
    object::Definition,
    script_runtime::{Source, Variables},
    spell_action::Selection,
    Result,
};
pub fn entries(
    id: u16,
    tile: [u16; 2],
    def: &Definition,
    state: Option<&Pending>,
    selection: Option<&Selection>,
    hidden: bool,
    variables: &Variables,
    source: &mut impl Source,
) -> Result<Vec<Entry<Action>>> {
    let mut entries = Vec::new();
    if let Some(selection) = selection {
        if let Some(entry) = (!hidden)
            .then(|| selection.world_entry(0, id, tile, &format!("<col=ffff>{}", def.name)))
            .flatten()
        {
            entries.push(entry);
        }
        return Ok(entries);
    }
    let examine = Entry {
        option: "Examine".into(),
        target: format!("<col=ffff>{}", def.name),
        action: Action::Examine {
            kind: 1,
            id: def.id as u16,
            tile,
        },
        submenu: false,
        children: Vec::new(),
    };
    for i in (0..5).rev() {
        if hidden || state.is_some_and(|s| s.operations & (1 << i) == 0) {
            continue;
        }
        let label = if let Some(label) = state
            .and_then(|s| s.labels.as_ref())
            .and_then(|s| s[i as usize].as_ref())
        {
            Some(label.as_str())
        } else {
            crate::entity_ops::label(&def.operations, i, None, variables, source)?
        };
        if let Some(label) = label.filter(|label| !label.is_empty()) {
            entries.push(Entry {
                option: label.into(),
                target: format!("<col=ffff>{}", def.name),
                action: Action::Object {
                    id,
                    tile,
                    operation: i + 1,
                    sub: 0,
                },
                submenu: false,
                children: crate::entity_ops::children(&def.operations, i, variables, source)?
                    .into_iter()
                    .map(|(sub, option)| Entry {
                        option,
                        target: format!("<col=ffff>{}", def.name),
                        action: Action::Object {
                            id,
                            tile,
                            operation: i + 1,
                            sub,
                        },
                        submenu: false,
                        children: Vec::new(),
                    })
                    .collect(),
            });
        }
    }
    entries.push(examine);
    Ok(entries)
}

pub fn queue<'a>(
    root: &'a crate::location_update::Queue,
    worlds: &'a std::collections::HashMap<u16, crate::world_entity::State>,
    view: u16,
) -> Option<&'a crate::location_update::Queue> {
    if view == 0 {
        Some(root)
    } else {
        // dx.af resolves child menu picks through the root entity table.
        // Numeric registration order is only relevant to action dispatch.
        worlds
            .get(&0)
            .and_then(|state| state.entities.get(&view))
            .map(|entity| &entity.locations)
    }
}

pub fn present(scene: &crate::scene::Scene, plane: i32, tile: [u16; 2], id: u16) -> bool {
    scene.objects.iter().any(|object| {
        !object.rejected
            && object.id == id as u32
            && scene.object_scene_plane(object).map(|p| p as i32) == Some(plane)
            && object.x as i32 == tile[0] as i32 - scene.base_x
            && object.y as i32 == tile[1] as i32 - scene.base_y
    })
}
