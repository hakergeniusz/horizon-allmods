// The mod's row in Options > Mods: one Enabled toggle, no keybind (the roll
// is a flight mode, not an action).
// Copyright (c) 2026 hakergeniusz. MIT.
fn barrel_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::barrel_roll as barrel;
    mods.push(crate::game::native_hooks::NativeMod {
        id: barrel::ID.into(),
        title: format!("Barrel Roll {}", barrel::VERSION),
        description: barrel::DESCRIPTION.into(),
        keys: vec![],
    });
}
