# The all-mods build — what ships in this client

`tools/build-all.py` composes every bundle below into one Horizon client
(release profile, glibc ≤ 2.43 floor). This file says what each one does and
what it adds to the game. In-game verification status noted per mod.

## The user's clean-room mods (built + signed locally)

| Mod | What it does | Controls / settings | Verified |
|---|---|---|---|
| **iron_stomach 1.1.0** | Hunger freezes at full; the food tick RUNS so vanilla natural regen keeps healing (1 HP / 4 s) while the bar never drains. Guests/mobs keep vanilla hunger. | always on; Options > Mods toggle | staged tests 6/6 + staged regen walk; in-game pass pending (`~/Projects/iron_stomach/tools/ingame-test.txt`) |
| **soft_landing 1.0.0** | Falls past the vanilla-safe height land unhurt (the clean-room nofall replacement; fall damage is server-side only in Horizon). | always on | in-game ✓ (25-block drop unhurt) |
| **freecam 0.2.0** | Detach the camera from the player and fly it; F3 overlay follows the camera; bots' clicks still mine. | keybind (freecam's settings page) | in-game ✓ (toast, camera flight, agent mining) |
| **statview 1.0.3** | The BetterStats-style statistics overlay with live data, search and tab filtering. | its key opens it | in-game ✓ (typing, filter, backspace) |
| **offline_name 0.1.0** | Per-server offline names + auto-`/login` (AuthMe-inspired). Needs real title-UI driving (ctl connect bypasses it). | its settings | in-game ✓ |
| **portal_interact 0.1.0** | The inventory opens inside the nether portal overlay. | always on | in-game ✓ |
| **fly 0.1.0** | Survival flight on a keybind: toggles `may_fly`/`flying`; the engine tells the server on its own. Pairs with soft_landing for the drop. | **R** toggles (rebindable) | chain green; in-game toggle pass pending |
| **pingfix 0.1.0** | Tab-player-list latency bars. | always on | in-game ✓ (Tab bars) |
| **baritone_helper 0.1.0** | Chat helper for the bundled Baritone: `!status`, `!q` stacked queue, dispatch by describe+pathing_busy. Needs Baritone 1.1.52. | chat commands | in-game ✓ (queue walks both legs) |
| **trajectory_preview 0.1.0** | Draws the flight of the projectile in hand (bow draw curve, crossbow, trident, throwables, potion lob) with the game's own physics; white line, red impact box, depth-tested. | always on | staged check + staged tests 5/5 (`~/Projects/trajectory_preview/tools/ingame-test.txt`) |
| **barrel_roll 0.1.0** | Elytra flight camera unlock: mouse X rolls, A/D steers, the camera banks into turns; physics untouched. | Options > Mods toggle | staged check + staged tests 7/7 (`~/Projects/barrel_roll/tools/ingame-test.txt`) |
| **elytra_slot 0.1.0** | Dedicated elytra slot drawn next to the armor slots: only elytra fits, click swaps it with the chest slot, gliding works from it; settings to disable. | always on (toggle in settings) | staged check + staged tests 13/13 (`~/Projects/elytra_slot/tools/ingame-test.txt`) |
| **hey_wiki 0.1.0** | H opens the Minecraft Wiki page of the aimed block/mob or hovered/held item — fetched from minecraft.wiki's MediaWiki API and rendered NATIVELY in a scrollable in-game viewer (loading state, error note); B opens wiki search; browser only a fallback button. | H / Alt+H / B + Options > Mods | staged check + staged tests 41/41 (`~/Projects/hey_wiki/tools/ingame-test.txt`) |
| **no_chat_reports 0.1.0** | Unsigned, unreportable chat (Disable Signing + hub relay strip); trust-badge gates; NCR + No-Chat-Restrictions combined. | Options > Mods rows | staged tests 5/5 |
| **worldedit 1.2.1** | The store bundle, chat-crate-migrated to the current pin (a dedicated re-target is in flight; this build ships the migrated manifest). | chat commands | disabled upstream; migration pending |

## The store bundles (upstream, enabled in the user's set)

eatinganimation 0.2.0 · fps_overlay 2.0.0 · fullbright_horizon 1.0.0 ·
mousetricks 0.2.0 · zoom 2.0.0 · smartf3 0.1.0 (patched for 2ebf00a) ·
chatheads 0.1.1 · motion_blur 0.1.0 · portal interact's companions ·
recipes 2.0.0 (user keeps it OFF) · schematics 1.0.0 · shulker_peek 0.1.0 ·
**baritone 1.1.52** (vendor; the installed 1.1.38 is superseded —
baritone_helper is verified against 1.1.52).

## Build

```
python3 -I tools/build-all.py --profile release   # 25 bundles, 134 modules, 172 injections
```

The glibc ≤ 2.43 deliverable builds inside docker `debian:trixie` (recipe in
the session memory); the artifact is
`dist/horizon-client-allmods-release-glibc243`, installed into the launcher
version dir via the same-directory rename.
