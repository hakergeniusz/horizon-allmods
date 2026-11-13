// Elytra Slot's brain: pure standard library, no game types, so it compiles
// and tests wherever — as this crate's library and, injected, as a module of
// the game. Every decision the helpers act on lives here, and every one of
// those decisions is tested below.
//
// The model: Horizon's survival inventory has one chest slot, so the mod's
// dedicated elytra slot is the chest slot worn as a wing bay. Equipping
// through the mod's slot moves a real elytra into the engine's real chest
// armor slot (every flight check, the wings renderer and the server all see
// honest equipment), and the chestplate it displaces is kept by the mod —
// tracked in its settings — until the elytra is unequipped and the
// chestplate is worn again by one click. What the engine cannot do is render
// two chest items at once, so while the elytra is equipped the chestplate is
// stored rather than visible; that is the port's one deviation from the
// original, where a Curios slot keeps both worn.
//
// Clean-room: written for Horizon from the game's own public behavior; it
// copies nothing from any mod.
//
// Copyright (c) 2026 hakergeniusz. MIT.

/// The mod's registry id: the settings folder and the Options > Mods row.
pub const ID: &str = "elytra_slot";
/// The version the row shows.
pub const VERSION: &str = "0.1.0";
/// The mod's description for its row.
pub const DESCRIPTION: &str = "The survival inventory gains an elytra slot beside the player \
model. Click it to wear an elytra and keep flying while your chestplate is kept safe, and \
click again to swap the two back. Works in single player, on LAN and on servers, because \
the elytra is worn in the real chest slot.";

/// The elytra: the only item the mod's slot equips, as the original's
/// `#elytraslot:elytra` tag holds.
pub const ELYTRA: &str = "minecraft:elytra";

/// The engine's chest armor slot (`armor[1]`, the inventory menu's slot 501,
/// the verified menu engine's slot 6): where the equipped elytra really goes.
pub const CHEST_SLOT: usize = 501;

/// The mod slot's place in the plain survival inventory, in panel-relative
/// GUI pixels: right of the player model (`ox + 26 .. ox + 75`), above the
/// offhand slot at (77, 62), clear of the recipe book's button at (104, 61).
pub const SLOT_X: f32 = 77.0;
pub const SLOT_Y: f32 = 8.0;
/// An inventory slot is 18 GUI pixels wide (16 plus the frame).
pub const SLOT_SIZE: f32 = 18.0;

/// The player inventory's 36 slots: 0..9 hotbar, 9..36 main.
pub const MAX_SLOT: usize = 36;

/// The mod's setting files (one file per key under the mod's data dir),
/// which every gate re-reads: a change lands within the tick that follows it.
pub const SETTING_ENABLED: &str = "enabled";
pub const SETTING_EQUIPPED: &str = "equipped";
pub const SETTING_STORED: &str = "stored";

/// Whether the mod is on. A missing file means ON — the mod exists for the
/// slot, so installing it is asking for it; anything but `false` in the file
/// is on, trimmed like the game reads its settings.
pub fn setting_on(get: impl FnOnce(&str) -> Option<String>) -> bool {
    !matches!(get(SETTING_ENABLED).as_deref(), Some(v) if v.trim() == "false")
}

/// Whether the mod's slot currently holds the elytra: the `equipped` file
/// says "true" and nothing else does.
pub fn equipped_of(get: impl FnOnce(&str) -> Option<String>) -> bool {
    matches!(get(SETTING_EQUIPPED).as_deref(), Some(v) if v.trim() == "true")
}

/// The slot the kept chestplate waits in, as the `stored` file holds it: a
/// decimal player-inventory index under 36. An empty, missing or out-of-range
/// file keeps nothing.
pub fn stored_of(get: impl FnOnce(&str) -> Option<String>) -> Option<usize> {
    get(SETTING_STORED)
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|i| *i < MAX_SLOT)
}

/// What to write for a kept chestplate's slot: its decimal index, or "" for
/// none (the game's settings are plain text files).
pub fn stored_text(stored: Option<usize>) -> String {
    stored.map_or(String::new(), |i| i.to_string())
}

