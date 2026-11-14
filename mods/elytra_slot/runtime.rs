// Elytra Slot's brain: pure standard library, no game types, so it compiles
// and tests wherever — as this crate's library and, injected, as a module of
// the game. Every decision the helpers act on lives here, and every one of
// those decisions is tested below.
//
// The model (0.2.0): the mod's slot is its own bay beside the armor row, and
// the elytra it holds is REAL state, not a chest-slot swap — the chestplate
// stays worn the whole time, armor and all. The held stack lives as plain
// text (item, count, damage, enchantments, repair cost) in the mod's setting
// files, so it survives restarts and deaths; the helpers turn that text back
// into the game's own ItemStack for drawing, the cursor and the glide check.
// One honest limitation: the glide check is a client answer, so the slot
// flies in single player and on LAN (where the client hosts the world) and
// stays display-only on a remote server. And the elytra in this bay does not
// take glide wear — the vanilla wear picks a worn piece out of the armor.
//
// Clean-room: written for Horizon from the game's own public behavior; it
// copies nothing from any mod.
//
// Copyright (c) 2026 hakergeniusz. MIT.

/// The mod's registry id: the settings folder and the Options > Mods row.
pub const ID: &str = "elytra_slot";
/// The version the row shows.
pub const VERSION: &str = "0.2.0";
/// The mod's description for its row.
pub const DESCRIPTION: &str = "The survival inventory gains an elytra slot beside the armor \
row: the elytra it holds is its own bay, so your chestplate stays worn beside it. Click to \
put an elytra in or take it out; it survives deaths and restarts. Gliding works from the \
slot in single player and on LAN.";

/// The elytra: the only item the mod's slot takes.
pub const ELYTRA: &str = "minecraft:elytra";

/// The mod slot's place in the plain survival inventory, in panel-relative
/// GUI pixels: right of the player model (`ox + 26 .. ox + 75`), above the
/// offhand slot at (77, 62), clear of the recipe book's button at (104, 61).
pub const SLOT_X: f32 = 77.0;
pub const SLOT_Y: f32 = 8.0;
/// An inventory slot is 18 GUI pixels wide (16 plus the frame).
pub const SLOT_SIZE: f32 = 18.0;

/// The mod's setting files (one file per key under the mod's data dir),
/// which every gate re-reads: a change lands within the tick that follows it.
pub const SETTING_ENABLED: &str = "enabled";
/// The held stack, one file per part.
pub const SLOT_ITEM: &str = "slot.item";
pub const SLOT_COUNT: &str = "slot.count";
pub const SLOT_DAMAGE: &str = "slot.damage";
pub const SLOT_ENCH: &str = "slot.ench";
pub const SLOT_REPAIR: &str = "slot.repair";

/// The stack the bay holds, as the plain text the setting files keep. An
/// elytra carries a damage bar, enchantments and a repair cost; the custom
/// name (a styled component) is the one part the files do not keep.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SlotStack {
    pub item: String,
    pub count: u32,
    pub damage: u32,
    /// Enchantment id and level, in the stack's own order.
    pub ench: Vec<(String, u32)>,
    pub repair: u32,
}

impl SlotStack {
    /// Whether the bay's stack is the elytra (it always is, but the files
    /// are user-editable text, so the gates ask).
    pub fn is_elytra(&self) -> bool {
        self.item == ELYTRA
    }
}

/// The enchantments as their file text: `id:level;id:level`.
pub fn ench_text(ench: &[(String, u32)]) -> String {
    ench.iter()
        .map(|(id, level)| format!("{id}:{level}"))
        .collect::<Vec<_>>()
        .join(";")
}

/// The file text back into enchantments; broken parts are skipped.
pub fn ench_of(text: &str) -> Vec<(String, u32)> {
    text.split(';')
        .filter(|part| !part.trim().is_empty())
        .filter_map(|part| {
            // From the right: ids carry colons of their own
            // ("minecraft:unbreaking"), so the level is the LAST colon.
            let (id, level) = part.trim().rsplit_once(':')?;
            let level = level.trim().parse::<u32>().ok()?;
            // Real entries are namespaced ids with a level; junk is dropped.
            let id = id.trim();
            if level == 0 || !id.contains(':') {
                return None;
            }
            Some((id.to_owned(), level))
        })
        .collect()
}

