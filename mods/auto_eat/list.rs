// The mod's row in Options > Mods: the mod's identity and what it does.
// Copyright (c) 2026 hakergeniusz. MIT.
fn auto_eat_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    mods.push(crate::game::native_hooks::NativeMod {
        id: "auto_eat".into(),
        title: "Auto Eat 0.1.0".into(),
        description: "Eats for you when hunger drops — picks food from the hotbar and holds use until the bite lands, whatever the server's rules. Keeps the cooked food in the hotbar and it does the rest.".into(),
        keys: vec![],
    });
}
