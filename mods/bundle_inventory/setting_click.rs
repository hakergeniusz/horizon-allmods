// A click on the mod's settings page: flip Enabled and keep it in the
// setting file the gates re-read, so the change lands within the tick that
// follows the click.
// Copyright (c) 2026 hakergeniusz. MIT.
fn bundle_inventory_page_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::bundle_inventory as bi;
    if mod_id != bi::ID {
        return;
    }
    let clicked = {
        let game = &*game;
        bi::page_click(button, bi::setting_on(|key| game.native_setting(bi::ID, key)))
    };
    if let Some(on) = clicked {
        let (key, value) = bi::stored_enabled(on);
        game.native_set_setting(bi::ID, key, &value);
    }
}