/// The bay's stack as the files hold it: a `slot.item` file that names the
/// elytra. Empty, missing or wrong-id files hold nothing.
pub fn stack_of(get: impl Fn(&str) -> Option<String>) -> Option<SlotStack> {
    let item = get(SLOT_ITEM)?.trim().to_owned();
    if item != ELYTRA {
        return None;
    }
    let count = get(SLOT_COUNT)
        .and_then(|v| v.trim().parse::<u32>().ok())
        .unwrap_or(1)
        .max(1);
    let damage = get(SLOT_DAMAGE)
        .and_then(|v| v.trim().parse::<u32>().ok())
        .unwrap_or(0);
    let ench = ench_of(&get(SLOT_ENCH).unwrap_or_default());
    let repair = get(SLOT_REPAIR)
        .and_then(|v| v.trim().parse::<u32>().ok())
        .unwrap_or(0);
    Some(SlotStack {
        item,
        count,
        damage,
        ench,
        repair,
    })
}

/// The files a held stack writes (empty item text clears the bay).
pub fn texts(stack: Option<&SlotStack>) -> Vec<(&'static str, String)> {
    let Some(s) = stack else {
        return vec![(SLOT_ITEM, String::new())];
    };
    vec![
        (SLOT_ITEM, s.item.clone()),
        (SLOT_COUNT, s.count.to_string()),
        (SLOT_DAMAGE, s.damage.to_string()),
        (SLOT_ENCH, ench_text(&s.ench)),
        (SLOT_REPAIR, s.repair.to_string()),
    ]
}

/// Whether the mod is on. A missing file means ON — the mod exists for the
/// slot, so installing it is asking for it; anything but `false` in the file
/// is on, trimmed like the game reads its settings.
pub fn setting_on(get: impl FnOnce(&str) -> Option<String>) -> bool {
    !matches!(get(SETTING_ENABLED).as_deref(), Some(v) if v.trim() == "false")
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

use std::sync::Mutex;

// The live bay: one Mutex-guarded Option, so the helpers never block each
// other beyond a copy. The files are the durable copy; this is the tick's.
static SLOT: Mutex<Option<SlotStack>> = Mutex::new(None);

/// The bay's stack as of now.
pub fn held() -> Option<SlotStack> {
    SLOT.lock().ok().and_then(|slot| slot.clone())
}

/// The bay takes or gives a stack.
pub fn set(stack: Option<SlotStack>) {
    if let Ok(mut slot) = SLOT.lock() {
        *slot = stack;
    }
}

/// What a click on the mod's slot does.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    /// The cursor's elytra goes in; the bay's old stack (if any) comes to
    /// the cursor, so no item is ever lost.
    Store,
    /// The bay's stack comes to the empty cursor.
    Take,
}

/// The action a click answers with: the cursor's elytra stores, an empty
/// cursor over a held elytra takes it, anything else is not the mod's.
pub fn click_action(cursor_item: Option<&str>, bay: Option<&SlotStack>) -> Option<Action> {
    match (cursor_item, bay) {
        (Some(ELYTRA), _) => Some(Action::Store),
        (None, Some(s)) if s.is_elytra() => Some(Action::Take),
        _ => None,
    }
}

/// The settings page's rows: the toggle and the bay's state.
pub fn page_labels(on: bool, bay: bool) -> Vec<String> {
    vec![
        format!("Enabled: {}", if on { "ON" } else { "OFF" }),
        if bay {
            "Slot: holding the elytra".to_owned()
        } else {
            "Slot: empty".to_owned()
        },
    ]
}

/// What a click on the page's row `i` stores: the first row toggles the mod,
/// the second is read-only. Returns the new value when the click was real.
pub fn page_click(i: usize, on: bool) -> Option<bool> {
    (i == 0).then_some(!on)
}

#[cfg(test)]
mod tests {
    use super::{
        click_action, ench_of, ench_text, held, over_slot, page_click, page_labels,
        set, setting_on, stack_of, texts, Action, SlotStack, ELYTRA, SLOT_ENCH, SLOT_ITEM,
        SLOT_X,
    };

    fn get(key: &str) -> Option<String> {
        match key {
            SLOT_ITEM => Some(ELYTRA.to_owned()),
            super::SLOT_COUNT => Some("1".into()),
            super::SLOT_DAMAGE => Some("31".into()),
            SLOT_ENCH => Some("minecraft:unbreaking:3;minecraft:mending:1".into()),
            super::SLOT_REPAIR => Some("2".into()),
            _ => None,
        }
    }

