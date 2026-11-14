// Every tick: the bay restores itself from the setting files once per
// session — the files are the durable copy (the bay survives restarts and
// deaths, because the elytra it holds is not part of the world's inventory),
// and the live copy fills from them the first tick it is empty. Nothing else
// runs per tick: the bay's clicks already wrote the files through.
// Copyright (c) 2026 hakergeniusz. MIT.
fn elytra_slot_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::elytra_slot as elytra;
    if !game.native_on(elytra::ID) {
        return;
    }
    if !elytra::setting_on(|k| game.native_setting(elytra::ID, k)) {
        return;
    }
    if elytra::held().is_none()
        && let Some(stack) = elytra::stack_of(|k| game.native_setting(elytra::ID, k))
    {
        elytra::set(Some(stack));
    }
}
