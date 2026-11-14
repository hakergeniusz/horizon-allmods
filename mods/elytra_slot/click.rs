// A click on the survival inventory that lands on the mod's slot: the elytra
// moves between the cursor and the bay here, as direct state — the bay is the
// mod's own, so no menu clicks or packets exist to make. In single player and
// on LAN that IS the shared state (the client hosts the world); on a remote
// server the slot stays read-only. An elytra on the cursor stores, and the
// bay's own stack (if any) comes to the cursor, so no item is ever lost.
// The vanilla click is skipped whenever the click lands on the slot.
//
// 0.3.0: the save comes FIRST and is verified — 0.2.0 moved the item and
// then trusted `native_set_setting`, which in the live game silently wrote
// nothing, so the stored elytra died with the process. Now the file is
// written atomically and read back, and the game's stacks only move when the
// disk provably holds the new bay; a failed save leaves the cursor's item
// where it is and shows why on the settings page.
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
    // Nowhere to persist: the click answers as an empty patch of panel
    // rather than moving an item it could not save.
    let Ok(dir) = crate::game::Game::native_config_dir() else {
        return core::ops::ControlFlow::Break(());
    };

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
            let Some(cursor_stack) = game.carried.as_ref() else {
                return core::ops::ControlFlow::Break(());
            };
            let incoming = bay::into_text(cursor_stack);
            // The bay's old stack (if any) takes the cursor's place: a swap,
            // never a loss — but only once the file provably holds the bay.
            if elytra::save_bay(&dir, Some(&incoming)).is_err() {
                return core::ops::ControlFlow::Break(());
            }
            game.carried = held.as_ref().map(|old| bay::from_text(old));
            elytra::set(Some(incoming));
        }
        elytra::Action::Take => {
            let Some(old) = held else {
                return core::ops::ControlFlow::Break(());
            };
            // The file goes empty first: a failed save keeps the elytra in
            // the bay, and the next tick's restore refills it from the file.
            if elytra::save_bay(&dir, None).is_err() {
                return core::ops::ControlFlow::Break(());
            }
            elytra::set(None);
            game.carried = Some(bay::from_text(&old));
        }
    }
    core::ops::ControlFlow::Break(())
}
