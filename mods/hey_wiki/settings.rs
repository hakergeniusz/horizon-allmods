// The mod's settings page: the four rows in the state the files hold (read
// fresh from disk, so the page always shows what the gates act on).
// Copyright (c) 2026 hakergeniusz. MIT.
fn hey_wiki_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::hey_wiki as wiki;
    if mod_id != wiki::ID {
        return;
    }
    let game = &*game;
    let enabled = wiki::setting_on(|key| game.native_setting(wiki::ID, key));
    let native = wiki::native_view(|key| game.native_setting(wiki::ID, key));
    let confirm = wiki::confirm_open(|key| game.native_setting(wiki::ID, key));
    let reach = wiki::reach_of(|key| game.native_setting(wiki::ID, key));
    buttons.extend(wiki::page_labels(enabled, native, confirm, reach));
}
