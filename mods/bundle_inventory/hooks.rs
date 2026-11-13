// The game-side half: the reads and the one write every helper shares, as
// real module functions (a helper file holds exactly one function, so the
// common game-type work lives here instead). Nothing here decides — the
// runtime module does — this only reads the game's own fields into the
// runtime's snapshot shapes and turns a selection into the engine's own
// predict-and-send.
//
// Copyright (c) 2026 hakergeniusz. MIT.
use crate::game::bundle_inventory as bi;

/// The stack one of the player's own slots holds: the main inventory, or the
/// off hand. The panel's slots and the insert's targets are all these.
pub(crate) fn slot_stack<'a, 'g>(
    game: &'a crate::game::Game<'g>,
    slot: usize,
) -> Option<&'a horizon_client::items::ItemStack> {
    if slot == 504 {
        game.offhand.as_ref()
    } else {
        game.inventory.get(slot).and_then(|stack| stack.as_ref())
    }
}

/// Whether the mod's panel may act here: on, in a window, enabled, and the
/// plain survival inventory (no container, no workstation, the 2x2 grid, no
/// mount screen, no creative tabs).
pub(crate) fn gates(game: &crate::game::Game<'_>) -> bool {
    game.native_on(bi::ID)
        && game.native_window()
        && bi::setting_on(|key| game.native_setting(bi::ID, key))
        && bi::plain_inventory(
            game.container.is_none(),
            game.station.is_none(),
            game.craft_size == 2,
            !game.mount_active(),
            !game.creative_tab_open(),
        )
}

/// The snapshot a plan is built from: the open bundle's entries and
/// selection, and the player's own slots. None when the slot holds no bundle
/// — the caller closes the panel rather than act on it.
pub(crate) fn snapshot(
    game: &crate::game::Game<'_>,
    bundle_slot: usize,
) -> Option<bi::View> {
    use horizon_client::bundle::Contents;
    let stack = slot_stack(game, bundle_slot)?;
    let items = game.res.items();
    if !items.get(&stack.item).is_some_and(|info| info.bundle) {
        return None;
    }
    let contents = Contents::of(items, stack)?;
    let entries = contents
        .items()
        .iter()
        .map(|stack| (stack.item.to_string(), stack.count))
        .collect();
    Some(bi::View {
        bundle_slot,
        selected: stack.data().map_or(-1, |data| data.bundle_selected.0),
        entries,
        slots: (0..36)
            .chain([504])
            .map(|slot| {
                let item = slot_stack(game, slot).map(|stack| bi::SlotItem {
                    id: stack.item.to_string(),
                    count: stack.count,
                });
                (slot, item)
            })
            .collect(),
    })
}

/// Vanilla's bundle selection, the way the engine's own wheel does it: the
/// entry chosen in the client's copy of the stack, and on a server the
/// selection packet sent ahead of the click that acts on it. The runtime
/// says whether a select is needed at all (`toggleSelectedItem` toggles).
pub(crate) fn select(game: &mut crate::game::Game<'_>, bundle_slot: usize, index: i32) {
    let selected = game
        .slot_mut(bundle_slot)
        .as_ref()
        .and_then(|stack| stack.data())
        .map_or(-1, |data| data.bundle_selected.0);
    if !bi::select_needed(selected, usize::try_from(index).unwrap_or(usize::MAX)) {
        return;
    }
    if let Some(stack) = game.slot_mut(bundle_slot) {
        horizon_client::bundle::toggle_selected(stack, index);
    }
    if game.remote.is_some() {
        let (_, menu) = game.remote_menu();
        if let Some(menu_slot) = crate::game::Game::menu_slot_index(&menu, bundle_slot) {
            game.send(horizon_protocol::packets::play::Serverbound::SelectBundleItem(
                horizon_protocol::packets::play::SelectBundleItem {
                    slot: menu_slot as i32,
                    index,
                },
            ));
        }
    }
}

/// One plan step's menu click, as the engine's own verified menu engine
/// predicts it and a server hears it. False when the engine refused it:
/// vanilla keeps nothing of a refused click, so the plan stops there.
pub(crate) fn menu_click(
    game: &mut crate::game::Game<'_>,
    slot: Option<usize>,
    right: bool,
) -> bool {
    let ops = [(
        slot,
        i32::from(right),
        horizon_client::menu::Input::Pickup,
    )];
    game.menu_ops(&ops)
}
