#!/usr/bin/env bash
# Build victory-wine_<ver>_amd64.deb from a Windows build of victory.exe.
# usage: packaging/deb/build-deb.sh path/to/victory.exe [version]
# The package holds no game files; the user points VICTORY_GAME_DIR at their own dump.
set -euo pipefail
exe=${1:?path to victory.exe}; ver=${2:-0.1.0}
here=$(cd "$(dirname "$0")" && pwd)
root=$(mktemp -d)/victory-wine_${ver}_amd64
install -Dm755 "$exe"            "$root/opt/victory/victory.exe"
install -Dm755 "$here/victory-linux" "$root/usr/bin/victory-linux"
install -Dm644 "$here/../../LICENSE" "$root/usr/share/doc/victory-wine/copyright"
install -Dm644 "$here/../../victory.example.toml" "$root/etc/victory/victory.example.toml"
mkdir -p "$root/DEBIAN"
cat > "$root/DEBIAN/control" <<CTL
Package: victory-wine
Version: $ver
Architecture: amd64
Maintainer: VIctory fork <noreply@example.invalid>
Depends: wine64 | wine, libvulkan1, nvidia-driver-535 | nvidia-driver
Recommends: vkd3d-proton | steam-devices, mesa-vulkan-drivers
Section: games
Priority: optional
Description: VIctory Xbox-title compatibility layer (Windows build run under Wine + vkd3d-proton)
 Interim Linux package: runs the Windows build with the NVIDIA card rendering and
 the motherboard (iGPU) HDMI port scanning out. Native Linux port: docs/linux-port.md.
CTL
dpkg-deb --root-owner-group --build "$root" "$PWD/"
