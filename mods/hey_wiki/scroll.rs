// The wheel over the wiki screen, before the screen scrolls: the page's
// text moves three lines a notch. The game's own link prompt, when it is
// up, keeps the wheel to itself.
// Copyright (c) 2026 hakergeniusz. MIT.
fn hey_wiki_screen_scroll(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    scroll: [f64; 2],
    modifiers: u32,
) -> core::ops::ControlFlow<()> {
    use crate::game::hey_wiki as wiki;
    let _ = modifiers;
    if screen.id != "mod" || game.native_mod_screen() != Some(wiki::ID) {
        return core::ops::ControlFlow::Continue(());
    }
    if !game.native_on(wiki::ID) || game.link_prompt.is_some() {
        return core::ops::ControlFlow::Continue(());
    }
    wiki::with_viewer(|viewer| viewer.scroll(scroll[1]));
    core::ops::ControlFlow::Break(())
}
