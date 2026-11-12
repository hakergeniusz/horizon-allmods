#!/bin/bash
# ingame-run.sh — drive one in-game verification: Xvfb display, the all-mods
# client with --control, focus+click (the sim pauses until a human-like focus
# click), then a horizon-ctl script; collect the screenshots.
#
# Usage: ingame-run.sh <world-name> <ctl-script.txt> [resources-dir]
#   GAME_BIN  overrides the client binary (default: the launcher's version
#             dir — the all-mods release build).
#   The ctl scripts must target the DEFAULT socket shape: this driver points
#   $HORIZON_CONTROL at the run's socket, which horizon-ctl reads first.
#
# Gotchas baked in (from the 2026-11-11 in-game sessions):
#   - launch with the Wayland session vars UNSET so winit takes X11;
#   - the sim PAUSES until the window is focused and clicked — the driver
#     focuses the window and sends one XTEST click before the script runs;
#   - xdotool must use plain XTEST (no --window XSendEvent, winit drops it);
#   - the game's data dir is the client's CWD (screenshots land there).
#
# Never run this while a compile is running (strict alternation).
set -u
WORLD="$1"; SCRIPT="$2"; RES="${3:-$HOME/.local/share/horizon/.horizon-cache/resourcepacks/7d8d6b710ab27c7f}"
GAME_BIN="${GAME_BIN:-$HOME/.local/share/horizon-launcher/versions/native-2ebf00a-42ebf5dd24f9/horizon-client}"
DATA="$(mktemp -d /tmp/hz-agent/game-XXXX)"
SOCK="$DATA/ctl.sock"
export HORIZON_CONTROL="$SOCK"

CG=/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/horizon-game
cleanup() {
  [ -n "${CLIENT_PID:-}" ] && kill "$CLIENT_PID" 2>/dev/null
  [ -n "${XVFB_PID:-}" ] && kill "$XVFB_PID" 2>/dev/null
  wait 2>/dev/null
}
trap cleanup EXIT

echo "data $DATA"
Xvfb :99 -screen 0 1280x720x24 & XVFB_PID=$!
sleep 1
mkdir -p "$CG" 2>/dev/null || true

(
  echo $$ > "$CG/cgroup.procs" 2>/dev/null || true
  echo 4G > "$CG/memory.max" 2>/dev/null || true
  echo 0 > "$CG/memory.swap.max" 2>/dev/null || true
  cd "$DATA"
  exec env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET -u XDG_SESSION_TYPE DISPLAY=:99 \
    "$GAME_BIN" --resources "$RES" --world "$WORLD" --width 1280 --height 720 --control
) & CLIENT_PID=$!

for i in $(seq 1 60); do
  horizon-ctl status >/dev/null 2>&1 && break
  sleep 1
done
horizon-ctl status || { echo "client never answered"; exit 1; }

# focus the window and click once (XTEST): the sim stays paused otherwise
sleep 2
WID=$(xdotool search --onlyvisible --class horizon | head -1 || true)
if [ -n "${WID:-}" ]; then
  xdotool windowfocus "$WID" 2>/dev/null || true
  xdotool mousemove 640 360 click 1 2>/dev/null || true
  echo "focused window $WID"
else
  echo "WARN: no window found — the sim may stay paused"
fi

horizon-ctl script "$SCRIPT"
CODE=$?
sleep 1
echo "--- screenshots ---"
cp -v "$DATA"/screenshots/*.png /tmp/hz-agent/ 2>/dev/null || echo "(none)"
horizon-ctl quit 2>/dev/null
exit "$CODE"