/// Whether the open screen is the plain survival inventory — the only one the
/// mod's slot draws on and answers in. The inventory screen also hosts every
/// container, station and mount window; those have a menu, a container, a
/// station, a mount menu or the 3x3 crafting table's size, and the creative
/// screen runs its own tab.
pub fn plain_inventory(
    container_none: bool,
    station_none: bool,
    craft_two: bool,
    mount: bool,
    creative_tab: bool,
) -> bool {
    container_none && station_none && craft_two && !mount && !creative_tab
}

/// Whether the cursor is over the mod's slot, drawn at `origin` (the panel's
/// top left in GUI pixels).
pub fn over_slot(cursor: (f32, f32), origin: (f32, f32)) -> bool {
    let (x, y) = (origin.0 + SLOT_X, origin.1 + SLOT_Y);
    cursor.0 >= x && cursor.0 < x + SLOT_SIZE && cursor.1 >= y && cursor.1 < y + SLOT_SIZE
}

/// Whether this stack's item is the elytra.
pub fn is_elytra(item: Option<&str>) -> bool {
    item == Some(ELYTRA)
}

/// The first elytra in the player's inventory: the main inventory first, then
/// the hotbar, as vanilla keeps worn-out of the way of what is in hand.
pub fn find_elytra(items: &[Option<&str>]) -> Option<usize> {
    (9..MAX_SLOT)
        .chain(0..9)
        .find(|&i| is_elytra(items.get(i).copied().flatten()))
}

/// The first free player-inventory slot: the hotbar first, then the main
/// inventory (`Inventory.getFreeSlot`).
pub fn find_free(items: &[Option<&str>]) -> Option<usize> {
    (0..9)
        .chain(9..MAX_SLOT)
        .find(|&i| items.get(i).is_none_or(|s| s.is_none()))
}

/// What the chest slot held before an equip: nothing, the elytra that is
/// being swapped for another one, or a chestpiece the mod must keep.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Displaced {
    Empty,
    Elytra,
    Chestplate,
}

/// What a click on the mod's slot does: equip the elytra (from the cursor
/// when one is carried, else the inventory's first), or unequip the worn one.
/// Nothing when there is no elytra to reach and none worn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Equip(Option<usize>),
    Unequip,
}

pub fn click_action(
    cursor_elytra: bool,
    chest_elytra: bool,
    inv_elytra: Option<usize>,
) -> Option<Action> {
    if cursor_elytra {
        Some(Action::Equip(None))
    } else if chest_elytra {
        Some(Action::Unequip)
    } else {
        inv_elytra.map(|slot| Action::Equip(Some(slot)))
    }
}

/// The equip's clicks, as left-button pickups on game slots, in order: take
/// the elytra (`from`, or it is already on the cursor), put it on the chest
/// slot, and — when the chest slot held a chestpiece — put that where it
/// waits. With no free slot the chestpiece stays on the cursor, like a
/// vanilla armor-slot swap does.
pub fn plan_equip(
    from: Option<usize>,
    chest_busy: bool,
    free: Option<usize>,
) -> Vec<Option<usize>> {
    let mut ops = Vec::new();
    if let Some(slot) = from {
        ops.push(Some(slot));
    }
    ops.push(Some(CHEST_SLOT));
    if chest_busy && let Some(slot) = free {
        ops.push(Some(slot));
    }
    ops
}

/// The unequip's clicks: take the elytra off the chest slot, put it away when
/// a slot is free, and wear the kept chestpiece back. With no free slot the
/// kept chestpiece's own slot takes the elytra by the swap (a pickup on an
/// occupied slot with a different stack), so the click still ends bare-handed.
pub fn plan_unequip(free: Option<usize>, restore: Option<usize>) -> Vec<Option<usize>> {
    let mut ops = vec![Some(CHEST_SLOT)];
    if let Some(slot) = free {
        ops.push(Some(slot));
    }
    if let Some(slot) = restore {
        ops.push(Some(slot));
        ops.push(Some(CHEST_SLOT));
    }
    ops
}

/// The kept chestpiece's clicks for the tick's own restore: take it from its
/// slot, wear it.
pub fn plan_restore(stored: usize) -> Vec<Option<usize>> {
    vec![Some(stored), Some(CHEST_SLOT)]
}

