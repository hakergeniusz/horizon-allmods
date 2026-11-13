// Bundle Inventory's brain: pure standard library, no game types, so it
// compiles and tests wherever — as this crate's library and, injected, as a
// module of the game. Every decision the helpers act on lives here, and every
// one of those decisions is tested below.
//
// The model: a bundle keeps its contents in the item stack's own data
// (`minecraft:bundle_contents`, a list of item stacks), and the engine's
// verified menu engine already runs vanilla's bundle click overrides on it —
// right-click takes the selected stack out, a carried stack clicked onto the
// bundle goes in as far as its weight allows, and the wheel selects which
// entry the next take removes. What vanilla has no room for is reaching into
// the bundle directly, so the mod draws an interactive panel beside the
// survival inventory: every entry as its own slot, click to take it onto the
// cursor, shift-click to move it into the inventory, Q over it to drop it,
// and any inventory click with the panel open to insert that stack.
//
// Nothing here invents state: every plan is a sequence of the engine's own
// menu clicks (and, on a server, the bundle-selection packet vanilla sends
// for the wheel), so the server sees ordinary play and the panel is honest
// on single player, LAN and servers alike. Each step says what cursor state
// it expects — a plan stops at the first step whose expectation fails, which
// is what makes the greedy transfer chain safe: a deposit that swallowed the
// whole carried stack ends the chain, and what was never carried is never
// clicked onto a stack that would pick it up.
//
// Clean-room: written for Horizon from the game's own public behavior and
// the original's feature description; it copies nothing from any mod.
//
// Copyright (c) 2026 hakergeniusz. MIT.

use std::cell::{Cell, RefCell};

/// The mod's registry id: the settings folder and the Options > Mods row.
pub const ID: &str = "bundle_inventory";
/// The version the row shows.
pub const VERSION: &str = "0.1.0";
/// The mod's description for its row.
pub const DESCRIPTION: &str = "Shift + right-click a bundle in your inventory to open an \
interactive panel of its contents beside the inventory: click an entry to take it, \
shift-click to move it into your inventory, click any item with the panel open to put it \
in, and Q over an entry to drop it. Works in single player, on LAN and on servers, \
because every move is an ordinary inventory click.";

/// The mod's setting files (one file per key under the mod's data dir),
/// which every gate re-reads: a change lands within the tick that follows it.
pub const SETTING_ENABLED: &str = "enabled";

// ------------------------------------------------------------- the geometry

/// The panel grid: four entries across, like the vanilla bundle tooltip.
pub const COLS: usize = 4;
/// The most entries the panel shows; the rest stay reachable through the
/// engine's own wheel selection on the bundle, or by taking some out first.
pub const MAX_SHOWN: usize = 36;
/// An entry cell is 18 GUI pixels (16 plus the frame), the inventory's pitch.
pub const ENTRY: f32 = 18.0;
/// The panel's inner padding, in GUI pixels.
pub const PAD: f32 = 6.0;
/// The gap between the grid and the weight bar.
pub const GAP: f32 = 5.0;
/// The weight bar's height.
pub const BAR_H: f32 = 6.0;
/// The weight bar's width: the grid's four columns.
pub const BAR_W: f32 = COLS as f32 * ENTRY;
/// The panel border's thickness.
pub const BORDER: f32 = 1.0;
/// The survival inventory's width: the panel sits a gap left of it.
pub const INVENTORY_W: f32 = 176.0;
/// The gap between the inventory's edge and the panel.
pub const PANEL_GAP: f32 = 8.0;
/// The panel's top edge, relative to the inventory origin.
pub const PANEL_TOP: f32 = 8.0;

/// The panel's width: the grid plus padding and a border.
pub fn panel_width() -> f32 {
    BORDER + PAD + BAR_W + PAD + BORDER
}

/// How many entries the panel shows, and how many it folds away.
pub fn shown_count(total: usize) -> (usize, usize) {
    let shown = total.min(MAX_SHOWN);
    (shown, total - shown)
}

/// The grid's row count for `shown` entries; an empty bundle keeps one row so
/// the panel never collapses to nothing.
pub fn grid_rows(shown: usize) -> usize {
    shown.div_ceil(COLS).max(1)
}

