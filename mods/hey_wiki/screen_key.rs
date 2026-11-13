// A key on the wiki screen, before the screen acts on it: the mode answers
// (the box edits, Enter submits or opens, the results pick, the page
// scrolls), the URL a submit starts fetches natively — or, with the native
// view off, opens in the browser on the next tick. Held keys repeat through
// the events the engine sends; Escape is never the screen's, so the
// engine's own Escape still closes it.
// Copyright (c) 2026 hakergeniusz. MIT.
fn hey_wiki_screen_key(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    key: &str,
    repeat: bool,
    modifiers: u32,
) -> core::ops::ControlFlow<()> {
    use crate::game::hey_wiki as wiki;
    use crate::game::hey_wiki_hooks as hooks;
    let _ = (repeat, modifiers);
    if screen.id != "mod" || game.native_mod_screen() != Some(wiki::ID) {
        return core::ops::ControlFlow::Continue(());
    }
    if !game.native_on(wiki::ID) {
        return core::ops::ControlFlow::Continue(());
    }
    let keyed = wiki::with_viewer(|viewer| viewer.key(wiki::search_key_of(key)));
    match keyed {
        wiki::Keyed::Nothing => core::ops::ControlFlow::Continue(()),
        wiki::Keyed::Kept => core::ops::ControlFlow::Break(()),
        wiki::Keyed::Submit(query) => {
            let native = wiki::native_view(|k| game.native_setting(wiki::ID, k));
            match wiki::submit_plan(native, &query) {
                wiki::SubmitPlan::Native(query) => hooks::search(&query),
                wiki::SubmitPlan::Browser(url) => wiki::set_browser(url),
                wiki::SubmitPlan::Nothing => {}
            }
            core::ops::ControlFlow::Break(())
        }
        wiki::Keyed::Open(title) => {
            // The screen is already up (the results sent it): fetch alone.
            hooks::fetch_page(&title);
            core::ops::ControlFlow::Break(())
        }
    }
}
