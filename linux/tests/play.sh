#!/bin/sh
# linux/sai-play end to end with a recording stand-in for SAILauncher, in the
# repository layout (library in ../sdk) and the package layout (library and
# binary beside the script): sh linux/tests/play.sh
set -u
HERE=$(cd "$(dirname "$0")" && pwd -P)
ROOT=$(cd "$HERE/../.." && pwd -P)
T=$(mktemp -d "${TMPDIR:-/tmp}/sai-play-test.XXXXXX")
trap 'rm -rf "$T"' EXIT
export T HOME=$T
unset XDG_CONFIG_HOME HYPRLAND_INSTANCE_SIGNATURE GAMESCOPE_WAYLAND_DISPLAY \
  DRI_PRIME __NV_PRIME_RENDER_OFFLOAD WINEDLLOVERRIDES SAI_LAUNCHER SAI_WAIT \
  SAI_GPU 2>/dev/null
fails=0
ok() { echo "ok   $1"; }
fail() { echo "FAIL $1"; fails=$((fails + 1)); }
check() { if [ "$2" = "$3" ]; then ok "$1"; else fail "$1: got '$2', want '$3'"; fi; }
contains() { case "$2" in *"$3"*) ok "$1" ;; *) fail "$1: '$2' lacks '$3'" ;; esac; }

# The stand-in: records the window run in $T/settings (exit $T/rc) and the
# game run in $T/ran; it does not open anything.
make_fake() {
  cat > "$1" <<'FAKE'
#!/bin/sh
case "$*" in
  *--settings*) echo "$*" > "$T/settings"; exit "$(cat "$T/rc" 2>/dev/null || echo 0)" ;;
esac
echo "$*" > "$T/ran"
FAKE
  chmod +x "$1"
}

mkdir -p "$T/repo/linux" "$T/repo/sdk" "$T/pkg/linux"
cp "$ROOT/linux/sai-play" "$T/repo/linux/"
cp "$ROOT/sdk/rea-play.sh" "$T/repo/sdk/"
cp "$ROOT/linux/sai-play" "$ROOT/sdk/rea-play.sh" "$T/pkg/linux/"
make_fake "$T/repo/SAILauncher"
make_fake "$T/pkg/linux/SAILauncher"

for layout in repo pkg; do
  play=$T/$layout/linux/sai-play
  check "[$layout] status" "$(sh "$play" status)" idle
  rm -f "$T/ran" "$T/settings"
  echo 0 > "$T/rc"
  sh "$play" > "$T/out" 2>&1; rc=$?
  check "[$layout] window, then the game: exit 0" "$rc" 0
  contains "[$layout] the window opens through the helper" "$(cat "$T/settings" 2>/dev/null)" "--settings --linux"
  contains "[$layout] without the window" "$(cat "$T/ran" 2>/dev/null)" "--no-window --linux"
  rm -f "$T/ran" "$T/settings"
  echo 3 > "$T/rc"
  sh "$play" > "$T/out" 2>&1
  check "[$layout] window closed without Play: nothing starts" "$(cat "$T/ran" 2>/dev/null)" ""
  rm -f "$T/ran"
  sh "$play" --play > "$T/out" 2>&1; rc=$?
  check "[$layout] --play: exit 0" "$rc" 0
  contains "[$layout] --play runs the game" "$(cat "$T/ran" 2>/dev/null)" "--no-window --linux"
  check "[$layout] stop" "$(sh "$play" stop)" stopped
  rm -f "$T/ran"
  sh "$play" --check --game "$T/roms" > "$T/out" 2>&1
  contains "[$layout] --check is forwarded" "$(cat "$T/ran" 2>/dev/null)" "--check --game"
done

echo "# source the script: REA_FUNCTIONS_ONLY=1 + REA_PLAY_HERE"
got=$(REA_FUNCTIONS_ONLY=1 REA_PLAY_HERE=$T/repo/linux sh -c '
  . "$1"
  echo "app=$REA_APP mode=$REA_MODE exe=$REA_EXE play=$REA_PLAY_ARGS gpu=${REA_GPU-unset}"
  echo "dropped=${REA_FUNCTIONS_ONLY-unset}/${REA_PLAY_HERE-unset}"' sh "$T/repo/linux/sai-play" 2>&1)
contains "sourced: configured" "$got" "app=sai-launcher mode=native exe=$T/repo/SAILauncher play=--no-window --linux"
contains "sourced: no automatic GPU choice" "$got" "gpu="
contains "sourced: both variables dropped once loaded" "$got" "dropped=unset/unset"

rm -f "$T/repo/SAILauncher"
out=$(sh "$T/repo/linux/sai-play" --play 2>&1); rc=$?
check "no launcher: exit 1" "$rc" 1
contains "no launcher: names the variable" "$out" "SAI_LAUNCHER"

if [ "$fails" = 0 ]; then echo "all passed"; exit 0; fi
echo "$fails failed"; exit 1