/// The panel's height: border, padding, the grid, the gap and the bar.
pub fn panel_height(shown: usize) -> f32 {
    BORDER + PAD + grid_rows(shown) as f32 * ENTRY + GAP + BAR_H + PAD + BORDER
}

/// The panel's rectangle for an inventory origin and the window width: a gap
/// left of the survival inventory, or a gap right of it when the left would
/// run off the window.
pub fn panel_rect(origin: [f32; 2], window_w: f32) -> [f32; 4] {
    let width = panel_width();
    let left = origin[0] - PANEL_GAP - width;
    let x = if left < BORDER {
        (origin[0] + INVENTORY_W + PANEL_GAP).min((window_w - width - BORDER).max(BORDER))
    } else {
        left
    };
    // The rectangle is drawn from the full grid's height, so the panel keeps
    // one size while it is open and the hit tests match what was drawn.
    [x, origin[1] + PANEL_TOP, width, panel_height(MAX_SHOWN)]
}

/// Entry `index`'s cell, in drawing order (row-major, the first entry
/// top-left — the first is the bundle's next-out, as the tooltip says).
pub fn entry_rect(panel: [f32; 4], index: usize) -> [f32; 4] {
    let (row, col) = (index / COLS, index % COLS);
    [
        panel[0] + BORDER + PAD + col as f32 * ENTRY,
        panel[1] + BORDER + PAD + row as f32 * ENTRY,
        ENTRY,
        ENTRY,
    ]
}

/// The weight bar's rectangle under the grid.
pub fn bar_rect(panel: [f32; 4], shown: usize) -> [f32; 4] {
    [
        panel[0] + BORDER + PAD,
        panel[1] + BORDER + PAD + grid_rows(shown) as f32 * ENTRY + GAP,
        BAR_W,
        BAR_H,
    ]
}

/// Whether `at` is inside `rect`.
pub fn in_rect(at: [f32; 2], rect: [f32; 4]) -> bool {
    at[0] >= rect[0] && at[0] < rect[0] + rect[2] && at[1] >= rect[1] && at[1] < rect[1] + rect[3]
}

/// The entry under the cursor, or None on the panel's chrome (padding,
/// border, bar): the chrome swallows clicks so they never fall through to
/// whatever is under the panel.
pub fn entry_at(at: [f32; 2], panel: [f32; 4], total: usize) -> Option<usize> {
    let (shown, _) = shown_count(total);
    (0..shown).find(|&i| in_rect(at, entry_rect(panel, i)))
}

// ---------------------------------------------------------------- the model

/// An item stack as the plans see it: the registry id and the count. Bundles
/// care about the item kind and how many, not the components — the engine's
/// own click keeps the stack whole, and its weight math runs there too.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotItem {
    pub id: String,
    pub count: u32,
}

/// What a plan is built from: a snapshot of the open bundle and the player
/// inventory, read once and passed to [`plan`].
#[derive(Clone, Debug, Default)]
pub struct View {
    /// The game slot the open bundle sits in (0..36 the player inventory,
    /// 504 the off hand).
    pub bundle_slot: usize,
    /// The bundle's `selectedItem` index, -1 for none.
    pub selected: i32,
    /// The bundle's entries, in the engine's display order (the first is the
    /// next out).
    pub entries: Vec<(String, u32)>,
    /// The player's inventory slots the plans may touch, in game-slot order
    /// (0..36 the main inventory, then the off hand).
    pub slots: Vec<(usize, Option<SlotItem>)>,
}

impl View {
    /// The entry `index`'s item id, when it exists.
    pub fn entry_id(&self, index: usize) -> Option<&str> {
        self.entries.get(index).map(|(id, _)| id.as_str())
    }

    /// The stack a game slot holds, as the snapshot sees it.
    pub fn slot_item(&self, slot: usize) -> Option<&SlotItem> {
        self.slots
            .iter()
            .find(|(s, _)| *s == slot)
            .and_then(|(_, item)| item.as_ref())
    }
}

/// What a click on the mod's surface asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Take the entry onto the cursor.
    Extract(usize),
    /// Take the entry and put it into the inventory (the panel's shift-click).
    TransferOut(usize),
    /// Put the stack in a game slot into the open bundle.
    Insert(usize),
    /// Drop the entry into the world (Q over it).
    Drop(usize),
}

