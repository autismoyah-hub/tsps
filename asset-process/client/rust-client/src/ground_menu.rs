use crate::{
    context_menu::{Action, Entry},
    entity_ops, item,
    script_runtime::{Source, Variables},
    spell_action::Selection,
    zone, Result,
};
pub fn entries(
    pile: &[zone::Item],
    tile: [u16; 2],
    items: &item::Definitions,
    selection: Option<&Selection>,
    hidden: bool,
    variables: &Variables,
    source: &mut impl Source,
) -> Result<Vec<Entry<Action>>> {
    let mut entries = Vec::new();
    // State stores ap/aq (render) order; dx.af menus traverse al/ao in reverse.
    for item in pile.iter().rev() {
        let id = item.id;
        let Some(def) = items.items.get(&(id as u32)) else {
            continue;
        };
        let target = format!("<col=ff9040>{}", def.name);
        if let Some(selection) = selection {
            if let Some(entry) = (!hidden)
                .then(|| selection.world_entry(3, id, tile, &target))
                .flatten()
            {
                entries.push(entry);
            }
            continue;
        }
        for i in (0..5).rev() {
            if hidden || item.operations & (1 << i) == 0 {
                continue;
            }
            if let Some(label) =
                entity_ops::label(&def.ground_operations, i, None, variables, source)?
            {
                entries.push(Entry {
                    option: label.to_owned(),
                    target: target.clone(),
                    action: Action::Ground {
                        id,
                        tile,
                        operation: i + 1,
                        sub: 0,
                    },
                    submenu: false,
                    children: entity_ops::children(&def.ground_operations, i, variables, source)?
                        .into_iter()
                        .map(|(sub, option)| Entry {
                            option,
                            target: target.clone(),
                            action: Action::Ground {
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
        entries.push(Entry {
            option: "Examine".into(),
            target,
            action: Action::Examine { kind: 2, id, tile },
            submenu: false,
            children: Vec::new(),
        });
    }
    Ok(entries)
}
/// The native renderer gives up to three pile models distinct IDs; Java picks the tile once.
pub fn deduplicate_piles(picks: &mut Vec<(u64, u16)>) {
    let mut seen = std::collections::HashSet::new();
    picks.retain(|&(tag, view)| {
        tag & 0x1000000000000000 == 0 || seen.insert((view, tag & 0x0fffffff))
    });
}
