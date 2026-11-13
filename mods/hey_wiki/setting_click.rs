// A click on the mod's settings page: the row's change goes straight into
// the setting file the gates re-read, so it lands within the tick that
// follows the click.
// Copyright (c) 2026 hakergeniusz. MIT.
fn hey_wiki_page_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::hey_wiki as wiki;
    if mod_id != wiki::ID {
        return;
    }
    let click = {
        let game = &*game;
        wiki::page_click(
            button,
            wiki::setting_on(|key| game.native_setting(wiki::ID, key)),
            wiki::native_view(|key| game.native_setting(wiki::ID, key)),
            wiki::confirm_open(|key| game.native_setting(wiki::ID, key)),
            wiki::reach_of(|key| game.native_setting(wiki::ID, key)),
        )
    };
    if let Some((key, value)) = wiki::stored(click) {
        game.native_set_setting(wiki::ID, key, &value);
    }
}
