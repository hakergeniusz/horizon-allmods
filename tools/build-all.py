#!/usr/bin/env python3
"""Build one Horizon client with every native mod at once: the launcher's installed
bundles, the four pending local ones and Baritone 1.1.52 (what baritone_helper is
verified against), on the launcher's own pinned source, CLI and toolchain.

Bundles arrive two ways, both re-rooted into `mods/<id>/` (the mixin CLI's layout):
extracted bundles from the launcher's store, and `.hnmod` archives (untrusted data —
every member name is checked before anything is written). Bundles that pin another
Horizon revision get the launcher's pin rewrite, then one apply composes the stage and
the launcher's own cargo line builds it into this project's target dir.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import zipfile

HOME = Path.home()
LAUNCHER = HOME / ".local/share/horizon-launcher"
STORE = LAUNCHER / "native/installed.incoming"
GAME_MODS = Path.home() / ".local/share/horizon/horizon-native-mods"
SOURCE = LAUNCHER / "native/source"
CLI = LAUNCHER / "native/tools/5a64af938ec8/bin/horizon-mixins"
RUST = LAUNCHER / "rust/1.99.1-x86_64-unknown-linux-gnu"

ROOT = Path(__file__).resolve().parents[1]
PIN = "2ebf00a0281c592f79191e76fce0e7d522c6f425"
MIXINS_PIN = "5a64af938ec86b00d53d16a408aabc0038fb3e57"

# What goes in, and where each bundle comes from. The launcher's own builds
# rewrite every pin to the workspace revision; so does this script. freecam is
# 0.2.0, baritone is 1.1.52 from the registry release baritone_helper is
# verified against (the installed one is 1.1.38), and the local mods come from
# their own projects' targets. The GPL nohunger/nofall ports were replaced by
# the clean-room iron_stomach/soft_landing (MIT, spec-only implementations);
# statview is the clean-room statistics screen.
BUNDLES = [
    ("eatinganimation", STORE / "eatinganimation", None),
    ("fps_overlay", STORE / "fps_overlay", None),
    ("fullbright_horizon", STORE / "fullbright_horizon", None),
    ("mousetricks", STORE / "mousetricks", None),
    ("zoom", STORE / "zoom", None),
    ("smartf3", STORE / "smartf3", None),
    ("worldedit", STORE / "worldedit", None),
    ("chatheads", GAME_MODS / "chatheads.hnmod", None),
    ("motion_blur", GAME_MODS / "motion_blur.hnmod", None),
    ("recipes", GAME_MODS / "recipes.hnmod", None),
    ("schematics", GAME_MODS / "schematics.hnmod", None),
    ("shulker_peek", GAME_MODS / "shulker_peek.hnmod", None),
    ("baritone", ROOT / "vendor/baritone-1.1.52.hnmod", "d46c541d1e673a95f343960b6fb1646a0669ec07df11958af97f1a76b3b14ffa"),
    ("freecam", HOME / "Projects/freecam/target/freecam-0.2.0.hnmod", None),
    ("pingfix", HOME / "Projects/pingfix/target/pingfix-0.1.0.hnmod", None),
    ("statview", HOME / "Projects/statview/target/statview-1.1.0.hnmod", None),
    ("portal_interact", HOME / "Projects/portal_interact/target/portal_interact-0.1.0.hnmod", None),
    ("iron_stomach", HOME / "Projects/iron_stomach/target/iron_stomach-1.1.0.hnmod", None),
    ("soft_landing", HOME / "Projects/soft_landing/target/soft_landing-1.0.0.hnmod", None),
    ("offline_name", HOME / "Projects/offline_name/target/offline_name-0.1.0.hnmod", None),
    ("baritone_helper", HOME / "Projects/baritone-helper/target/baritone_helper-0.1.0.hnmod", None),
    ("fly", HOME / "Projects/fly/target/fly-0.1.0.hnmod", None),
    ("no_chat_reports", HOME / "Projects/no_chat_reports/target/no_chat_reports-0.1.0.hnmod", None),
    ("trajectory_preview", HOME / "Projects/trajectory_preview/target/trajectory_preview-0.2.0.hnmod", None),
    ("elytra_slot", HOME / "Projects/elytra_slot/target/elytra_slot-0.2.0.hnmod", None),
    ("hey_wiki", HOME / "Projects/hey_wiki/target/hey_wiki-0.2.0.hnmod", None),
    # barrel_roll dropped 2026-11-14: gyro (CGRLabs, MIT) already does the
    # Do-a-Barrel-Roll port on the registry; bundle_inventory dropped: the
    # registry's shulkerboxtooltip shows bundle contents in tooltips. Both
    # registry mods are vendor-pinned by sha256 (unsigned upstream).
    ("mod_menu", ROOT / "vendor/mod_menu-0.3.5.hnmod", "5fef518e6ce7417944ab2621711be013cf7e25fdd6d3b25922876e78f090bfa3"),
    ("shulkerboxtooltip", ROOT / "vendor/shulkerboxtooltip-1.0.1.hnmod", "005244f696d9c5214980f554251f6442523768f040a78f505ef8d36090484bba"),
]

# Nothing is excluded now: worldedit's manifest is migrated below. The helpers
# themselves may still name paths that moved — the staged check names each one.
EXCLUDED = {}

# Small stage-local migrations for bundles that apply but no longer compile.
# Each entry is a bundle, the file, and (old, new) pairs of exact text.
# smartf3's crosshair entity test: `mob_hit` now also returns the hit position
# (ignored, as the game's own debug entry does), and `guest_hit` is gone — LAN
# guests are world entities the mobs list carries, so mob_hit already offers
# them under their own kind. Mirrors game/debug_lines.rs at this pin.
PATCHES = [
    # worldedit was written when Horizon had one chat.rs; at this pin the
    # chat screen lives in the render crate, and the render crate cannot reach
    # back into horizon-client — so both of worldedit's chat-side modules move
    # into the render crate's chat.rs (paths is already `pub` there: the game
    # side reads it across the crate line), the suggestion hook's target moves with its
    # screen, and every game-side reference to paths follows it there.
    ("worldedit", "horizon-mixin.toml", [
        ('file = "crates/horizon-client/src/chat.rs"',
         'file = "crates/horizon-client-render/src/chat.rs"'),
        ('target = "horizon_client::chat::ChatScreen::update_command_info"',
         'target = "horizon_client_render::chat::ChatScreen::update_command_info"'),
    ]),
    ("worldedit", "*.rs", [
        ("horizon_client::chat::worldedit_paths",
         "horizon_client_render::chat::worldedit_paths"),
        # show_suggestions grew the `immediate` flag (the narrator says the
        # first completion at once when Tab asked); the game's own auto path
        # passes false, and so does this one.
        ("show_suggestions(m)", "show_suggestions(m, false)"),
    ]),
    # API drift since the bundles were written: tooltip_box now names the
    # stack it draws for (shulker_peek has it at hand), an item's custom name
    # is an NBT Tag, and the Y clamp comes from the loaded level's shape (the
    # game's own is_loaded impl reads it the same way).
    ("shulker_peek", "*.rs", [
        # tooltip_box became a method that names the stack it draws for.
        ("crate::game::Game::tooltip_box(batch, atlas, at.x, at.y, at.w, at.h);",
         "crate::game::Game::tooltip_box(&game, batch, Some(stack), at.x, at.y, at.w, at.h);"),
        ('= Some("Edge".into())',
         '= Some(horizon_world::nbt::Tag::String("Edge".into()))'),
    ]),
    ("worldedit", "*.rs", [
        ("game.dim.min_y()", "game.world.shape.min_y"),
        ("game.dim.max_y()", "game.world.shape.max_y()"),
        ("= Some(WAND_NAME.to_owned())",
         "= Some(horizon_world::nbt::Tag::String(WAND_NAME.into()))"),
        ("data.custom_name.as_deref()) == Some(WAND_NAME)",
         "data.custom_name.as_ref()) == Some(&horizon_world::nbt::Tag::String(WAND_NAME.into()))"),
    ]),
    ("smartf3", "native.rs", [
        ("if let Some((i, d)) = self.mob_hit() {",
         "if let Some((i, d, _)) = self.mob_hit() {"),
        ("""        if let Some((_, d)) = self.guest_hit() {
            offer(d, "minecraft:player".to_owned());
        }