/// One engine click of a plan. Every variant is an ordinary menu click (or,
/// for [`Step::Select`], the bundle-selection packet vanilla sends for the
/// wheel); the tick that runs the plan checks each step's cursor expectation
/// against the live game and stops the plan on the first mismatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Select the entry the rest of the plan acts on. The executor re-checks
    /// the live selection: vanilla's `toggleSelectedItem` toggles, so sending
    /// a select for what is selected already would clear it.
    Select(i32),
    /// Right-click the bundle's slot with an empty cursor: the selected entry
    /// comes out onto the cursor (the menu's bundle click override). Stops
    /// the plan when the cursor already carries something.
    Take { bundle: usize },
    /// Left-click a slot with whatever the cursor carries: a deposit, or,
    /// with `slot: None`, the click outside the menu that drops the carried
    /// stack. Stops the plan when the cursor is empty, unless `allow_empty`
    /// — which only the put-back onto the slot a plan emptied itself uses,
    /// where an empty cursor makes the click an honest no-op.
    Give {
        slot: Option<usize>,
        allow_empty: bool,
    },
    /// Left-click a slot with an empty cursor: the insert's carry. Stops the
    /// plan when the cursor already carries something.
    Carry { slot: usize },
}

/// Vanilla's `BundleContents.canItemBeInBundle`: shulker boxes do not fit.
/// The engine re-checks on the real click; this only spares the player the
/// fail sound and a pointless click sequence.
pub fn can_insert(id: &str) -> bool {
    !id.ends_with("shulker_box")
}

/// Whether a game slot is one of the player's own: the main inventory or the
/// off hand. The panel opens on these and the insert takes from these.
pub fn player_slot(slot: usize) -> bool {
    slot < 36 || slot == 504
}

/// Whether the selection step is needed: vanilla's `toggleSelectedItem`
/// toggles, so selecting what is selected already would clear it and the
/// take would remove the first entry instead.
pub fn select_needed(selected: i32, index: usize) -> bool {
    selected != index as i32
}

/// Where a stack of `id` taken out of the bundle should go, best first:
/// stacks of the same kind it can merge onto (the main rows, then the hotbar,
/// never the off hand nor the bundle's own slot), then empty slots, same
/// order. The plan clicks them in turn and what still does not fit returns
/// to the bundle, so a wrong guess costs nothing.
pub fn deposit_targets(view: &View, id: &str) -> Vec<usize> {
    let main = |slot: usize| (9..36).contains(&slot);
    let candidates = |keep: &dyn Fn(Option<&SlotItem>) -> bool| {
        let mut hotbar = Vec::new();
        let mut slots = Vec::new();
        for &(slot, ref item) in &view.slots {
            if slot == view.bundle_slot || slot >= 504 || !player_slot(slot) {
                continue;
            }
            if !keep(item.as_ref()) {
                continue;
            }
            if main(slot) {
                slots.push(slot);
            } else {
                hotbar.push(slot);
            }
        }
        slots.extend(hotbar);
        slots
    };
    let mut targets = candidates(&|item| item.is_some_and(|s| s.id == id && s.count < 64));
    targets.extend(candidates(&|item| item.is_none()));
    targets.truncate(4);
    targets
}

