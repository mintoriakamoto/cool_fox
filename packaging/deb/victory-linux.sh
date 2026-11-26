#!/usr/bin/env bash
# Launch VIctory under Wine with vkd3d-proton on an NVIDIA compute-class card
# (CMP 170HX-class, 40 GB, no display outputs) while the iGPU's HDMI shows the picture.
# The numbers below are read from this machine (GPU, VRAM, PCIe link, RAM, swap):
#   victory-linux --print-env     shows what it chose and why, runs nothing.
# Every VICTORY_* value you export yourself wins over the chosen one.
set -euo pipefail

# --- this machine -----------------------------------------------------------
gpu_line=$(nvidia-smi --query-gpu=name,memory.total,pcie.link.gen.current,pcie.link.width.current \
    --format=csv,noheader,nounits 2>/dev/null | grep -iE 'CMP|A100' | head -1 || true)
[ -n "$gpu_line" ] || gpu_line=$(nvidia-smi --query-gpu=name,memory.total,pcie.link.gen.current,pcie.link.width.current \
    --format=csv,noheader,nounits 2>/dev/null | head -1 || true)
IFS=',' read -r gpu_name vram_mb link_gen link_width <<<"${gpu_line:-unknown,0,0,0}"
gpu_name=$(echo "$gpu_name" | xargs); vram_mb=$(echo "${vram_mb:-0}" | xargs)
link_gen=$(echo "${link_gen:-0}" | xargs); link_width=$(echo "${link_width:-0}" | xargs)
ram_mb=$(awk '/MemTotal/ {print int($2/1024)}' /proc/meminfo)
swap_mb=$(awk '/SwapTotal/ {print int($2/1024)}' /proc/meminfo)

# PCIe megabytes a second per lane, by generation; half the link is for textures at 30 frames a second.
case "$link_gen" in 1) lane=250;; 2) lane=500;; 3) lane=985;; 4) lane=1969;; 5) lane=3938;; *) lane=985;; esac
[ "$link_width" -gt 0 ] 2>/dev/null || link_width=4
link_mbs=$((lane * link_width))
stage_mb=$((link_mbs / 2 / 30)); [ "$stage_mb" -lt 16 ] && stage_mb=16; [ "$stage_mb" -gt 128 ] && stage_mb=128
# 32 GB of RAM or less: the title's own 40-46 GB needs the smaller staging blocks.
if [ "$ram_mb" -le 34000 ]; then dstorage_mb=128; else dstorage_mb=256; fi

set_default() { [ -n "${!1+x}" ] || export "$1=$2"; }
set_default VICTORY_TEXTURE_STAGE_MB "$stage_mb"
set_default VICTORY_DSTORAGE_STAGING_MB "$dstorage_mb"
# Upload ahead, never wait on a texture (the link is narrow).
set_default VICTORY_UPLOAD_AHEAD 1; set_default VICTORY_TEXTURE_NO_WAIT 1
# Belobug's renderer ports (untested on NVIDIA; =0 turns one off). PORT_CBV stays off.
for v in VICTORY_PORT_CONSTANTS VICTORY_PORT_DESCRIPTORS VICTORY_PORT_GUARD VICTORY_PORT_JOURNAL; do set_default $v 1; done
set_default VICTORY_FSR_4K 0    # FSR 4 is an AMD RDNA 4 upscaler
set_default VICTORY_CACHE_DIR "$HOME/.cache/victory"

report() {
    echo "gpu: $gpu_name, ${vram_mb} MB VRAM, PCIe gen${link_gen} x${link_width} (~${link_mbs} MB/s)"
    echo "ram: ${ram_mb} MB, swap: ${swap_mb} MB"
    echo "texture staging: ${VICTORY_TEXTURE_STAGE_MB} MiB a frame; DirectStorage staging: ${VICTORY_DSTORAGE_STAGING_MB} MiB"
    [ "$ram_mb" -le 34000 ] && [ "$swap_mb" -lt 16000 ] && \
        echo "WARNING: the title uses 40-46 GB of memory; add 16+ GB of swap on NVMe (not zram)."
    [ "${vram_mb:-0}" -gt 0 ] && [ "$vram_mb" -lt 16000 ] && \
        echo "NOTE: ${vram_mb} MB of VRAM is below what this profile assumes (40 GB)."
    [ "$gpu_name" = unknown ] && echo "WARNING: nvidia-smi found no NVIDIA card; render offload will not work."
    return 0
}
if [ "${1:-}" = "--print-env" ]; then report; env | grep '^VICTORY_' | sort; exit 0; fi
report >&2

: "${VICTORY_GAME_DIR:?set to your own dump folder containing game_scarlett_master.exe}"
export WINEPREFIX=${WINEPREFIX:-$HOME/.local/share/victory/prefix}
export WINEARCH=win64 WINEDEBUG=${WINEDEBUG:--all}
# Render on NVIDIA, present through the iGPU (PRIME render offload).
export __NV_PRIME_RENDER_OFFLOAD=1 __GLX_VENDOR_LIBRARY_NAME=nvidia __VK_LAYER_NV_optimus=NVIDIA_only
export VKD3D_FILTER_DEVICE_NAME=${VKD3D_FILTER_DEVICE_NAME:-NVIDIA}
export VKD3D_FEATURE_LEVEL=12_2 VKD3D_CONFIG=${VKD3D_CONFIG:-no_upload_hvv}
exec wine /opt/victory/victory.exe run "$(winepath -w "$VICTORY_GAME_DIR")/game_scarlett_master.exe" --console series-s --execute "$@"
