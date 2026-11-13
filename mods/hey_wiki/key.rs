// Wherever a key goes down or up: the mod's two keys act here, before the
// game's own keybinds. H looks up what the crosshair is on over the world
// (or the plain survival inventory's hovered slot while a screen is open),
// Alt+H the item in the main hand anywhere, B the wiki screen. What the
// answer becomes — the page fetched into the game's own screen, or the
// browser behind the game's link confirmation — the native-view setting
// routes. The keys are not broken: H and B are unbound in the game (H
// carries only the debug screen's advanced-tooltips toggle).
// Copyright (c) 2026 hakergeniusz. MIT.
fn hey_wiki_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::hey_wiki as wiki;
    use crate::game::hey_wiki_hooks as hooks;
    if !game.native_on(wiki::ID) {
        return;
    }
    let enabled = wiki::setting_on(|k| game.native_setting(wiki::ID, k));
    let open_key = game.native_mod_key(wiki::ID, wiki::OPEN_KEYBIND, wiki::DEFAULT_OPEN_KEY);
    let search_key = game.native_mod_key(wiki::ID, wiki::SEARCH_KEYBIND, wiki::DEFAULT_SEARCH_KEY);
    let press = wiki::route(
        pressed,
        enabled,
        game.native_playing(),
        game.native_mod_screen() == Some(wiki::ID),
        key,
        &open_key,
        &search_key,
        game.has_alt_down(),
    );
    match press {
        wiki::Press::None => {}
        wiki::Press::Search => {
            wiki::reset_screen();
            game.native_mod_screen_open(wiki::ID);
        }
        wiki::Press::Held | wiki::Press::Lookup => {
            // The id of the answer: Alt+H is the hand's item; H is the
            // crosshair's target (or the hovered slot over a screen).
            let id = if press == wiki::Press::Held {
                game.held().map(|s| s.item.to_string())
            } else {
                hooks::lookup_id(game, false)
            };
            // What it becomes, as the native-view setting sends it: the
            // page fetched into the mod's own screen, or the browser behind
            // the game's link confirmation.
            let native = wiki::native_view(|k| game.native_setting(wiki::ID, k));
            match wiki::h_plan(native, id) {
                wiki::HPlan::Native(title) => hooks::show_page(game, &title),
                wiki::HPlan::Browser(url) => hooks::open_url(game, url),
                wiki::HPlan::Nothing => {}
            }
        }
    }
}