/// Whether the mod's slot must let go of the elytra this tick: the `equipped`
/// file says so but the chest slot holds anything else — the elytra left by
/// outside means (death, a vanilla slot click, another mod).
pub fn disarms(equipped: bool, chest: Option<&str>) -> bool {
    equipped && !is_elytra(chest)
}

/// Whether a kept stack is the chestpiece the mod kept: the engine's item
/// table puts it in the chest slot (`equip_slot` chest), and it is not an
/// elytra — a second elytra the player parked there is their own business,
/// and wearing one back would fight the very unequip the player just made.
pub fn restore_worthy(chest_valid: bool, item: Option<&str>) -> bool {
    chest_valid && item.is_some() && !is_elytra(item)
}

/// What this tick does with the kept chestpiece's slot, after any disarm:
/// wear it back when the chest slot is free and a menu click can run (on a
/// server the clicks go out only while the inventory is open, so the restore
/// waits for one), clear the tracking when the slot no longer holds the
/// chestpiece, and otherwise keep waiting. A disarm over a worn chestpiece —
/// the player swapped pieces themselves — also ends the tracking: what was
/// kept stays where it is, in plain sight in the inventory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kept {
    Keep,
    Clear,
    Wear(usize),
}

pub fn kept_step(
    chest: Option<&str>,
    stored: Option<usize>,
    worthy: bool,
    can_menu: bool,
    disarmed: bool,
) -> Kept {
    let chest_free = chest.is_none();
    if disarmed && !chest_free && stored.is_some() {
        return Kept::Clear;
    }
    match (stored, worthy, chest_free, can_menu) {
        (Some(i), true, true, true) => Kept::Wear(i),
        (Some(_), false, _, _) => Kept::Clear,
        _ => Kept::Keep,
    }
}

/// What the `stored` file holds after a successful equip: the free slot a
/// displaced chestpiece was put in, or the tracking the equip inherited (the
/// chest slot held another elytra, or nothing at all). A chestpiece with
/// nowhere to go is on the cursor, untracked.
pub fn stored_after_equip(
    old: Option<usize>,
    displaced: Displaced,
    free: Option<usize>,
) -> Option<usize> {
    match displaced {
        Displaced::Chestplate => free,
        Displaced::Elytra | Displaced::Empty => old,
    }
}

/// The settings page's two rows: the Enabled toggle and a read-only line
/// saying what the slot holds, so the state is visible without the inventory.
pub fn labels(on: bool, worn: bool, kept: Option<&str>) -> Vec<String> {
    let mut rows = vec![format!("Enabled: {}", if on { "ON" } else { "OFF" })];
    rows.push(if worn {
        "Slot: elytra worn".to_owned()
    } else {
        "Slot: empty".to_owned()
    });
    rows.push(match kept {
        Some(item) => format!("Kept: {item}"),
        None => "Kept: nothing".to_owned(),
    });
    rows
}

/// A click on the settings page: the first row toggles the mod, the rest are
/// read-only. Returns the new value when the click was the toggle's.
pub fn page_click(i: usize, on: bool) -> Option<bool> {
    (i == 0).then_some(!on)
}