/// The steps for an action against a snapshot. Empty when there is nothing
/// honest to do: a click on an entry that is not there, or an insert whose
/// slot holds nothing insertable. Every step is an ordinary menu click, so a
/// plan that stops early leaves the world consistent by itself.
pub fn plan(action: Action, view: &View) -> Vec<Step> {
    /// The take every bundle-side action starts from.
    fn take(view: &View, index: usize, steps: &mut Vec<Step>) {
        if select_needed(view.selected, index) {
            steps.push(Step::Select(index as i32));
        }
        steps.push(Step::Take {
            bundle: view.bundle_slot,
        });
    }
    match action {
        Action::Extract(index) => {
            if view.entry_id(index).is_none() {
                return Vec::new();
            }
            let mut steps = Vec::new();
            take(view, index, &mut steps);
            steps
        }
        Action::TransferOut(index) => {
            let Some(id) = view.entry_id(index) else {
                return Vec::new();
            };
            let id = id.to_owned();
            let targets = deposit_targets(view, &id);
            let mut steps = Vec::new();
            take(view, index, &mut steps);
            if targets.is_empty() {
                // Nothing to move into: taking it out onto the cursor is the
                // whole job (the same as a plain extract).
                return steps;
            }
            for slot in targets {
                steps.push(Step::Give {
                    slot: Some(slot),
                    allow_empty: false,
                });
            }
            // What no slot took goes back into the bundle instead of riding
            // the cursor. The deposits before it stop the plan the moment
            // the cursor empties, so this only runs while there is
            // something to put back.
            steps.push(Step::Give {
                slot: Some(view.bundle_slot),
                allow_empty: false,
            });
            steps
        }
        Action::Insert(slot) => {
            if slot == view.bundle_slot || !player_slot(slot) {
                return Vec::new();
            }
            let Some(item) = view.slot_item(slot) else {
                return Vec::new();
            };
            if !can_insert(&item.id) {
                return Vec::new();
            }
            // Carry the stack, click the bundle (what fits goes in), and put
            // the remainder back where it came from — its slot is empty now,
            // so the put-back is an honest no-op when the bundle took it all.
            vec![
                Step::Carry { slot },
                Step::Give {
                    slot: Some(view.bundle_slot),
                    allow_empty: false,
                },
                Step::Give {
                    slot: Some(slot),
                    allow_empty: true,
                },
            ]
        }
        Action::Drop(index) => {
            if view.entry_id(index).is_none() {
                return Vec::new();
            }
            let mut steps = Vec::new();
            take(view, index, &mut steps);
            steps.push(Step::Give {
                slot: None,
                allow_empty: false,
            });
            steps
        }
    }
}

// --------------------------------------------------------- the shared state

thread_local! {
    /// The open bundle's game slot, -1 for closed. The panel is modal over
    /// the plain survival inventory, one bundle at a time.
    static OPEN: Cell<i32> = const { Cell::new(-1) };
    /// The steps of the action the last click or key asked for, run by the
    /// next tick — the hooks read the game through a shared borrow, and the
    /// menu clicks want the exclusive one (hey_wiki's defer-to-tick shape).
    static DEFERRED: RefCell<Option<Vec<Step>>> = const { RefCell::new(None) };
}

/// The open bundle's game slot, if the panel is up.
pub fn open_slot() -> Option<usize> {
    usize::try_from(OPEN.with(|open| open.get())).ok()
}

/// Open the panel on a bundle's slot, or close it.
pub fn set_open(slot: Option<usize>) {
    OPEN.with(|open| open.set(slot.map_or(-1, |slot| slot as i32)));
}

/// Queue a plan for the next tick.
pub fn defer(steps: Vec<Step>) {
    DEFERRED.with(|plan| *plan.borrow_mut() = Some(steps));
}

/// Take the queued plan, if the last click or key left one.
pub fn take_deferred() -> Option<Vec<Step>> {
    DEFERRED.with(|plan| plan.borrow_mut().take())
}

// ------------------------------------------------------------- the settings

/// Whether the mod is on. A missing file means ON — the mod exists for the
/// panel, so installing it is asking for it; anything but `false` in the file
/// is on, trimmed like the game reads its settings.
pub fn setting_on(get: impl FnOnce(&str) -> Option<String>) -> bool {
    get(SETTING_ENABLED).is_none_or(|on| on.trim() != "false")
}

/// The settings page's rows: the one toggle, then nothing — the panel has no
/// state worth showing that the inventory does not already show.
pub fn labels(on: bool) -> Vec<String> {
    vec![format!("Enabled: {}", if on { "on" } else { "off" })]
}

/// What a click on the settings page's `button`th row does: row 0 flips
/// Enabled, the rest are read-only.
pub fn page_click(button: usize, on: bool) -> Option<bool> {
    (button == 0).then_some(!on)
}

/// The setting file and value for an Enabled flip.
pub fn stored_enabled(on: bool) -> (&'static str, &'static str) {
    (SETTING_ENABLED, if on { "true" } else { "false" })
}

