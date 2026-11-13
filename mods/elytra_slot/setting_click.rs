// A click on the mod's settings page: flip Enabled and keep it in the
// setting file the gates re-read, so the change lands within the tick that
// follows the click. The page's other rows are read-only.
// Copyright (c) 2026 hakergeniusz. MIT.
fn elytra_slot_page_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::elytra_slot as elytra;
    if mod_id != elytra::ID {
        return;
    }
    let clicked = {
        let game = &*game;
        elytra::page_click(
            button,
            elytra::setting_on(|key| game.native_setting(elytra::ID, key)),
        )
    };
    if let Some(on) = clicked {
        let (key, value) = elytra::stored_enabled(on);
        game.native_set_setting(elytra::ID, key, &value);
    }
}
