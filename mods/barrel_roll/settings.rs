// The settings page: the single Enabled row, in the state the file holds
// (read fresh from disk, so the page always shows what the gates act on).
// Copyright (c) 2026 hakergeniusz. MIT.
fn barrel_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::barrel_roll as barrel;
    if mod_id != barrel::ID {
        return;
    }
    let game = &*game;
    let on = barrel::setting_on(|key| game.native_setting(barrel::ID, key));
    buttons.push(barrel::label(on));
}
