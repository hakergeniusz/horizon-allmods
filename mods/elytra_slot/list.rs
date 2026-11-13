// The mod's row in Options > Mods: the mod's identity and description. The
// slot answers in the inventory, so the row carries no keybinds.
// Copyright (c) 2026 hakergeniusz. MIT.
fn elytra_slot_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::elytra_slot as elytra;
    mods.push(crate::game::native_hooks::NativeMod {
        id: elytra::ID.into(),
        title: format!("Elytra Slot {}", elytra::VERSION),
        description: elytra::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
