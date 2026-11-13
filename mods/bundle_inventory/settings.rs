// The mod's settings page: the Enabled toggle, read fresh from the setting
// file so the page always shows what the gates act on.
// Copyright (c) 2026 hakergeniusz. MIT.
fn bundle_inventory_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::bundle_inventory as bi;
    if mod_id != bi::ID {
        return;
    }
    let game = &*game;
    let on = bi::setting_on(|key| game.native_setting(bi::ID, key));
    buttons.extend(bi::labels(on));
}
