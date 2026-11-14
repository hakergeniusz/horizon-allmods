// The game-side half of the mod: the fns that hold the game's types. The
// key helper lands in the mod-hooks file and answers through these; the
// runtime module stays pure and keeps every decision tested. The HTTP is
// the engine's own `ureq` (the client's dependency), run on the fetch's
// worker thread so the game never waits on the network.
// Copyright (c) 2026 hakergeniusz. MIT.

/// The registry id of what the H press looks up, as the runtime's plan
/// asks it: with a screen open, the plain survival inventory's hovered
/// slot; over the world, the crosshair's block or mob within the reach
/// setting. The hits the game already picked this tick (a guest's forced
/// target, a hosted world's tick picks) stand as they are; otherwise the
/// game's own rays run out to the setting — and the mob only when its hit
/// is nearer than the block's, so a wall hides it as always.
pub fn lookup_id(game: &mut crate::game::Game<'_>, alt: bool) -> Option<String> {
    use crate::game::hey_wiki as wiki;
    // Where the press looks: a screen's hover, or the crosshair over the
    // world. `screen_view_now` is None over the world, Some over the screens
    // the hooks take — and the hover is the plain survival inventory's only
    // (a container's slots number its own grid, not the player's).
    let focus = {
        let view = game.screen_view_now();
        wiki::lookup_focus(
            view.is_some(),
            view.is_some_and(|v| v.id == "inventory" && v.menu.is_none()),
            alt,
        )
    };
    match focus {
        wiki::Focus::Nothing => None,
        wiki::Focus::Held => game.held().map(|s| s.item.to_string()),
        wiki::Focus::Hovered => {
            let slot = wiki::hover_slot(game.inventory_target().0?)?;
            match slot {
                wiki::HoverSlot::Armor(which) => game
                    .armor
                    .get(which)?
                    .as_ref()
                    .map(|s| s.item.to_string()),
                wiki::HoverSlot::Offhand => game.stack_in(40).map(|s| s.item.to_string()),
                wiki::HoverSlot::Inventory(which) => {
                    game.stack_in(which).map(|s| s.item.to_string())
                }
            }
        }
        wiki::Focus::Crosshair => aimed_id(game),
    }
}

/// The registry id of what the crosshair is on, block or mob, at the reach
/// the setting holds.
fn aimed_id(game: &mut crate::game::Game<'_>) -> Option<String> {
    use crate::game::hey_wiki as wiki;
    let reach = f64::from(wiki::reach_of(|k| game.native_setting(wiki::ID, k)));
    let (origin, dir) = game.aim()?;
    let block = game.forced_hit.or_else(|| match game.tick_hit {
        Some(picked) => picked,
        None => game.clip_ray(origin, dir, reach),
    });
    let block_t = block.as_ref().map(|hit| {
        let [dx, dy, dz] = [
            hit.point[0] - origin[0],
            hit.point[1] - origin[1],
            hit.point[2] - origin[2],
        ];
        (dx * dx + dy * dy + dz * dz).sqrt()
    });
    let mob = match game.tick_mob {
        Some(picked) => picked.map(|(index, t, _)| (index, t)),
        None => game.mob_hit_within(reach).map(|(index, t, _)| (index, t)),
    };
    match wiki::resolve_target(block_t, mob) {
        Some(wiki::Which::Block) => {
            let hit = block?;
            let state = game.world.block(hit.pos[0], hit.pos[1], hit.pos[2]);
            let name = game.res.facts().get(state).name.clone();
            Some(name.to_string())
        }
        Some(wiki::Which::Mob(index)) => {
            let kind = game.mobs.iter().nth(index)?.kind.clone();
            Some(kind.id.clone())
        }
        None => None,
    }
}

/// Where a URL goes: behind the game's own open-or-copy link confirmation
/// when the setting asks (the dialog answers to the mouse), else straight
/// to the browser (`packs::open_uri`, the game's own link opener).
pub fn open_url(game: &mut crate::game::Game<'_>, url: String) {
    use crate::game::hey_wiki as wiki;
    let confirm = wiki::confirm_open(|k| game.native_setting(wiki::ID, k));
    match wiki::open_plan(confirm) {
        wiki::OpenPlan::Prompt => {
            // The prompt is answered with the cursor, as vanilla's always
            // is — it only ever opens over free-cursor screens (the chat,
            // the links screen). The H press finds the pointer held; let it
            // go first, or the dialog's buttons stay out of reach while the
            // view turns under the mouse. The tick re-grabs once the
            // prompt is answered no, the engine's own refocus re-grab after
            // the browser round-trip on yes.
            if game.captured {
                game.capture(false);
            }
            game.link_prompt = Some(url);
        }
        wiki::OpenPlan::Direct => crate::game::packs::open_uri(&url),
    }
}

/// A wiki page, fetched natively: the view starts waiting and the worker's
/// answer fills it in when it lands. The screen stays as it is.
pub fn fetch_page(title: &str) {
    use crate::game::hey_wiki as wiki;
    let Some(url) = wiki::api_extract_url(title) else {
        return;
    };
    wiki::with_viewer(|viewer| viewer.start(wiki::FetchKind::Page, title));
    wiki::fetch_async(wiki::FetchKind::Page, title.to_string(), url, http);
}

/// A wiki page, fetched natively and shown over the world: the mod's own
/// screen opens on top of the fetch's wait.
pub fn show_page(game: &mut crate::game::Game<'_>, title: &str) {
    use crate::game::hey_wiki as wiki;
    fetch_page(title);
    game.native_mod_screen_open(wiki::ID);
}

/// A wiki search, fetched natively: the view waits for the ranked titles.
/// The screen is already open (the box sent the query); nothing opens.
pub fn search(query: &str) {
    use crate::game::hey_wiki as wiki;
    let Some(url) = wiki::api_search_url(query) else {
        return;
    };
    wiki::with_viewer(|viewer| viewer.start(wiki::FetchKind::Search, query));
    wiki::fetch_async(wiki::FetchKind::Search, query.to_string(), url, http);
}

/// The HTTP the workers run: the engine's own client, ten seconds to answer,
/// the body or nothing (any failure is the same nothing to the fetch).
fn http(url: &str) -> Option<String> {
    let response = ureq::get(url)
        .timeout(std::time::Duration::from_secs(10))
        .call();
    response.ok().and_then(|answer| answer.into_string().ok())
}
