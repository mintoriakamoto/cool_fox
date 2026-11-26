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

## CMP 170HX / big-VRAM specifics (not yet implemented)
- Compute-class GA100: no display engine -> present via iGPU (the launcher sets PRIME offload).
- PCIe link is narrow (x4 class): uploads are the bottleneck, so with 40 GB make the
  texture/upload budgets assume everything stays resident (`texture_floor_bytes`,
  `staging_budget_per_frame`, `default_card_limits_mb` already scale with VRAM budget — add a >=32 GB tier).
- FP32 FMA is throttled on this part (community reports; verify on hardware): emit
  separate mul+add instead of `Fma` in victory-shader (a `VICTORY_NO_FMA` switch,
  cache-keyed like `VICTORY_RDNA_FLOAT`).
- No RT cores: keep `-RayTracing` off; mesh-shader and SM 6.6 support must be confirmed
  through vkd3d-proton on that driver.

## Cross-check of the last 40 commits (docs/cross-check-report.tsv)
39 pass. `e8d3d918` fails (`host.rs:8721` uses `Presenter.panel`, which does not exist in that
commit): it is not bisectable; its merge `455f4d6d` onward is fine.
