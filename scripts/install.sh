#!/usr/bin/env bash
#
# Install the Photon release binary, mascot app icon, and desktop entry.
#
# By default this installs to the user's local prefix (~/.local), which needs
# no root privileges. Pass --system to install system-wide (requires sudo).
#
# Usage:
#   scripts/install.sh            # user-local install
#   scripts/install.sh --system   # system-wide install (sudo)
#   scripts/install.sh --no-build # skip cargo build, use existing binary
#
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY_NAME="photon"
ICON_ID="dev.photon.photon"

BUILD_RELEASE=1
SYSTEM=0

for arg in "$@"; do
    case "$arg" in
        --system)   SYSTEM=1 ;;
        --no-build) BUILD_RELEASE=0 ;;
        *)          echo "Unknown argument: $arg" >&2; exit 2 ;;
    esac
done

if [[ "$SYSTEM" -eq 1 ]]; then
    PREFIX="/usr/local"
    PREFIX_LOCAL="$PREFIX"
    BIN_DIR="$PREFIX/bin"
    ICON_DIR="$PREFIX/share/icons/hicolor"
    DESKTOP_DIR="$PREFIX/share/applications"
    METAINFO_DIR="$PREFIX/share/metainfo"
    RUN_SYSTEM=1
else
    PREFIX="$HOME/.local"
    BIN_DIR="$PREFIX/bin"
    ICON_DIR="$PREFIX/share/icons/hicolor"
    DESKTOP_DIR="$PREFIX/share/applications"
    METAINFO_DIR="$PREFIX/share/metainfo"
    RUN_SYSTEM=0
fi

# ----------------------------- build release -----------------------------
if [[ "$BUILD_RELEASE" -eq 1 ]]; then
    echo ">> Building release binary..."
    (cd "$PROJECT_ROOT" && cargo build --release --bin "$BINARY_NAME")
fi

BINARY_SRC="$PROJECT_ROOT/target/release/$BINARY_NAME"
if [[ ! -f "$BINARY_SRC" ]]; then
    echo "error: release binary not found at $BINARY_SRC" >&2
    echo "       Build it with 'cargo build --release' or pass --no-build." >&2
    exit 1
fi

# ----------------------------- icon generation ---------------------------
MASCOT_SVG="$PROJECT_ROOT/extra/images/logo_app.svg"
echo ">> Generating app icons from mascot..."
TMP_ICON_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_ICON_DIR"' EXIT

ICON_SIZES=(16 32 48 64 128 256 512 1024)
HAVE_RSVG=0
command -v rsvg-convert >/dev/null 2>&1 && HAVE_RSVG=1
HAVE_CONVERT=0
command -v convert >/dev/null 2>&1 && HAVE_CONVERT=1

if [[ "$HAVE_RSVG" -ne 1 && "$HAVE_CONVERT" -ne 1 ]]; then
    echo "error: need rsvg-convert or imagemagick (convert) to build the icons" >&2
    exit 1
fi

for size in "${ICON_SIZES[@]}"; do
    out="$TMP_ICON_DIR/${ICON_ID}-${size}.png"
    if [[ "$HAVE_RSVG" -eq 1 ]]; then
        rsvg-convert -w "$size" -h "$size" "$MASCOT_SVG" -o "$out"
    else
        convert -background none -density 384 "$MASCOT_SVG" -resize "${size}x${size}" "$out"
    fi
done

# ----------------------------- install files -----------------------------
install_with_prefix() {
    local dest="$1"
    if [[ "$RUN_SYSTEM" -eq 1 ]]; then
        sudo install -dm755 "$(dirname "$dest")"
        sudo install -m755 "$2" "$dest"
    else
        mkdir -p "$(dirname "$dest")"
        install -m755 "$2" "$dest"
    fi
}

echo ">> Installing binary to $BIN_DIR/$BINARY_NAME"
install_with_prefix "$BIN_DIR/$BINARY_NAME" "$BINARY_SRC"

echo ">> Installing app icons"
for size in "${ICON_SIZES[@]}"; do
    dest="$ICON_DIR/${size}x${size}/apps/${ICON_ID}.png"
    install_with_prefix "$dest" "$TMP_ICON_DIR/${ICON_ID}-${size}.png"
done

# Scalar SVG icon (some desktops/theme pickers prefer it)
install_with_prefix "$ICON_DIR/scalable/apps/${ICON_ID}.svg" "$MASCOT_SVG"

echo ">> Installing desktop entry"

# Rewrite Exec= to the absolute binary path. Desktop environments launch
# with a limited PATH (~/.local/bin is usually missing), so a bare `photon`
# would fail to resolve. Pointing at the full path works in every case.
DESKTOP_TMP="$TMP_ICON_DIR/${ICON_ID}.desktop"
sed -e "s|^Exec=photon |Exec=$BIN_DIR/$BINARY_NAME |g" \
    "$PROJECT_ROOT/extra/linux/${ICON_ID}.desktop" > "$DESKTOP_TMP"
install_with_prefix "$DESKTOP_DIR/${ICON_ID}.desktop" "$DESKTOP_TMP"

echo ">> Installing appstream metadata"
install_with_prefix \
    "$METAINFO_DIR/${ICON_ID}.metainfo.xml" \
    "$PROJECT_ROOT/extra/linux/${ICON_ID}.metainfo.xml"

# ----------------------------- refresh caches ----------------------------
echo ">> Refreshing desktop/icon caches..."
if [[ "$RUN_SYSTEM" -eq 1 ]]; then
    command -v gtk-update-icon-cache >/dev/null 2>&1 \
        && sudo gtk-update-icon-cache -f -t "$PREFIX/share/icons/hicolor" >/dev/null 2>&1 || true
    command -v update-desktop-database >/dev/null 2>&1 \
        && sudo update-desktop-database "$DESKTOP_DIR" >/dev/null 2>&1 || true
else
    command -v gtk-update-icon-cache >/dev/null 2>&1 \
        && gtk-update-icon-cache -f -t "$ICON_DIR" >/dev/null 2>&1 || true
    command -v update-desktop-database >/dev/null 2>&1 \
        && update-desktop-database "$DESKTOP_DIR" >/dev/null 2>&1 || true
    # Ensure ~/.local/bin is on PATH for convenience.
    if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
        echo
        echo "note: $BIN_DIR is not on your PATH."
        echo "      Add 'export PATH=\"$BIN_DIR:\$PATH\"' to your shell rc,"
        echo "      or run photon via '$BIN_DIR/$BINARY_NAME'."
    fi
fi

echo
echo ">> Done. Run 'photon' to start Photon."
