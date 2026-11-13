// Every tick: what the screen helper deferred — a link for the clipboard,
// or a browser opening (the search box's Enter with the native view off,
// the page's Open in browser button) — happens here, where the &mut game
// is. The browser goes where the confirmation setting sends it, the game's
// own link prompt or straight to the browser, and the screen closes first,
// so the prompt reads over the world as it always does.
// Copyright (c) 2026 hakergeniusz. MIT.
fn hey_wiki_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::hey_wiki as wiki;
    if let Some(url) = wiki::take_copy() {
        use horizon_client::clipboard::Clipboard;
        game.clipboard.set(&url);
    }
    let Some(url) = wiki::take_browser() else {
        return;
    };
    if game.native_mod_screen() == Some(wiki::ID) {
        game.close_mod_screen();
    }
    let confirm = wiki::confirm_open(|k| game.native_setting(wiki::ID, k));
    match wiki::open_plan(confirm) {
        wiki::OpenPlan::Prompt => game.link_prompt = Some(url),
        wiki::OpenPlan::Direct => crate::game::packs::open_uri(&url),
    }
}