/// The panel answers only over the plain survival inventory: no container,
/// no workstation, the 2x2 crafting grid, no mount screen, no creative tabs.
/// On a server the menu clicks go out only while the inventory is open,
/// which the modal panel guarantees anyway.
pub fn plain_inventory(
    container_none: bool,
    station_none: bool,
    craft2: bool,
    mount_inactive: bool,
    creative_closed: bool,
) -> bool {
    container_none && station_none && craft2 && mount_inactive && creative_closed
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A plain view: the bundle in slot 3, everything else empty.
    fn base() -> View {
        View {
            bundle_slot: 3,
            selected: -1,
            entries: Vec::new(),
            slots: (0..36).chain([504]).map(|s| (s, None)).collect(),
        }
    }

    /// An entry in the bundle.
    fn with_entry(mut view: View, id: &str, count: u32) -> View {
        view.entries.push((id.to_owned(), count));
        view
    }

    /// A stack in a game slot.
    fn put(mut view: View, slot: usize, item: Option<(&str, u32)>) -> View {
        for (s, at) in view.slots.iter_mut() {
            if *s == slot {
                *at = item.map(|(id, count)| SlotItem {
                    id: id.to_owned(),
                    count,
                });
            }
        }
        view
    }

    /// Full stacks of something else, so those slots are neither partial
    /// matches nor empties.
    fn filled_with_other(mut view: View, slots: &[usize]) -> View {
        for &slot in slots {
            view = put(view, slot, Some(("minecraft:sand", 64)));
        }
        view
    }

    fn click(slot: usize) -> Step {
        Step::Give {
            slot: Some(slot),
            allow_empty: false,
        }
    }

    fn take() -> Step {
        Step::Take { bundle: 3 }
    }

    // ---------------------------------------------------------- geometry

    #[test]
    fn the_panel_sits_left_of_the_inventory_and_flips_when_it_would_not_fit() {
        let width = panel_width();
        assert!((width - 86.0).abs() < 1e-4);
        // Room on the left: a gap left of the inventory.
        let panel = panel_rect([300.0, 100.0], 800.0);
        assert_eq!(panel[0], 300.0 - PANEL_GAP - width);
        assert_eq!(panel[1], 108.0);
        // Hugged by the window's left edge: a gap right of the inventory.
        let panel = panel_rect([70.0, 100.0], 800.0);
        assert_eq!(panel[0], 70.0 + INVENTORY_W + PANEL_GAP);
        // Even hugging on the right, the panel never leaves the window.
        let panel = panel_rect([70.0, 100.0], 280.0);
        assert!(panel[0] >= BORDER);
        assert!(panel[0] + width <= 280.0);
        // The full grid's height: nine rows of eighteen plus the chrome.
        let height = panel_height(MAX_SHOWN);
        assert!((height - 187.0).abs() < 1e-4);
    }

    #[test]
    fn entries_lay_out_row_major_and_the_chrome_swallows() {
        let panel = panel_rect([300.0, 100.0], 800.0);
        let first = entry_rect(panel, 0);
        let second = entry_rect(panel, 1);
        let fifth = entry_rect(panel, 4);
        assert_eq!(first[0], panel[0] + BORDER + PAD);
        assert_eq!(second[0], first[0] + ENTRY);
        assert_eq!(fifth[0], first[0]);
        assert_eq!(fifth[1], first[1] + ENTRY);
        // Every entry's center hits back its own index.
        for index in 0..MAX_SHOWN {
            let rect = entry_rect(panel, index);
            let at = [rect[0] + 9.0, rect[1] + 9.0];
            assert_eq!(entry_at(at, panel, MAX_SHOWN), Some(index));
        }
        // The bar and the padding are no entry.
        let bar = bar_rect(panel, MAX_SHOWN);
        assert_eq!(
            entry_at([bar[0] + 1.0, bar[1] + 1.0], panel, MAX_SHOWN),
            None
        );
        assert_eq!(
            entry_at([panel[0] + 2.0, panel[1] + 2.0], panel, MAX_SHOWN),
            None
        );
        // Entries past what the panel shows are not clickable here.
        assert_eq!(shown_count(40), (36, 4));
        assert_eq!(shown_count(3), (3, 0));
    }

    #[test]
    fn a_small_grid_keeps_one_row_and_the_bar_below_it() {
        let panel = panel_rect([300.0, 100.0], 800.0);
        let bar = bar_rect(panel, 2);
        assert_eq!(bar[1], panel[1] + BORDER + PAD + ENTRY + GAP);
        // An empty bundle's grid keeps a row so the bar has something above.
        let bar = bar_rect(panel, 0);
        assert_eq!(bar[1], panel[1] + BORDER + PAD + ENTRY + GAP);
        assert_eq!(grid_rows(0), 1);
        assert_eq!(grid_rows(5), 2);
    }

    // ------------------------------------------------------------ settings

    #[test]
    fn enabled_defaults_on_and_the_page_toggles_it() {
        assert!(setting_on(|_| None));
        assert!(setting_on(
            |key| (key == SETTING_ENABLED).then(|| "true".into())
        ));
        assert!(!setting_on(|_| Some("false".into())));
        // Whitespace around the file's contents trims, so `false` stays off
        // however it was written.
        assert!(!setting_on(|_| Some(" false \n".into())));
        assert!(setting_on(|_| Some("yes".into())));
        let labels = labels(true);
        assert_eq!(labels.len(), 1);
        assert!(labels[0].starts_with("Enabled: on"));
        assert_eq!(page_click(0, true), Some(false));
        assert_eq!(page_click(1, true), None);
        assert_eq!(stored_enabled(false), (SETTING_ENABLED, "false"));
        assert_eq!(stored_enabled(true), (SETTING_ENABLED, "true"));
    }

    #[test]
    fn the_panel_is_plain_inventory_only() {
        assert!(plain_inventory(true, true, true, true, true));
        assert!(!plain_inventory(false, true, true, true, true));
        assert!(!plain_inventory(true, false, true, true, true));
        assert!(!plain_inventory(true, true, false, true, true));
        assert!(!plain_inventory(true, true, true, false, true));
        assert!(!plain_inventory(true, true, true, true, false));
    }

    // -------------------------------------------------------------- plans

    #[test]
    fn an_extract_selects_then_right_clicks_the_bundle() {
        let mut open = with_entry(base(), "minecraft:stone", 32);
        open.selected = 0;
        // Already selected: no select step, or the toggle would clear it and
        // the take would remove the first entry instead.
        assert_eq!(plan(Action::Extract(0), &open), vec![take()]);
        let closed = with_entry(base(), "minecraft:stone", 32);
        assert_eq!(
            plan(Action::Extract(0), &closed),
            vec![Step::Select(0), take()]
        );
        // Nothing on an index the bundle does not have.
        assert!(plan(Action::Extract(9), &closed).is_empty());
    }

    #[test]
    fn a_transfer_out_merges_then_fills_then_returns_the_rest() {
        let open = with_entry(
            put(
                put(
                    put(base(), 10, Some(("minecraft:stone", 30))),
                    20,
                    Some(("minecraft:stone", 1)),
                ),
                12,
                Some(("minecraft:stone", 64)),
            ),
            "minecraft:stone",
            40,
        );
        // The plan: select, take, merge onto slot 10 (30/64) and slot 20
        // (1/64) — slot 12 is full and no target — then fill empty slots,
        // then put what is left back into the bundle.
        assert_eq!(
            plan(Action::TransferOut(0), &open),
            vec![
                Step::Select(0),
                take(),
                click(10),
                click(20),
                click(9),
                click(11),
                click(3),
            ]
        );
        // No target at all: the plain extract, the stack rides the cursor.
        let empty = filled_with_other(
            with_entry(base(), "minecraft:stone", 1),
            &[
                0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
            ],
        );
        assert_eq!(
            plan(Action::TransferOut(0), &empty),
            plan(Action::Extract(0), &empty)
        );
        // Nothing on an index that is not there.
        assert!(plan(Action::TransferOut(9), &empty).is_empty());
    }

    #[test]
    fn deposits_prefer_partial_stacks_then_empties_main_before_hotbar() {
        let view = filled_with_other(
            with_entry(base(), "minecraft:stone", 1),
            &[
                0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
                25, 26, 27, 28, 29, 31, 32, 33, 34, 35,
            ],
        );
        let open = put(
            put(
                put(
                    put(view, 2, Some(("minecraft:stone", 10))),
                    12,
                    Some(("minecraft:stone", 63)),
                ),
                30,
                None,
            ),
            504,
            Some(("minecraft:stone", 1)),
        );
        // The bundle sits in the off hand for this one: its own slot is no
        // deposit target either way.
        let mut open = open;
        open.bundle_slot = 504;
        let targets = deposit_targets(&open, "minecraft:stone");
        // Partials first (main rows before the hotbar), then empties, same
        // order; the off hand and the bundle's own slot are never targets.
        // Slot 20 holds sand (full, no match), 12 is a partial match, 2 is a
        // hotbar partial, 30 is the only empty slot left.
        assert_eq!(targets, vec![12, 2, 30]);
    }

    #[test]
    fn an_insert_carries_clicks_the_bundle_and_returns_the_rest() {
        let open = put(base(), 17, Some(("minecraft:dirt", 12)));
        assert_eq!(
            plan(Action::Insert(17), &open),
            vec![
                Step::Carry { slot: 17 },
                click(3),
                Step::Give {
                    slot: Some(17),
                    allow_empty: true
                },
            ]
        );
        // The bundle's own slot click stays vanilla's (take one out, or put
        // the carried stack in).
        assert!(plan(Action::Insert(3), &open).is_empty());
        // Only the player's own slots insert; a container slot does not.
        let mut container = put(base(), 300, Some(("minecraft:dirt", 1)));
        container.slots.clear();
        assert!(plan(Action::Insert(300), &container).is_empty());
        // An empty slot has nothing to insert.
        assert!(plan(Action::Insert(20), &open).is_empty());
        // Shulker boxes do not go in, sparing the fail sound.
        let shulker = put(base(), 17, Some(("minecraft:red_shulker_box", 1)));
        assert!(plan(Action::Insert(17), &shulker).is_empty());
        assert!(!can_insert("minecraft:shulker_box"));
        assert!(can_insert("minecraft:stone"));
    }

    #[test]
    fn a_drop_takes_the_entry_and_throws_the_cursor() {
        let mut open = with_entry(
            with_entry(base(), "minecraft:stone", 5),
            "minecraft:dirt",
            2,
        );
        open.selected = 1;
        assert_eq!(
            plan(Action::Drop(0), &open),
            vec![
                Step::Select(0),
                take(),
                Step::Give {
                    slot: None,
                    allow_empty: false
                },
            ]
        );
        assert!(plan(Action::Drop(2), &open).is_empty());
    }

    #[test]
    fn the_selection_step_spares_what_is_already_selected() {
        assert!(select_needed(-1, 0));
        assert!(select_needed(1, 0));
        assert!(!select_needed(0, 0));
        assert!(!select_needed(3, 3));
    }

    #[test]
    fn the_panel_opens_on_the_player_s_own_slots_only() {
        assert!(player_slot(0));
        assert!(player_slot(35));
        assert!(player_slot(504));
        assert!(!player_slot(36));
        assert!(!player_slot(500));
        assert!(!player_slot(504 + 1));
    }

    // --------------------------------------------------------------- state

    #[test]
    fn the_panel_state_opens_closes_and_queues_one_plan() {
        set_open(None);
        assert_eq!(open_slot(), None);
        set_open(Some(504));
        assert_eq!(open_slot(), Some(504));
        set_open(None);
        assert_eq!(open_slot(), None);
        assert_eq!(take_deferred(), None);
        defer(vec![Step::Select(0)]);
        defer(vec![Step::Give {
            slot: None,
            allow_empty: false,
        }]);
        assert_eq!(
            take_deferred(),
            Some(vec![Step::Give {
                slot: None,
                allow_empty: false,
            }])
        );
        assert_eq!(take_deferred(), None);
    }

    #[test]
    fn a_view_answers_its_entries_and_slots() {
        let view = put(
            with_entry(base(), "minecraft:stone", 5),
            8,
            Some(("minecraft:dirt", 3)),
        );
        assert_eq!(view.entry_id(0), Some("minecraft:stone"));
        assert_eq!(view.entry_id(1), None);
        assert_eq!(
            view.slot_item(8),
            Some(&SlotItem {
                id: "minecraft:dirt".into(),
                count: 3
            })
        );
        assert_eq!(view.slot_item(9), None);
    }
}