""", ""),
    ]),
]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def unpack(bundle_id, origin, hnmod_sha, mods):
    """One bundle into mods/<id>/: from an extracted store dir, or from a .hnmod."""
    out = mods / bundle_id
    shutil.rmtree(out, ignore_errors=True)
    if origin.is_dir():
        inner = origin / "mixins" / bundle_id
        if not inner.is_dir():
            raise ValueError(f"{origin} holds no mixins/{bundle_id}/")
        shutil.copytree(inner, out)
        return
    if hnmod_sha is not None and digest(origin) != hnmod_sha:
        raise ValueError(f"{origin.name} does not match its pinned SHA-256")
    prefix = f"mixins/{bundle_id}/"
    with zipfile.ZipFile(origin) as archive:
        for info in archive.infolist():
            if info.is_dir():
                continue
            if not info.filename.startswith(prefix):
                if info.filename == "horizon-native.json":
                    continue
                raise ValueError(f"unexpected member in {origin.name}: {info.filename}")
            relative = Path(info.filename)
            if relative.is_absolute() or ".." in relative.parts:
                raise ValueError(f"unsafe member name: {info.filename}")
            target = out / relative.relative_to(prefix)
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(archive.read(info))
    if not (out / "horizon-mixin.toml").exists():
        raise ValueError(f"{origin.name} holds no {prefix}horizon-mixin.toml")


def repin(bundle_id, mods):
    """The launcher's rewrite: the manifest's one horizon_revision line becomes the pin."""
    manifest = mods / bundle_id / "horizon-mixin.toml"
    text = manifest.read_text()
    lines = [line for line in text.splitlines() if "horizon_revision" in line]
    if len(lines) != 1:
        raise ValueError(f"{manifest}: expected exactly one horizon_revision")
    if PIN not in lines[0]:
        manifest.write_text(text.replace(lines[0], f'horizon_revision = "{PIN}"'))
        return bundle_id
    return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--skip-build", action="store_true", help="stage only")
    parser.add_argument("--profile", default="fast", choices=["fast", "release"],
                        help="fast = optimized-but-quick iteration; release = the launcher's \"performance\" (thin LTO, one codegen unit)")
    args = parser.parse_args()

    source = SOURCE / PIN
    if not (source / "Cargo.toml").exists():
        raise ValueError(f"the launcher holds no source export at {source}")

    mods = ROOT / "mods"
    shutil.rmtree(mods, ignore_errors=True)
    mods.mkdir(parents=True)
    repinned = []
    for bundle_id, origin, sha in BUNDLES:
        if not origin.exists():
            raise ValueError(f"missing bundle source: {origin}")
        unpack(bundle_id, origin, sha, mods)
        if repin(bundle_id, mods):
            repinned.append(bundle_id)
    for bundle_id, name, pairs in PATCHES:
        files = [mods / bundle_id / name]
        if name == "*.rs":
            files = sorted((mods / bundle_id).glob("*.rs"))
        for path in files:
            text = path.read_text()
            for old, new in pairs:
                if not text.count(old):
                    continue
                text = text.replace(old, new)
            path.write_text(text)
    print(f"composed {len(BUNDLES)} bundles ({', '.join(repinned) or 'none'} repinned)", flush=True)

    stage = ROOT / "stage"
    shutil.rmtree(stage, ignore_errors=True)
    applied = subprocess.run([str(CLI), "apply", "--workspace", str(source), "--mods", str(mods),
                              "--out", str(stage)], check=True, capture_output=True, text=True)
    report = json.loads(applied.stdout)
    print(json.dumps({k: report[k] for k in ("modules", "injections", "status", "seconds")}), flush=True)

    # The launcher writes this file itself; build.rs fails without it.
    (stage / ".horizon-mod-api-injections.json").write_text("[]")

    # The CLI drops a bundle whose targets no longer resolve and still says ok —
    # every bundle's declared modules must have landed in the staged sources, or
    # stop here. Module names come from each bundle's own manifest; the staged
    # sources inline them as `… mod <name> {`.
    import tomllib
    sources = list((stage / "crates").rglob("*.rs"))
    missing = []
    for bundle_id, _, _ in [b for b in BUNDLES if b[0] not in EXCLUDED]:
        manifest = tomllib.loads((mods / bundle_id / "horizon-mixin.toml").read_text())
        names = [m.get("name") for m in manifest.get("module", []) if m.get("name")]
        if names and not any(f"mod {name}" in text.read_text(errors="ignore")
                             for name in names for text in sources):
            missing.append(bundle_id)
    if missing:
        raise ValueError(f"these bundles did not apply: {', '.join(missing)}")
    print("all bundles present in the stage", flush=True)

    if args.skip_build:
        return

    env = dict(os.environ)
    for key in ("CARGO_ENCODED_RUSTDOCFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_INCREMENTAL",
                "RUSTC_WORKSPACE_WRAPPER", "RUSTC_WRAPPER", "RUSTDOCFLAGS", "RUSTFLAGS",
                "RUSTUP_TOOLCHAIN"):
        env.pop(key, None)
    env.update(
        CARGO_HOME=str(LAUNCHER / "cargo"),
        CARGO_TARGET_DIR=str(ROOT / "target"),
        RUSTC=str(RUST / "bin/rustc"),
        TEMP=str(LAUNCHER / "tmp"), TMP=str(LAUNCHER / "tmp"), TMPDIR=str(LAUNCHER / "tmp"),
        HORIZON_MOD_API_INJECTIONS=str(stage / ".horizon-mod-api-injections.json"),
        PATH=f"{RUST}/bin:" + env.get("PATH", ""),
    )
    build = subprocess.run([str(RUST / "bin/cargo"), "build", "--manifest-path", str(stage / "Cargo.toml"),
                            "--locked", "--profile", args.profile, "-p", "horizon-client", "--bin", "horizon-client"],
                           capture_output=True, text=True, env=env)
    log = ROOT / f"target/build-{args.profile}.log"
    log.write_text(build.stdout + build.stderr)
    if build.returncode:
        print(build.stderr[-12000:], file=sys.stderr)
        raise SystemExit(f"cargo build exited {build.returncode}; log: {log}")
    binary = ROOT / f"target/{args.profile}/horizon-client"
    print(json.dumps({"binary": str(binary), "size": binary.stat().st_size,
                      "sha256": digest(binary)}), flush=True)


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        if isinstance(error, subprocess.CalledProcessError) and error.stderr:
            print(error.stderr[-6000:], file=sys.stderr)
        print(f"build-all: {error}", file=sys.stderr)
        raise SystemExit(1)
