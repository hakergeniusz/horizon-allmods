// Every tick: the slot's self-healing. The elytra can leave the chest slot
// by outside means — a death (it drops, like any worn armor), a vanilla click
// on the chest slot, another mod — and when it does, the mod's tracking must
// follow: the slot lets go at once, and the chestpiece it kept goes back on
// through the game's own menu clicks (on a server only once the inventory is
// open, which is when clicks go out there at all).
// Copyright (c) 2026 hakergeniusz. MIT.
fn elytra_slot_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::elytra_slot as elytra;
    if !game.native_on(elytra::ID) || !game.native_window() {
        return;
    }
    if !elytra::setting_on(|k| game.native_setting(elytra::ID, k)) {
        return;
    }
    let equipped = elytra::equipped_of(|k| game.native_setting(elytra::ID, k));
    let stored = elytra::stored_of(|k| game.native_setting(elytra::ID, k));
    let chest = game.armor[1].as_ref().map(|s| s.item.to_string());
    let kept_item = stored
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
    // On a server the verified menu engine sends clicks only while the
    // inventory screen is open; anywhere else a session would predict a move
    // the server never hears of. Single player and LAN have no such gate.
    let can_menu = game.remote.is_none() || game.screen == crate::game::Screen::Inventory;
    let disarmed = elytra::disarms(equipped, chest.as_deref());
    if disarmed {
        game.native_set_setting(elytra::ID, elytra::SETTING_EQUIPPED, "false");
    }
    match elytra::kept_step(chest.as_deref(), stored, kept_worthy, can_menu, disarmed) {
        elytra::Kept::Keep => {}
        elytra::Kept::Clear => {
            game.native_set_setting(elytra::ID, elytra::SETTING_STORED, "");
        }
        elytra::Kept::Wear(slot) => {
            let ops: Vec<(Option<usize>, i32, horizon_client::menu::Input)> = elytra::plan_restore(slot)
                .into_iter()
                .map(|slot| (slot, 0, horizon_client::menu::Input::Pickup))
                .collect();
            let ok = !ops.is_empty() && game.menu_ops(&ops);
            if ok && game.armor[1].is_some() {
                game.native_set_setting(elytra::ID, elytra::SETTING_STORED, "");
            }
        }
    }
}
