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

## Your box: CMP 170HX-class with 40 GB VRAM, 32 GB DDR5 (as stated by the user)
- No display engine: render on the NVIDIA card, scan out through the motherboard HDMI (the launcher sets PRIME offload).
- VRAM is plentiful, system RAM is the tight resource. The README measured 40-46 GB of private memory
  for this title on a 64 GB PC; with 32 GB use `--console series-s` (launcher default) and the
  `pareto-residency` settings (built for 32 GB RAM, docs/bug-journal.md), and give Linux swap on NVMe
  (not zram) so the commit does not hit the limit. The code's >= 20 GiB VRAM tier already applies
  (card buffers up to 8 GiB, 384 MiB staging a frame, capped to 128 MiB by the 32 GB RAM rule).
- The PCIe link of this card is narrow: keep textures resident once uploaded rather than streaming.
  The launcher sets `VICTORY_TEXTURE_STAGE_MB=64` so the first fill is smooth; tune it on the card.
- FP32 FMA is reportedly throttled on this part (community reports, unverified here). Implemented:
  `VICTORY_NO_FMA=1|0` (unset: on when the adapter name contains "CMP") writes every console
  multiply-add as a separate multiply and add (`fmad` in victory-shader), and is part of the shader cache key.
  The emitted float ops carry no fast-math flags (strict in DXIL), so a correct driver should not
  fuse the pair back; unverified on the card. Measure with and without on the card;
  delete the shader cache between runs only if `host_settings` was not changed.
- No RT cores: keep `-RayTracing` off. Mesh shaders and SM 6.6 through vkd3d-proton on that driver are unconfirmed.
- Nothing here was run on the card.

## Cross-check of the last 40 commits (docs/cross-check-report.txt)
39 pass. `e8d3d918` fails (`host.rs:8721` uses `Presenter.panel`, which does not exist in that
commit): it is not bisectable; its merge `455f4d6d` onward is fine.

## CI
`.github/workflows/ci.yml`: `cross-check` (Linux, type-checks for Windows on every push),
`build-exe` (Windows release build) and `deb` (packages that exe as `victory-wine_*.deb`, artifact).

## Belobug merge and the AMD -> NVIDIA map
Merged `Belobug/VIctory-Belobug-AMD` (a-publier): renderer ports 5-7, texture upload in the submission prelude,
frame-rate unlock, output size switches. Conflict choices: Belobug's texture upload and `draw.rs` taken whole (our
movie-plane and synchronous bake uploads are gone: video rings now go through the prelude refresh); residency, XMA refill,
SWEEP_PER_HEAP and FALLBACK_HEAPS stay ON by default (Belobug has them off); our `VICTORY_NO_FMA`, CI and packaging kept;
lane-uniform analysis is Belobug's. Type-checks for Windows, 210 shader tests pass on Linux, **nothing ran on a GPU**.

| AMD-only piece | On NVIDIA |
|---|---|
| RDNA float shortcuts (`set_host_rdna`) | not used; generic code |
| `emulate_amd_only` (CPU `mwaitx`, `rdpru`) | already emulated for any CPU |
| Indirect dispatch widened past 65,535 groups (`indirect::amd`) | not available: D3D12 caps groups per dimension on NVIDIA; large dispatches stay cut (open) |
| FSR 4 upscaler (`VICTORY_FSR_4K`) | off; it needs an RDNA 4 card. DLSS needs NVIDIA's SDK (not added) |
| the five port switches `VICTORY_PORT_CONSTANTS`, `_DESCRIPTORS`, `_GUARD`, `_JOURNAL`, `_CBV` (off in Belobug's builds) | on in our launchers except `PORT_CBV` (GPU-side, try it by hand) |

## DexNom/VIctory (read to 64bd58f, 2026-11-25)
Its history is independent (it starts at an "Initial commit"), so nothing can be merged: commits are picked one by one.
Taken: 76749ce (voxel visibility shader, wave64 mask restored from scalar copies) and ddb7332 (40 shader
variants, integer-target blending off). Already here: 921d8d3, ab34a8b. Not taken: bc3cd4b, 5f4c3df, b8ff22e
(ray-tracing structure building, City texture streaming and a Radeon renderer export; 11-20 conflicting files each,
and the CMP card has no ray-tracing hardware); 4775e8a and 2e01ab4 (tests of code we do not have); 7d73e2b (our cache
already retries temp-file collisions). Its NVIDIA advice (untested by them): ray tracing off (`--no-rt`), FSR 4 off.
