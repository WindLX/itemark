#!/bin/sh
# Itemark installer: installs from a GitHub Release by default, or from a local
# archive when --archive is given. Safe to pipe from curl.
#
#   curl -fsSL https://raw.githubusercontent.com/WindLX/itemark/main/scripts/install.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/WindLX/itemark/main/scripts/install.sh | sh -s -- --version 0.1.1
#
# Offline:
#   ./install.sh --archive itemark-v0.1.1-x86_64-unknown-linux-gnu.tar.gz
set -eu

repo=${ITEMARK_REPO:-WindLX/itemark}
version=${ITEMARK_VERSION:-}
prefix=${ITEMARK_PREFIX:-${HOME:?HOME must be set}/.local}
base_url=${ITEMARK_BASE_URL:-}
archive=
skill_archive=
skill_profile=
skill_home=
release_target=
have_curl=0
have_wget=0
binary_stage=
receipt_stage=
skill_dest=
binary_installed=0
receipt_installed=0
skill_installed=0
tmp=$(mktemp -d)

usage() {
    cat <<'EOF'
Usage:
  install.sh [--version V|latest] [--prefix DIR] [--repo OWNER/REPO]
             [--base-url URL] [--skill-profile codex|claude] [--skill-home DIR]
  install.sh --archive FILE [--prefix DIR] [--skill-archive FILE]
             [--skill-profile codex|claude] [--skill-home DIR]

Without --archive the installer downloads the archive for this platform from
GitHub Releases; --version defaults to latest and --prefix defaults to
$HOME/.local (the binary goes to $prefix/bin/itemark). With --archive it
installs a local archive instead, and any skill package must be passed
separately with --skill-archive. --skill-profile also installs a skill package
(downloaded in online mode) for the codex or claude profile. --base-url
overrides the release asset directory (mirror or test use) and requires
--version. Environment: ITEMARK_VERSION, ITEMARK_PREFIX, ITEMARK_REPO,
ITEMARK_BASE_URL.
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
        --version) version=${2:?missing value for --version}; shift 2 ;;
        --archive) archive=${2:?missing value for --archive}; shift 2 ;;
        --prefix) prefix=${2:?missing value for --prefix}; shift 2 ;;
        --repo) repo=${2:?missing value for --repo}; shift 2 ;;
        --base-url) base_url=${2:?missing value for --base-url}; shift 2 ;;
        --skill-archive) skill_archive=${2:?missing value for --skill-archive}; shift 2 ;;
        --skill-profile) skill_profile=${2:?missing value for --skill-profile}; shift 2 ;;
        --skill-home) skill_home=${2:?missing value for --skill-home}; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) echo "unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

download() {
    if [ "$have_curl" -eq 1 ]; then
        curl -fsSL -o "$2" "$1"
    else
        wget -q -O "$2" "$1"
    fi
}

sha256_file() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | awk '{print $1}'
    elif command -v openssl >/dev/null 2>&1; then
        openssl dgst -sha256 "$1" | awk '{print $NF}'
    else
        echo "sha256sum, shasum or openssl is required to verify the download" >&2
        exit 1
    fi
}

verify_checksum() {
    file=$1
    sums=$2
    name=${file##*/}
    expected=$(awk -v name="$name" '$2 == name { print $1; exit }' "$sums")
    [ -n "$expected" ] || { echo "SHA256SUMS has no entry for $name" >&2; exit 1; }
    actual=$(sha256_file "$file")
    [ "$expected" = "$actual" ] || { echo "checksum mismatch for $name" >&2; exit 1; }
}

latest_release_version() {
    if [ "$have_curl" -eq 1 ]; then
        url=$(curl -fsSLI -o /dev/null -w '%{url_effective}' "https://github.com/${repo}/releases/latest") ||
            { echo "cannot query the latest release of ${repo}" >&2; exit 1; }
    else
        url=$(wget -qS --spider "https://github.com/${repo}/releases/latest" 2>&1 |
            sed -n 's/^[[:space:]]*[Ll]ocation:[[:space:]]*//p' | tail -n 1)
    fi
    tag=${url##*/}
    case "$tag" in
        v[0-9]*) printf '%s\n' "${tag#v}" ;;
        *) echo "no released version found for ${repo} (latest redirects to $url)" >&2; exit 1 ;;
    esac
}

if [ -z "$archive" ]; then
    command -v curl >/dev/null 2>&1 && have_curl=1
    command -v wget >/dev/null 2>&1 && have_wget=1
    if [ "$have_curl" -eq 0 ] && [ "$have_wget" -eq 0 ]; then
        echo "online install needs curl or wget; use --archive to install a local archive" >&2
        exit 1
    fi
    case "$version" in
        ""|latest)
            [ -z "$base_url" ] || { echo "--base-url requires an explicit --version" >&2; exit 2; }
            version=$(latest_release_version)
            ;;
        v*) version=${version#v} ;;
    esac
    printf '%s' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+([-+][A-Za-z0-9.]+)?$' ||
        { echo "invalid version: $version (expected something like 0.1.1)" >&2; exit 2; }
    case "$(uname -s)/$(uname -m)" in
        Linux/x86_64|Linux/amd64) release_target=x86_64-unknown-linux-gnu ;;
        Linux/aarch64|Linux/arm64) release_target=aarch64-unknown-linux-gnu ;;
        Darwin/x86_64) release_target=x86_64-apple-darwin ;;
        Darwin/arm64|Darwin/aarch64) release_target=aarch64-apple-darwin ;;
        *)
            echo "unsupported platform: $(uname -s)/$(uname -m)" >&2
            echo "release archives exist for Linux/macOS on x86_64 and aarch64; on Windows use scripts/install.ps1" >&2
            exit 1
            ;;
    esac
    base_url=${base_url:-https://github.com/${repo}/releases/download/v${version}}
    archive="$tmp/itemark-v${version}-${release_target}.tar.gz"
    sums="$tmp/SHA256SUMS"
    echo "Downloading itemark v${version} for ${release_target}"
    download "${base_url}/itemark-v${version}-${release_target}.tar.gz" "$archive"
    download "${base_url}/SHA256SUMS" "$sums"
    verify_checksum "$archive" "$sums"
    if [ -n "$skill_profile" ] && [ -z "$skill_archive" ]; then
        skill_archive="$tmp/itemark-skill-v${version}.zip"
        download "${base_url}/itemark-skill-v${version}.zip" "$skill_archive"
        verify_checksum "$skill_archive" "$sums"
    fi
fi

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
case ":${PATH:-}:" in
    *":${prefix}/bin:"*) ;;
    *) printf 'Add %s/bin to PATH to run itemark directly:\n  export PATH="%s/bin:$PATH"\n' "$prefix" "$prefix" ;;
esac
