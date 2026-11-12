// The mod's settings page: the single Enabled row, in the state the file
// holds (read fresh from disk, so the page always shows what the gates act
// on).
// Copyright (c) 2026 hakergeniusz. MIT.
fn fly_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::fly as fly;
    if mod_id != fly::ID {
        return;
    }
    let game = &*game;
    let on = fly::setting_on(|key| game.native_setting(fly::ID, key));
    buttons.push(fly::label(on));
}
