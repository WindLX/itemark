#!/bin/sh
set -eu

archive=
prefix=${HOME:?HOME must be set}/.local
skill_archive=
skill_profile=
skill_home=
binary_stage=
receipt_stage=
skill_dest=
binary_installed=0
receipt_installed=0
skill_installed=0
tmp=$(mktemp -d)

usage() {
    cat <<'EOF'
Usage: install.sh --archive FILE [--prefix DIR] [--skill-archive FILE]
                  [--skill-profile codex|claude] [--skill-home DIR]

Installs a local release archive. The optional skill archive is separate from
the binary archive. --skill-home overrides the per-user skill parent directory
for isolated installs and testing.
EOF
}

cleanup() {
    result=$?
    trap - EXIT
    set +e
    if [ "$result" -ne 0 ]; then
        [ "$skill_installed" -eq 0 ] || rm -rf "$skill_dest"
        [ "$receipt_installed" -eq 0 ] || rm -f "$prefix/.itemark-install"
        [ "$binary_installed" -eq 0 ] || rm -f "$prefix/bin/itemark"
    fi
    [ -z "$binary_stage" ] || rm -f "$binary_stage"
    [ -z "$receipt_stage" ] || rm -f "$receipt_stage"
    rm -rf "$tmp"
    exit "$result"
}
trap cleanup EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM

while [ "$#" -gt 0 ]; do
    case "$1" in
        --archive) archive=${2:?missing value for --archive}; shift 2 ;;
        --prefix) prefix=${2:?missing value for --prefix}; shift 2 ;;
        --skill-archive) skill_archive=${2:?missing value for --skill-archive}; shift 2 ;;
        --skill-profile) skill_profile=${2:?missing value for --skill-profile}; shift 2 ;;
        --skill-home) skill_home=${2:?missing value for --skill-home}; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) echo "unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

[ -n "$archive" ] || { echo "--archive is required" >&2; exit 2; }
[ -f "$archive" ] || { echo "archive not found: $archive" >&2; exit 1; }
if [ -n "$skill_archive" ]; then
    [ -f "$skill_archive" ] || { echo "skill archive not found: $skill_archive" >&2; exit 1; }
    case "$skill_profile" in
        codex) default_skill_home=${HOME}/.agents/skills ;;
        claude) default_skill_home=${HOME}/.claude/skills ;;
        "") echo "--skill-profile codex|claude is required with --skill-archive" >&2; exit 2 ;;
        *) echo "invalid skill profile: $skill_profile" >&2; exit 2 ;;
    esac
    skill_home=${skill_home:-$default_skill_home}
    command -v unzip >/dev/null 2>&1 || { echo "unzip is required to install the skill archive" >&2; exit 1; }
elif [ -n "$skill_profile" ] || [ -n "$skill_home" ]; then
    echo "--skill-profile and --skill-home require --skill-archive" >&2
    exit 2
fi

# Validate both archives before creating installation files.
mkdir -p "$tmp/bin"
tar -xzf "$archive" -C "$tmp/bin"
[ -f "$tmp/bin/itemark" ] || { echo "archive must contain an itemark executable at its root" >&2; exit 1; }
[ -x "$tmp/bin/itemark" ] || { echo "archive itemark file is not executable" >&2; exit 1; }
if [ -n "$skill_archive" ]; then
    mkdir -p "$tmp/skill"
    unzip -q "$skill_archive" -d "$tmp/skill"
    [ -f "$tmp/skill/itemark/SKILL.md" ] || { echo "skill archive must contain itemark/SKILL.md" >&2; exit 1; }
    [ -f "$tmp/skill/itemark/references/cli.md" ] || { echo "skill archive is missing itemark/references/cli.md" >&2; exit 1; }
fi

mkdir -p "$prefix/bin"
prefix=$(CDPATH= cd -- "$prefix" && pwd -P)
binary="$prefix/bin/itemark"
receipt="$prefix/.itemark-install"
[ ! -e "$binary" ] && [ ! -L "$binary" ] || { echo "installation already exists under $prefix; remove it with uninstall.sh first" >&2; exit 1; }
[ ! -e "$receipt" ] && [ ! -L "$receipt" ] || { echo "installation receipt already exists under $prefix" >&2; exit 1; }

if [ -n "$skill_archive" ]; then
    mkdir -p "$skill_home"
    skill_home=$(CDPATH= cd -- "$skill_home" && pwd -P)
    skill_dest="$skill_home/itemark"
    [ ! -e "$skill_dest" ] && [ ! -L "$skill_dest" ] || { echo "skill already exists: $skill_dest" >&2; exit 1; }
fi

# Stage the executable and receipt beside their final paths, then link without
# overwriting a file that appeared after the preflight check.
binary_stage=$(mktemp "$prefix/bin/.itemark.XXXXXX")
install -m 0755 "$tmp/bin/itemark" "$binary_stage"
ln "$binary_stage" "$binary" || { echo "installation target appeared: $binary" >&2; exit 1; }
binary_installed=1

receipt_stage=$(mktemp "$prefix/.itemark-install.XXXXXX")
printf 'prefix=%s\n' "$prefix" > "$receipt_stage"
ln "$receipt_stage" "$receipt" || { echo "installation receipt appeared: $receipt" >&2; exit 1; }
receipt_installed=1

if [ -n "$skill_archive" ]; then
    mkdir "$skill_dest" || { echo "skill destination appeared: $skill_dest" >&2; exit 1; }
    skill_installed=1
    cp -R "$tmp/skill/itemark/." "$skill_dest/"
    printf '%s\n' 'installed by itemark' > "$skill_dest/.itemark-managed"
    printf 'skill=%s\n' "$skill_dest" >> "$receipt"
    echo "Installed skill $skill_dest"
fi

echo "Installed $binary"
echo "Add $prefix/bin to PATH if needed."
