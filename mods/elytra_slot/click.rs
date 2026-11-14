// A click on the survival inventory that lands on the mod's slot: the elytra
// moves between the cursor and the bay here, as direct state — the bay is the
// mod's own, so no menu clicks or packets exist to make. In single player and
// on LAN that IS the shared state (the client hosts the world); on a remote
// server the slot stays read-only. An elytra on the cursor stores, and the
// bay's own stack (if any) comes to the cursor, so no item is ever lost.
// The vanilla click is skipped whenever the click lands on the slot.
// Copyright (c) 2026 hakergeniusz. MIT.
fn elytra_slot_click(
    game: &mut crate::game::Game<'_>,
    right: bool,
) -> core::ops::ControlFlow<()> {
    use crate::game::elytra_slot as elytra;
    use crate::game::elytra_slot_bay as bay;
    let _ = right;
    if !game.native_on(elytra::ID) || !game.native_window() {
        return core::ops::ControlFlow::Continue(());
    }
    if !elytra::setting_on(|k| game.native_setting(elytra::ID, k)) {
        return core::ops::ControlFlow::Continue(());
    }
    if game.remote.is_some() {
        return core::ops::ControlFlow::Continue(());
    }
    if !elytra::plain_inventory(
        game.container.is_none(),
        game.station.is_none(),
        game.craft_size == 2,
        game.mount_active(),
        game.creative_tab_open(),
    ) {
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
    let origin = game.inventory_origin(&batch);
    if !elytra::over_slot(game.cursor, origin) {
        return core::ops::ControlFlow::Continue(());
    }

    // The click is the slot's from here on: nothing vanilla runs after us.
    let cursor_item = game.carried.as_ref().map(|s| s.item.to_string());
    let held = elytra::held();
    let Some(action) = elytra::click_action(cursor_item.as_deref(), held.as_ref()) else {
        // The slot is ours but answers nothing (a non-elytra on the cursor):
        // the click ends here, as it would on any empty place of the panel.
        return core::ops::ControlFlow::Break(());
    };
    match action {
        elytra::Action::Store => {
            let incoming = game
                .carried
                .take()
                .map(|stack| bay::into_text(&stack))
                .expect("the action asked for the cursor's elytra");
            // The bay's old stack (if any) takes the cursor's place: a swap,
            // never a loss.
            game.carried = held.as_ref().map(|old| bay::from_text(old));
            elytra::set(Some(incoming.clone()));
            for (key, value) in elytra::texts(Some(&incoming)) {
                game.native_set_setting(elytra::ID, key, &value);
            }
        }
        elytra::Action::Take => {
            let old = held.expect("the action asked for the bay's elytra");
            game.carried = Some(bay::from_text(&old));
            elytra::set(None);
            for (key, value) in elytra::texts(None) {
                game.native_set_setting(elytra::ID, key, &value);
            }
        }
    }
    core::ops::ControlFlow::Break(())
}
