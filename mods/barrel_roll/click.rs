// A click on the mod's settings page: flip Enabled and keep it in the
// setting file the gates re-read, so the change lands within the tick that
// follows the click.
// Copyright (c) 2026 hakergeniusz. MIT.
fn barrel_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::barrel_roll as barrel;
    if mod_id != barrel::ID {
        return;
    }
    let clicked = {
        let game = &*game;
        barrel::click(button, barrel::setting_on(|key| {
            game.native_setting(barrel::ID, key)
        }))
    };
    if let Some(on) = clicked {
        let (key, value) = barrel::stored(on);
        game.native_set_setting(barrel::ID, key, &value);
    }
}
