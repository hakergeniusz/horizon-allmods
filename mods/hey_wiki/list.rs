// The mod's row in Options > Mods: the mod's identity, what it does, and
// its two keybind slots — the lookup key (H; Alt is the held item) and the
// search screen's key (B) — the same triples freecam's row carries.
// Copyright (c) 2026 hakergeniusz. MIT.
fn hey_wiki_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::hey_wiki as wiki;
    mods.push(crate::game::native_hooks::NativeMod {
        id: wiki::ID.into(),
        title: format!("Hey Wiki {}", wiki::VERSION),
        description: wiki::DESCRIPTION.into(),
        keys: vec![
            [
                wiki::OPEN_KEYBIND.into(),
                "Open Wiki Page".into(),
                wiki::DEFAULT_OPEN_KEY.into(),
            ],
            [
                wiki::SEARCH_KEYBIND.into(),
                "Wiki Search".into(),
                wiki::DEFAULT_SEARCH_KEY.into(),
            ],
        ],
    });
}
