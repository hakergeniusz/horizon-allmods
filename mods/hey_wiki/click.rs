// A press on the wiki screen, before the screen acts on it: the page's
// buttons (the browser fallback, the link copy, back to the box) and a
// result's row answer here, the plan the brain made. The game's own link
// prompt, when it is up, answers first instead — the button that opened it
// keeps quiet until it is gone.
// Copyright (c) 2026 hakergeniusz. MIT.
fn hey_wiki_screen_click(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    button: i32,
    modifiers: u32,
) -> core::ops::ControlFlow<()> {
    use crate::game::hey_wiki as wiki;
    let _ = modifiers;
    if screen.id != "mod" || game.native_mod_screen() != Some(wiki::ID) {
        return core::ops::ControlFlow::Continue(());
    }
    if !game.native_on(wiki::ID) || button != 0 || game.link_prompt.is_some() {
        // Not the mouse's left button, or the engine's own open-or-copy
        // dialog owns this click.
        return core::ops::ControlFlow::Continue(());
    }
    let cursor = screen.mouse;
    let panel = [
        (screen.rect[2] - wiki::PANEL_W) / 2.0,
        (screen.rect[3] - wiki::PANEL_H) / 2.0,
        wiki::PANEL_W,
        wiki::PANEL_H,
    ];
    let plan = wiki::with_viewer(|viewer| wiki::click_plan(viewer, panel, cursor));
    match plan {
        wiki::ClickPlan::None => core::ops::ControlFlow::Continue(()),
        wiki::ClickPlan::Browser => {
            // The browser waits for the tick, which holds the &mut the
            // confirmation and the opening need.
            if let Some(url) = wiki::with_viewer(|viewer| viewer.page_url()) {
                wiki::set_browser(url);
            }
            core::ops::ControlFlow::Break(())
        }
        wiki::ClickPlan::Copy => {
            // The clipboard wants the &mut the tick's helper holds; the link
            // waits there.
            if let Some(url) = wiki::with_viewer(|viewer| viewer.page_url()) {
                wiki::set_copy(url);
            }
            core::ops::ControlFlow::Break(())
        }
        wiki::ClickPlan::Back => {
            wiki::with_viewer(|viewer| viewer.go_back());
            core::ops::ControlFlow::Break(())
        }
        wiki::ClickPlan::Result(index) => {
            let title = wiki::with_viewer(|viewer| viewer.result_title(index));
            if let Some(title) = title {
                crate::game::hey_wiki_hooks::fetch_page(&title);
            }
            core::ops::ControlFlow::Break(())
        }
    }
}
