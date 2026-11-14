// The game-side half of the bay: the mod's plain-text stack as the game's
// own ItemStack, and back. The only fns that touch the game's item types —
// everything that decides stays in the runtime module, tested on its own.
// Copyright (c) 2026 hakergeniusz. MIT.
use crate::game::elytra_slot as elytra;
use horizon_client::items::ItemStack;

/// The bay's text state as a real stack: the item, its damage bar and its
/// enchantments (the glint and the tooltip read them), plus the repair cost.
/// The custom name is the one component the files do not keep.
pub fn from_text(s: &elytra::SlotStack) -> ItemStack {
    let mut stack = ItemStack::new(s.item.as_str(), s.count);
    stack.damage = s.damage;
    if !s.ench.is_empty() || s.repair > 0 {
        let mut data = crate::item_data::StackData::default();
        data.enchantments = s
            .ench
            .iter()
            .map(|(id, level)| (std::sync::Arc::from(id.as_str()), *level))
            .collect();
        data.repair_cost = s.repair;
        stack.extra = Some(Box::new(data));
    }
    stack
}

/// A real stack as the bay's text state: what an elytra carries.
pub fn into_text(stack: &ItemStack) -> elytra::SlotStack {
    let (ench, repair) = stack
        .extra
        .as_ref()
        .map_or((Vec::new(), 0), |data| {
            (
                data.enchantments
                    .iter()
                    .map(|(id, level)| (id.to_string(), *level))
                    .collect(),
                data.repair_cost,
            )
        });
    elytra::SlotStack {
        item: stack.item.to_string(),
        count: stack.count,
        damage: stack.damage,
        ench,
        repair,
    }
}