/// What to store for the toggle's new value.
pub fn stored_enabled(on: bool) -> (&'static str, String) {
    (
        SETTING_ENABLED,
        if on { "true" } else { "false" }.to_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setting<'a>(key: &'a str, value: &'a str) -> impl Fn(&str) -> Option<String> + 'a {
        move |k| (k == key).then(|| value.to_owned())
    }

    fn inv<'a>(items: &[Option<&'a str>]) -> Vec<Option<&'a str>> {
        let mut v = vec![None; MAX_SLOT];
        for (i, item) in items.iter().enumerate() {
            v[i] = *item;
        }
        v
    }

    const PLATE: &str = "minecraft:iron_chestplate";

    /// Installed and never touched: the mod is on. Only an explicit `false`
    /// turns it off, and what the click writes reads back as that state.
    #[test]
    fn installed_means_on_until_declined() {
        assert!(setting_on(|_| None), "the default must be on");
        assert!(setting_on(setting(SETTING_ENABLED, " true \n")));
        assert!(setting_on(setting(SETTING_ENABLED, "on")));
        assert!(!setting_on(setting(SETTING_ENABLED, "false")));
        for on in [true, false] {
            let (key, value) = stored_enabled(on);
            assert_eq!(key, SETTING_ENABLED);
            assert_eq!(
                setting_on(setting(key, &value)),
                on,
                "{value} reads back {on}"
            );
        }
    }

    /// The equipped and stored files round-trip: what the helpers write, the
    /// gates read back; junk and out-of-range slots keep nothing.
    #[test]
    fn the_state_files_round_trip() {
        assert!(!equipped_of(|_| None));
        assert!(equipped_of(setting(SETTING_EQUIPPED, "true")));
        assert!(
            equipped_of(setting(SETTING_EQUIPPED, " true\n")),
            "trimmed like settings are"
        );
        assert!(!equipped_of(setting(SETTING_EQUIPPED, "false")));
        assert!(!equipped_of(setting(SETTING_EQUIPPED, "")));
        for slot in [0, 8, 9, 35] {
            assert_eq!(
                stored_of(setting(SETTING_STORED, &slot.to_string())),
                Some(slot)
            );
        }
        assert_eq!(stored_of(setting(SETTING_STORED, "36")), None);
        assert_eq!(stored_of(setting(SETTING_STORED, "-1")), None);
        assert_eq!(stored_of(setting(SETTING_STORED, "junk")), None);
        assert_eq!(stored_of(setting(SETTING_STORED, "")), None);
        assert_eq!(stored_of(|_| None), None);
        for kept in [None, Some(0), Some(35)] {
            assert_eq!(stored_of(setting(SETTING_STORED, &stored_text(kept))), kept);
        }
        assert_eq!(stored_text(None), "");
    }

    /// The mod's slot answers only on the plain survival inventory: every
    /// window that hosts the inventory screen — chests, stations, mounts,
    /// the crafting table, the creative tab — is not it.
    #[test]
    fn the_slot_lives_on_the_plain_inventory() {
        assert!(plain_inventory(true, true, true, false, false));
        assert!(
            !plain_inventory(false, true, true, false, false),
            "a chest is open"
        );
        assert!(
            !plain_inventory(true, false, true, false, false),
            "a station is open"
        );
        assert!(
            !plain_inventory(true, true, false, false, false),
            "a crafting table"
        );
        assert!(
            !plain_inventory(true, true, true, true, false),
            "a mount menu"
        );
        assert!(
            !plain_inventory(true, true, true, false, true),
            "the creative tab"
        );
    }

    /// The slot's hit box is its 18-pixel square at the panel-relative spot;
    /// a pixel off on any side misses.
    #[test]
    fn the_hit_box_is_the_slot_square() {
        let origin = (100.0, 50.0);
        let (x, y) = (origin.0 + SLOT_X, origin.1 + SLOT_Y);
        assert!(over_slot((x, y), origin));
        assert!(over_slot((x + 17.5, y + 17.5), origin));
        assert!(!over_slot((x - 0.5, y), origin), "a pixel left misses");
        assert!(!over_slot((x, y - 0.5), origin), "a pixel above misses");
        assert!(
            !over_slot((x + SLOT_SIZE, y), origin),
            "the right edge misses"
        );
        assert!(
            !over_slot((x, y + SLOT_SIZE), origin),
            "the bottom edge misses"
        );
        assert!(
            !over_slot((origin.0 + 104.0, origin.1 + 62.0), origin),
            "the book button"
        );
        assert!(
            !over_slot((origin.0 + 85.0, origin.1 + 62.0), origin),
            "the offhand slot"
        );
    }

    /// The elytra is the one item the slot equips, found main-inventory
    /// first; the free slot is hotbar first.
    #[test]
    fn the_scans_walk_the_inventory_in_vanilla_order() {
        assert!(is_elytra(Some(ELYTRA)));
        assert!(!is_elytra(Some(PLATE)));
        assert!(!is_elytra(None));
        let mut items = inv(&[]);
        items[20] = Some(ELYTRA);
        items[3] = Some(ELYTRA);
        assert_eq!(find_elytra(&items), Some(20), "main inventory first");
        let mut hotbar_only = inv(&[]);
        hotbar_only[3] = Some(ELYTRA);
        assert_eq!(find_elytra(&hotbar_only), Some(3));
        assert_eq!(find_elytra(&inv(&[])), None, "no elytra, no equip");
        let mut free = inv(&[Some(PLATE)]);
        free[0] = None;
        free[10] = None;
        assert_eq!(find_free(&free), Some(0), "hotbar first");
        let mut main_only = inv(&[Some(PLATE); 9]);
        main_only[10] = None;
        assert_eq!(
            find_free(&main_only),
            Some(9),
            "a full hotbar frees the main inventory's first"
        );
        assert_eq!(
            find_free(&inv(&[Some(PLATE); 36])),
            None,
            "a full inventory"
        );
    }

    /// A click equips from the cursor first, then the worn elytra comes off,
    /// then the inventory's first elytra goes on — and with none of those the
    /// click is nothing.
    #[test]
    fn the_click_picks_the_action_by_what_it_reaches() {
        assert_eq!(
            click_action(true, false, Some(5)),
            Some(Action::Equip(None))
        );
        assert_eq!(
            click_action(true, true, None),
            Some(Action::Equip(None)),
            "a swap"
        );
        assert_eq!(click_action(false, true, Some(5)), Some(Action::Unequip));
        assert_eq!(
            click_action(false, false, Some(5)),
            Some(Action::Equip(Some(5)))
        );
        assert_eq!(click_action(false, false, None), None, "no elytra anywhere");
    }

    /// The equip's clicks: take the elytra from its slot unless it is already
    /// on the cursor, put it on the chest slot, and a displaced chestpiece
    /// goes to the free slot. A chest slot that held another elytra leaves
    /// that elytra on the cursor instead.
    #[test]
    fn the_equip_plan_moves_three_stacks_at_most() {
        assert_eq!(
            plan_equip(Some(12), false, Some(0)),
            vec![Some(12), Some(CHEST_SLOT)],
            "an empty chest keeps nothing"
        );
        assert_eq!(
            plan_equip(Some(12), true, Some(0)),
            vec![Some(12), Some(CHEST_SLOT), Some(0)],
            "the chestpiece waits at the free slot"
        );
        assert_eq!(
            plan_equip(Some(12), true, None),
            vec![Some(12), Some(CHEST_SLOT)],
            "no room: the chestpiece lands on the cursor, no deposit click"
        );
        assert_eq!(
            plan_equip(None, true, Some(0)),
            vec![Some(CHEST_SLOT), Some(0)],
            "the elytra is already carried"
        );
    }

    /// The unequip's clicks: the elytra off, away when there is room, and the
    /// kept chestpiece worn again — by plain clicks when the elytra found a
    /// slot, by the swap through its own slot when the inventory was full.
    #[test]
    fn the_unequip_plan_always_ends_bare_handed() {
        assert_eq!(
            plan_unequip(Some(4), Some(20)),
            vec![Some(CHEST_SLOT), Some(4), Some(20), Some(CHEST_SLOT)],
        );
        assert_eq!(
            plan_unequip(None, Some(20)),
            vec![Some(CHEST_SLOT), Some(20), Some(CHEST_SLOT)],
            "the swap parks the elytra where the chestpiece waited"
        );
        assert_eq!(
            plan_unequip(Some(4), None),
            vec![Some(CHEST_SLOT), Some(4)],
            "nothing was kept"
        );
        assert_eq!(plan_unequip(None, None), vec![Some(CHEST_SLOT)]);
        assert_eq!(plan_restore(20), vec![Some(20), Some(CHEST_SLOT)]);
    }

    /// The slot lets go whenever the chest slot stops holding the elytra:
    /// death, a vanilla slot click, another mod. While the elytra is worn as
    /// tracked, nothing changes.
    #[test]
    fn the_slot_disarms_when_the_elytra_leaves() {
        assert!(!disarms(false, None));
        assert!(!disarms(true, Some(ELYTRA)));
        assert!(disarms(true, None), "the elytra is gone");
        assert!(disarms(true, Some(PLATE)), "wearing something else");
    }

    /// Only a chestpiece the engine wears in the chest slot is the kept one —
    /// and never an elytra, which would fight the unequip that parked it.
    #[test]
    fn the_kept_chestpiece_is_never_an_elytra() {
        assert!(restore_worthy(true, Some(PLATE)));
        assert!(!restore_worthy(true, Some(ELYTRA)));
        assert!(!restore_worthy(false, Some(PLATE)), "not a chest item");
        assert!(!restore_worthy(true, None), "the slot is empty now");
        assert!(!restore_worthy(false, None));
    }

    /// The kept chestpiece comes back when the chest slot is free and a menu
    /// click can run; on a server it waits for the inventory to open. The
    /// tracking clears when the slot lost the chestpiece, and when a disarm
    /// found the player wearing something else themselves.
    #[test]
    fn the_kept_chestpiece_waits_for_a_free_chest_and_a_menu() {
        let worthy = true;
        assert_eq!(
            kept_step(None, Some(20), worthy, true, false),
            Kept::Wear(20)
        );
        assert_eq!(
            kept_step(None, Some(20), worthy, false, false),
            Kept::Keep,
            "a server: the clicks wait for the inventory screen"
        );
        assert_eq!(
            kept_step(Some(ELYTRA), Some(20), worthy, true, false),
            Kept::Keep,
            "the chest slot is busy again"
        );
        assert_eq!(
            kept_step(None, Some(20), false, true, false),
            Kept::Clear,
            "the kept slot no longer holds the chestpiece"
        );
        assert_eq!(kept_step(None, None, worthy, true, false), Kept::Keep);
        assert_eq!(
            kept_step(Some(PLATE), Some(20), worthy, true, true),
            Kept::Clear,
            "disarmed over a self-worn chestpiece"
        );
        assert_eq!(
            kept_step(None, Some(20), worthy, true, true),
            Kept::Wear(20),
            "disarmed with the chest free: back on at once"
        );
        assert_eq!(
            kept_step(None, Some(20), worthy, false, true),
            Kept::Keep,
            "disarmed on a server with no menu: wait"
        );
        assert_eq!(kept_step(None, None, worthy, true, true), Kept::Keep);
    }

    /// After an equip, the kept tracking follows what the chest slot gave up:
    /// a chestpiece goes to the free slot it was parked in, while a swapped
    /// elytra or an empty chest keeps the tracking the equip inherited.
    #[test]
    fn the_equips_tracking_follows_the_displaced_stack() {
        let old = Some(20);
        assert_eq!(
            stored_after_equip(old, Displaced::Chestplate, Some(4)),
            Some(4)
        );
        assert_eq!(
            stored_after_equip(old, Displaced::Chestplate, None),
            None,
            "no room: the chestpiece rides on the cursor, untracked"
        );
        assert_eq!(
            stored_after_equip(old, Displaced::Elytra, Some(4)),
            old,
            "an elytra came off: the chestpiece stays kept"
        );
        assert_eq!(stored_after_equip(old, Displaced::Empty, Some(4)), old);
        assert_eq!(
            stored_after_equip(None, Displaced::Chestplate, Some(4)),
            Some(4)
        );
        assert_eq!(stored_after_equip(None, Displaced::Empty, None), None);
    }

    /// The settings page shows the three rows in the state the files hold,
    /// and a click on the first flips the mod; the other rows are read-only.
    #[test]
    fn the_page_shows_the_state_and_flips_on_the_first_row() {
        assert_eq!(
            labels(true, true, Some(PLATE)),
            vec![
                "Enabled: ON".to_owned(),
                "Slot: elytra worn".to_owned(),
                "Kept: minecraft:iron_chestplate".to_owned(),
            ]
        );
        assert_eq!(
            labels(false, false, None),
            vec![
                "Enabled: OFF".to_owned(),
                "Slot: empty".to_owned(),
                "Kept: nothing".to_owned(),
            ]
        );
        assert_eq!(page_click(0, false), Some(true));
        assert_eq!(page_click(0, true), Some(false));
        assert_eq!(page_click(1, false), None, "read-only rows");
        assert_eq!(page_click(2, false), None);
        assert_eq!(page_click(7, true), None);
        let (key, value) = stored_enabled(true);
        assert_eq!((key, value.as_str()), (SETTING_ENABLED, "true"));
    }
}