    #[test]
    fn the_bay_round_trips_through_the_files() {
        let stack = stack_of(get).expect("the files hold an elytra");
        assert!(stack.is_elytra());
        assert_eq!(stack.count, 1);
        assert_eq!(stack.damage, 31);
        assert_eq!(stack.repair, 2);
        assert_eq!(
            stack.ench,
            vec![
                ("minecraft:unbreaking".to_owned(), 3),
                ("minecraft:mending".to_owned(), 1)
            ]
        );
        let files = texts(Some(&stack));
        let read = |key: &str| {
            files
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.clone())
        };
        assert_eq!(stack_of(read), Some(stack), "files write back the same stack");
        let empty = stack_of(|_| None);
        assert_eq!(empty, None);
        let cleared = texts(None);
        assert_eq!(cleared.len(), 1);
        assert_eq!(cleared[0].0, SLOT_ITEM);
        assert!(cleared[0].1.is_empty(), "an empty item text clears the bay");
        assert_eq!(stack_of(|k| (k == SLOT_ITEM).then(|| "minecraft:elytra".into())).map(|s| s.damage), Some(0), "missing parts default");
        assert_eq!(stack_of(|k| (k == SLOT_ITEM).then(|| "minecraft:bow".into())), None, "a wrong item holds nothing");
    }

    #[test]
    fn the_enchant_text_round_trips_and_tolerates_junk() {
        let ench = vec![("minecraft:mending".to_owned(), 1)];
        assert_eq!(ench_of(&ench_text(&ench)), ench);
        assert_eq!(ench_of(&ench_text(&[])), Vec::<(String, u32)>::new());
        assert_eq!(ench_of(";;junk;minecraft:aqua_affinity:1;x:0"), vec![
            ("minecraft:aqua_affinity".to_owned(), 1)
        ]);
    }

    #[test]
    fn the_bay_holds_one_stack() {
        set(None);
        assert_eq!(held(), None);
        set(Some(SlotStack {
            item: ELYTRA.to_owned(),
            count: 1,
            damage: 0,
            ench: Vec::new(),
            repair: 0,
        }));
        assert!(held().is_some());
        set(None);
        assert_eq!(held(), None);
    }

    #[test]
    fn a_click_answers_what_the_cursor_and_bay_hold() {
        let bay = Some(SlotStack {
            item: ELYTRA.to_owned(),
            count: 1,
            damage: 0,
            ench: Vec::new(),
            repair: 0,
        });
        assert_eq!(click_action(Some(ELYTRA), None), Some(Action::Store));
        assert_eq!(
            click_action(Some(ELYTRA), bay.as_ref()),
            Some(Action::Store),
            "an elytra on the cursor swaps the bay's own out to the cursor"
        );
        assert_eq!(click_action(None, bay.as_ref()), Some(Action::Take));
        assert_eq!(click_action(None, None), None, "nothing anywhere");
        assert_eq!(click_action(Some("minecraft:diamond_chestplate"), None), None, "the slot takes elytra only");
    }

    #[test]
    fn the_slot_answers_only_over_itself() {
        assert!(over_slot((77.0, 8.0), (0.0, 0.0)));
        assert!(over_slot((94.9, 25.9), (0.0, 0.0)));
        assert!(!over_slot((95.1, 8.0), (0.0, 0.0)));
        assert!(!over_slot((77.0, 26.1), (0.0, 0.0)));
        assert!(over_slot((87.0, 18.0), (10.0, 10.0)), "the origin moves it");
    }

    #[test]
    fn the_page_shows_the_state_and_toggles() {
        assert_eq!(
            page_labels(true, false),
            vec!["Enabled: ON".to_owned(), "Slot: empty".to_owned()]
        );
        assert_eq!(
            page_labels(false, true)[1],
            "Slot: holding the elytra".to_owned()
        );
        assert_eq!(page_click(0, true), Some(false));
        assert_eq!(page_click(1, true), None, "the bay row is read-only");
        assert!(setting_on(|_| None), "installed means on");
        assert!(!setting_on(|k| (k == super::SETTING_ENABLED).then(|| "false".into())));
        assert_eq!(SLOT_X, 77.0, "the slot sits right of the player model");
    }
}
