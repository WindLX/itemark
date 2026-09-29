#!/bin/sh
# Itemark uninstaller: like the installer, it can be run straight from the network.
#
#   curl -fsSL https://raw.githubusercontent.com/WindLX/itemark/main/scripts/uninstall.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/WindLX/itemark/main/scripts/uninstall.sh | sh -s -- --prefix "$HOME/.local"
set -eu

prefix=${ITEMARK_PREFIX:-${HOME:?HOME must be set}/.local}
usage() { echo "Usage: uninstall.sh [--prefix DIR]   (default: \$HOME/.local)"; }
while [ "$#" -gt 0 ]; do
    case "$1" in
        --prefix) prefix=${2:?missing value for --prefix}; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) echo "unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

if [ ! -d "$prefix" ]; then
    echo "installation prefix not found: $prefix" >&2
    exit 1
fi
prefix=$(CDPATH= cd -- "$prefix" && pwd -P)
receipt="$prefix/.itemark-install"
binary="$prefix/bin/itemark"
[ -f "$receipt" ] || { echo "no Itemark installation receipt under $prefix" >&2; exit 1; }
grep -Fqx "prefix=$prefix" "$receipt" || { echo "installation receipt does not match $prefix" >&2; exit 1; }
rm -f "$binary"
skill=$(sed -n 's/^skill=//p' "$receipt")
if [ -n "$skill" ]; then
    case "$skill" in */itemark) ;; *) echo "invalid skill path in receipt; preserving it" >&2; exit 1 ;; esac
    if [ -f "$skill/.itemark-managed" ] && [ ! -L "$skill" ] && [ "$(cat "$skill/.itemark-managed")" = 'installed by itemark' ]; then
        rm -rf "$skill"
    else
        echo "skill installation marker missing; preserving $skill" >&2
    fi
fi
rm -f "$receipt"
echo "Removed Itemark files installed under $prefix; parent directories and project records were preserved."
