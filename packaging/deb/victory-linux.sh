#!/usr/bin/env bash
# Launch VIctory under Wine with vkd3d-proton on an NVIDIA compute-class card
# (CMP 170HX-class, 10 GB, no display outputs) while the iGPU's HDMI shows the picture.
set -euo pipefail
: "${VICTORY_GAME_DIR:?set to your own dump folder containing game_scarlett_master.exe}"
export WINEPREFIX=${WINEPREFIX:-$HOME/.local/share/victory/prefix}
export WINEARCH=win64 WINEDEBUG=${WINEDEBUG:--all}
# Render on NVIDIA, present through the iGPU (PRIME render offload).
export __NV_PRIME_RENDER_OFFLOAD=1 __GLX_VENDOR_LIBRARY_NAME=nvidia __VK_LAYER_NV_optimus=NVIDIA_only
export VKD3D_FILTER_DEVICE_NAME=${VKD3D_FILTER_DEVICE_NAME:-NVIDIA}
export VKD3D_FEATURE_LEVEL=12_2 VKD3D_CONFIG=${VKD3D_CONFIG:-no_upload_hvv}
# 10 GB card behind a narrow PCIe link, 32 GB RAM: upload ahead, never wait on a texture.
export VICTORY_UPLOAD_AHEAD=${VICTORY_UPLOAD_AHEAD:-1} VICTORY_TEXTURE_NO_WAIT=${VICTORY_TEXTURE_NO_WAIT:-1}
export VICTORY_CACHE_DIR=${VICTORY_CACHE_DIR:-$HOME/.cache/victory}
exec wine /opt/victory/victory.exe run "$(winepath -w "$VICTORY_GAME_DIR")/game_scarlett_master.exe" --console series-s --execute "$@"
