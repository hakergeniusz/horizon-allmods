// A key went down or up, after rebinding (`mapped_key`): while the panel is
// open over the plain survival inventory, Escape closes the panel instead of
// the inventory, and Q over an entry drops that entry into the world — the
// vanilla drop key, acting on the entry under the cursor instead of the slot
// behind the panel. Presses only: a release must always reach the game's own
// bookkeeping. The drop defers to the tick, as a plan of ordinary menu
// clicks (select the entry, take it out, throw what the cursor carries).
// Copyright (c) 2026 hakergeniusz. MIT.
fn bundle_inventory_key(
    game: &crate::game::Game<'_>,
    key: winit::keyboard::KeyCode,
    pressed: bool,
    repeat: bool,
) -> core::ops::ControlFlow<Result<()>> {
    use crate::game::bundle_inventory as bi;
    use crate::game::bundle_inventory_hooks as hooks;
    if !pressed
        || repeat
        || game.screen != crate::game::Screen::Inventory
        || !hooks::gates(game)
    {
        return core::ops::ControlFlow::Continue(());
    }
    let Some(open) = bi::open_slot() else {
        return core::ops::ControlFlow::Continue(());
    };
    let Some(name) = horizon_client::keybinds::key_name(key) else {
        return core::ops::ControlFlow::Continue(());
    };
    match name {
        "key.keyboard.escape" => {
            bi::set_open(None);
            core::ops::ControlFlow::Break(Ok(()))
        }
        "key.keyboard.q" => {
            let Some(size) = game
                .renderer
                .as_ref()
                .map(|r| [r.config.width, r.config.height])
            else {
                return core::ops::ControlFlow::Continue(());
            };
            let batch = horizon_client::hud::HudBatch::new(size);
            if game.book_covers_menu(batch.width) {
                return core::ops::ControlFlow::Continue(());
            }
            let Some(view) = hooks::snapshot(game, open) else {
                return core::ops::ControlFlow::Continue(());
            };
            let (ox, oy) = game.inventory_origin(&batch);
            let panel = bi::panel_rect([ox, oy], batch.width);
            let at = [game.cursor.0, game.cursor.1];
            let Some(index) = bi::entry_at(at, panel, view.entries.len()) else {
                // Q over nothing of the panel's: the game's own drop key.
                return core::ops::ControlFlow::Continue(());
            };
            if game.carried.is_some() {
                // The cursor carries a stack of its own; a drop under it
                // would throw the wrong thing. The click stays vanilla's.
                return core::ops::ControlFlow::Continue(());
            }
            let steps = bi::plan(bi::Action::Drop(index), &view);
            if steps.is_empty() {
                return core::ops::ControlFlow::Continue(());
            }
            bi::defer(steps);
            core::ops::ControlFlow::Break(Ok(()))
        }
        _ => core::ops::ControlFlow::Continue(()),
    }
}
