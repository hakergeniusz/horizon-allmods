// A press on the survival inventory, before anything the screen does with it
// (`ScreenEvent.MouseButtonPressed.Pre`): this is the modal panel's whole
// routing. Shift + right-click on a bundle opens it; with the panel open a
// click on an entry takes it (shift: into the inventory), a click on any
// other player slot puts that stack in, and a click anywhere else closes the
// panel — which also keeps the click from falling through and dropping what
// the cursor carries. Every move is deferred to the tick as a plan of
// ordinary menu clicks, because this hook reads the game through a shared
// borrow and the menu clicks want the exclusive one.
// Copyright (c) 2026 hakergeniusz. MIT.
fn bundle_inventory_screen_click(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    button: i32,
    modifiers: u32,
) -> core::ops::ControlFlow<()> {
    use crate::game::bundle_inventory as bi;
    use crate::game::bundle_inventory_hooks as hooks;
    let _ = modifiers;
    if screen.id != "inventory" || screen.menu.is_some() || !hooks::gates(game) {
        return core::ops::ControlFlow::Continue(());
    }
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
    let at = [game.cursor.0, game.cursor.1];
    let Some(open) = bi::open_slot() else {
        // Closed: shift + right-click on a bundle in one of the player's own
        // slots opens the panel. Vanilla's right-click on a bundle takes one
        // out, so this press is ours to take instead.
        if button == 1 && game.has_shift_down() {
            let (target, _) = game.inventory_target();
            if let Some(hit) = target
                && bi::player_slot(hit)
                && hooks::slot_stack(game, hit)
                    .is_some_and(|stack| game.res.items().get(&stack.item).is_some_and(|i| i.bundle))
            {
                bi::set_open(Some(hit));
                return core::ops::ControlFlow::Break(());
            }
        }
        return core::ops::ControlFlow::Continue(());
    };
    let Some(view) = hooks::snapshot(game, open) else {
        // The bundle left the slot between the frame and the click: close
        // and keep the click from falling anywhere.
        bi::set_open(None);
        return core::ops::ControlFlow::Break(());
    };
    let (ox, oy) = game.inventory_origin(&batch);
    let panel = bi::panel_rect([ox, oy], batch.width);
    if bi::in_rect(at, panel) {
        // The panel, its chrome included, never lets a click through to
        // whatever it covers. With nothing on the cursor an entry answers:
        // plain click takes it, shift-click moves it into the inventory.
        if game.carried.is_none()
            && let Some(index) = bi::entry_at(at, panel, view.entries.len())
        {
            let action = if game.has_shift_down() {
                bi::Action::TransferOut(index)
            } else {
                bi::Action::Extract(index)
            };
            let steps = bi::plan(action, &view);
            if !steps.is_empty() {
                bi::defer(steps);
            }
        }
        return core::ops::ControlFlow::Break(());
    }
    // Outside the panel: the modal insert. A click on another player slot
    // with an empty cursor puts that stack into the open bundle (what fits;
    // the rest back where it came from). The bundle's own slot stays
    // vanilla's, a click with the cursor carrying stays vanilla's, and a
    // click on nothing at all closes the panel — and keeps the vanilla
    // click-outside from dropping what the cursor carries.
    let (target, _) = game.inventory_target();
    match target {
        Some(hit) if hit == open => core::ops::ControlFlow::Continue(()),
        Some(hit)
            if bi::player_slot(hit)
                && game.carried.is_none()
                && hooks::slot_stack(game, hit).is_some_and(|stack| bi::can_insert(&stack.item)) =>
        {
            let steps = bi::plan(bi::Action::Insert(hit), &view);
            if !steps.is_empty() {
                bi::defer(steps);
            }
            core::ops::ControlFlow::Break(())
        }
        Some(_) => core::ops::ControlFlow::Continue(()),
        None => {
            bi::set_open(None);
            core::ops::ControlFlow::Break(())
        }
    }
}
