// The mod's row in Options > Mods: the mod's identity and description. The
// panel answers in the inventory, so the row carries no keybinds.
// Copyright (c) 2026 hakergeniusz. MIT.
fn bundle_inventory_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::bundle_inventory as bi;
    mods.push(crate::game::native_hooks::NativeMod {
        id: bi::ID.into(),
        title: format!("Bundle Inventory {}", bi::VERSION),
        description: bi::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
