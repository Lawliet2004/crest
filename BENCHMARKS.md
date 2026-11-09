# Performance observations

The initial optimized Linux build was measured on the development host on November 9, 2026. Each sample starts a Crest process and waits for its first top-level X11 window to map on an already running Xvfb display. The harness polls every 60 ms and uses software rendering. Each fresh run gets a separate profile, and its paired reuse run restarts with that profile. OS caches were left as-is.

| Case | Samples | Median |
| --- | --- | --- |
| Fresh profile | 323 ms, 261 ms, 210 ms | 261 ms |
| Reused profile | 328 ms, 220 ms, 260 ms | 260 ms |

The optimized Rust executable is about 1.3 MiB. The bundled Chromium and CEF shared library is about 1.38 GiB, and the compressed Ubuntu package is about 337 MiB. These are disk sizes.

The comparative benchmark milestone covers identical Chromium and Firefox workloads, process-tree proportional memory, idle CPU, latency, and frame pacing. Agent workloads enter that suite after Agent Spaces are implemented. [Roadmap](ROADMAP.md)
