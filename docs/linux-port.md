# Linux: what exists, what is missing

Target box: NVIDIA compute-class card (CMP 170HX style, no display outputs; reported
40 GB — check `nvidia-smi`, a stock CMP 170HX has 8 GB), picture out of the
motherboard's HDMI (iGPU scans out, the NVIDIA card renders).

## Now (interim, in this tree)
- `scripts/cross-check-commits.sh` type-checks any commit range for
  `x86_64-pc-windows-msvc` from Linux, no SDK. HEAD passes (~40 s warm). Needs
  blake3's `pure` feature in the throwaway worktree (its MSVC build wants `ml64.exe`).
- `packaging/deb/build-deb.sh victory.exe` -> `victory-wine_*.deb` (Wine + vkd3d-proton,
  PRIME render offload, iGPU HDMI). Tested as a package; **not run against the title**.
- Full link of `victory.exe` on Linux is **not done**: needs MSVC CRT/SDK libs
  (xwin); this sandbox cannot reach Microsoft's download hosts. Build the exe in CI
  (windows-latest) and feed it to the `.deb` job.

## Native Linux port: size of the job
Files using Win32/D3D12 APIs: gpu 65/92 (88k loc), hle 17/35 (13k), loader 11/16
(4.6k), shader 8/40 (9k), app 1/9. Portable as is: core, formats, game, pe, pipeline,
most of shader (translator logic).

| Windows piece | Linux replacement | Risk |
|---|---|---|
| D3D12 / DXGI (87+50+30 use sites) | Vulkan 1.3 (ash) directly, or link vkd3d-proton as a library | very high; 88k loc |
| VirtualAlloc/Protect (191), MapViewOfFile aliasing (81) | mmap/mprotect, memfd double-mapping | high: title relies on fixed addresses and aliased views |
| Vectored exception handler write guards | SIGSEGV + userfaultfd | high, perf-critical |
| WaitOnAddress, thread context tricks | futex, ptrace-free ucontext | medium |
| `extern "system"` (~590) title ABI | `extern "ms_abi"` trampolines | medium |
| DirectStorage HLE (2.5k loc) | io_uring/pread | low |
| title PE image mapping + TLS slot 0 redirect | own PE mapper exists; TLS via `%gs` fixups | medium |

Order of work: (1) platform trait for memory/threads/exceptions in victory-loader,
(2) Linux loader that runs the PE with ms_abi thunks, (3) `GpuBackend` seam in
victory-gpu, Vulkan backend behind it, (4) hle host-passthrough off kernel32.
Wine/vkd3d-proton remains the fallback while this is built.

## Your box: CMP 170HX-class, 10 GB VRAM, 32 GB DDR5 (corrects the earlier 40 GB assumption)
- No display engine: render on the NVIDIA card, scan out through the motherboard HDMI (the launcher sets PRIME offload).
- The README measured 40-46 GB of private memory for this title on a 64 GB PC. 32 GB RAM + 10 GB VRAM
  is below that: use `--console series-s` (already the launcher default), the `pareto-residency`
  settings (built for 32 GB RAM, see docs/bug-journal.md), and give the Linux box real swap/zram
  (Wine commit is backed by overcommit, but the page cache and the title's mappings still need headroom).
- The PCIe link of this card is narrow, so textures cannot be streamed freely: with 10 GB of VRAM the
  working set will not stay resident. Keep upload-ahead on, but keep per-frame staging small
  (the code already scales with the VRAM budget, and caps staging at 128 MiB a frame on 32 GB RAM).
  The launcher sets `VICTORY_TEXTURE_STAGE_MB=64`; tune it on the card. Expect texture pop-in before frame rate.
- Swap (reported 40 GB) keeps the title from being killed at 40-46 GB; put it on NVMe, not zram.
- FP32 FMA is reportedly throttled on this part (community reports, unverified here): emit separate
  mul+add instead of `Fma` in victory-shader behind a cache-keyed `VICTORY_NO_FMA` switch, after a test on the card.
- No RT cores: keep `-RayTracing` off. Mesh shaders and SM 6.6 through vkd3d-proton on that driver are unconfirmed.
- Not implemented yet: the FMA switch. Nothing here was run on the card.

## Cross-check of the last 40 commits (docs/cross-check-report.txt)
39 pass. `e8d3d918` fails (`host.rs:8721` uses `Presenter.panel`, which does not exist in that
commit): it is not bisectable; its merge `455f4d6d` onward is fine.
