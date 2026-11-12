// A click on the mod's settings page: flip Enabled and keep it in the
// setting file the gates re-read, so the change lands within the tick that
// follows the click.
// Copyright (c) 2026 hakergeniusz. MIT.
fn fly_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::fly as fly;
    if mod_id != fly::ID {
        return;
    }
    let clicked = {
        let game = &*game;
        fly::click(button, fly::setting_on(|key| {
            game.native_setting(fly::ID, key)
        }))
    };
    if let Some(on) = clicked {
        let (key, value) = fly::stored(on);
        game.native_set_setting(fly::ID, key, &value);
    }
}
