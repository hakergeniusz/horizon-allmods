// Every tick: what the screen helper deferred — a link for the clipboard,
// or a browser opening (the search box's Enter with the native view off,
// the page's Open in browser button) — happens here, where the &mut game
// is. The browser goes where the confirmation setting sends it, the game's
// own link prompt or straight to the browser, and the screen closes first,
// so the prompt reads over the world as it always does.
// Copyright (c) 2026 hakergeniusz. MIT.
fn hey_wiki_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::hey_wiki as wiki;
    // Playing means captured. The engine re-grabs on a screen close and on
    // a refocus that lost a held pointer, but a browser round-trip hands
    // the focus over while the pointer is already free — the link prompt
    // needs the cursor — and nothing re-grabs on the way back. The tick
    // re-asserts what playing means; the prompt dialog and its kin keep the
    // cursor (a grab would pin it at the panel's center, where the prompt's
    // buttons are not).
    if wiki::should_regrab(
        game.screen == crate::game::Screen::Playing,
        game.focused,
        game.keyboard_mouse,
        game.captured,
        game.screen_over_playing(),
    ) {
        game.capture(true);
    }
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
        wiki::OpenPlan::Prompt => {
            // The prompt is answered with the cursor, as vanilla's always
            // is — it only opens over free-cursor screens (the chat, the
            // links screen). Closing the wiki screen held the pointer; let
            // it go before the prompt draws. The tick re-grabs when the
            // prompt is answered no, and the engine's own refocus re-grab
            // takes it back after the browser round-trip on yes.
            if game.captured {
                game.capture(false);
            }
            game.link_prompt = Some(url);
        }
        wiki::OpenPlan::Direct => crate::game::packs::open_uri(&url),
    }
}
