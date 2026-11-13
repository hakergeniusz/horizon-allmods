// A mod's own screen closing — Escape over the wiki, another screen taking
// over: the view and the deferred browser are forgotten, so nothing
// survives to leak into the next opening. Any mod's screen closing resets
// them (they only hold anything while this mod's own is up).
// Copyright (c) 2026 hakergeniusz. MIT.
fn hey_wiki_closed(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
) {
    use crate::game::hey_wiki as wiki;
    if screen.id != "mod" || !game.native_on(wiki::ID) {
        return;
    }
    wiki::reset_screen();
}
