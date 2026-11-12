// The mod's row in Options > Mods: one toggle and its keybind slot (the
// setting name, the label the controls show, and the key out of the box —
// the same triple freecam's row carries).
// Copyright (c) 2026 hakergeniusz. MIT.
fn fly_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::fly as fly;
    mods.push(crate::game::native_hooks::NativeMod {
        id: fly::ID.into(),
        title: format!("Fly {}", fly::VERSION),
        description: fly::DESCRIPTION.into(),
        keys: vec![[
            fly::TOGGLE.into(),
            "Toggle Fly".into(),
            fly::DEFAULT_TOGGLE_KEY.into(),
        ]],
    });
}
