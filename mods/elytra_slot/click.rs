// A click on the survival inventory: when it lands on the mod's elytra slot,
// the whole equip or unequip runs here — as real menu clicks through the
// game's own verified menu engine (`menu_ops`), which predicts them and, on a
// server, sends them — and the vanilla click is skipped. The engine's own
// clicks reach `click_inventory` for every press over no engine slot, which a
// click over the mod's drawn slot is, on the window and a server alike.
// Copyright (c) 2026 hakergeniusz. MIT.
fn elytra_slot_click(
    game: &mut crate::game::Game<'_>,
    right: bool,
) -> core::ops::ControlFlow<()> {
    use crate::game::elytra_slot as elytra;
    let _ = right;
    if !game.native_on(elytra::ID) || !game.native_window() {
        return core::ops::ControlFlow::Continue(());
    }
    if !elytra::setting_on(|k| game.native_setting(elytra::ID, k)) {
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

    // The click is the mod slot's from here on: nothing vanilla runs after us.
    // Everything the plan needs is read before the menu engine is handed the
    // game, so the borrows never meet.
    let chest = game.armor[1].as_ref().map(|s| s.item.to_string());
    let cursor_elytra = game
        .carried
        .as_ref()
        .is_some_and(|s| &*s.item == elytra::ELYTRA);
    let ids: Vec<Option<String>> = game
        .inventory
        .iter()
        .map(|s| s.as_ref().map(|s| s.item.to_string()))
        .collect();
    let refs: Vec<Option<&str>> = ids.iter().map(|s| s.as_deref()).collect();
    let old_stored = elytra::stored_of(|k| game.native_setting(elytra::ID, k));
    let kept_item = old_stored
        .and_then(|i| game.inventory.get(i))
        .and_then(|s| s.as_ref().map(|s| s.item.to_string()));
    let kept_worthy = {
        let items = game.res.items();
        elytra::restore_worthy(
            kept_item
                .as_deref()
                .is_some_and(|id| items.get(id).and_then(|i| i.equip_slot.as_deref()) == Some("chest")),
            kept_item.as_deref(),
        )
    };
    let displaced = match chest.as_deref() {
        None => elytra::Displaced::Empty,
        Some(id) if id == elytra::ELYTRA => elytra::Displaced::Elytra,
        Some(_) => elytra::Displaced::Chestplate,
    };
    let action = elytra::click_action(
        cursor_elytra,
        elytra::is_elytra(chest.as_deref()),
        elytra::find_elytra(&refs),
    );
    let Some(action) = action else {
        // The slot is ours but holds nothing to reach: the click ends here,
        // as it would on any empty place of the panel.
        return core::ops::ControlFlow::Break(());
    };
    let free = elytra::find_free(&refs);
    let restore = match old_stored {
        Some(i) if kept_worthy => Some(i),
        _ => None,
    };
    let plan = match action {
        elytra::Action::Equip(from) => {
            elytra::plan_equip(from, displaced == elytra::Displaced::Chestplate, free)
        }
        elytra::Action::Unequip => elytra::plan_unequip(free, restore),
    };
    let ops: Vec<(Option<usize>, i32, horizon_client::menu::Input)> = plan
        .into_iter()
        .map(|slot| (slot, 0, horizon_client::menu::Input::Pickup))
        .collect();
    let ok = !ops.is_empty() && game.menu_ops(&ops);
    if !ok {
        // The menu engine refused the sequence: nothing moved, nothing to record.
        return core::ops::ControlFlow::Break(());
    }
    let worn_elytra = game
        .armor
        .get(1)
        .and_then(|s| s.as_ref())
        .is_some_and(|s| &*s.item == elytra::ELYTRA);
    match action {
        elytra::Action::Equip(_) if worn_elytra => {
            game.native_set_setting(elytra::ID, elytra::SETTING_EQUIPPED, "true");
            let kept = elytra::stored_after_equip(old_stored, displaced, free);
            game.native_set_setting(elytra::ID, elytra::SETTING_STORED, &elytra::stored_text(kept));
        }
        elytra::Action::Unequip if !worn_elytra => {
            game.native_set_setting(elytra::ID, elytra::SETTING_EQUIPPED, "false");
            game.native_set_setting(elytra::ID, elytra::SETTING_STORED, "");
        }
        _ => {}
    }
    core::ops::ControlFlow::Break(())
}
