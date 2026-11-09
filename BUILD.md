# Build and test Crest Browser

## Host requirements

- Linux x86-64 and a desktop session. The verified host runs Ubuntu 26.04.1 LTS.
- Rustup with Rust 1.99.0. The workspace pins the toolchain and Cargo resolves dependencies from the lockfile.
- System libraries used by CEF, including GTK 3, GLib, NSS, ALSA, X11 or the corresponding desktop stack, and GBM. Run `ldd target/release/libcef.so` to identify missing libraries.
- A working Chromium sandbox. The verified host permits unprivileged user namespaces. Launch as a regular user.
- A few GiB of free disk. The upstream Linux minimal archive is about 333 MB compressed and the extracted runtime is about 1.5 GB. Debug and release each receive their own runtime copy. [CEF Linux distributions](https://cef-builds.spotifycdn.com/index.html) · [Chromium sandbox on Linux](https://chromium.googlesource.com/chromium/src/+/main/sandbox/linux/)

## Build and launch

```sh
cargo build --locked -j 2
./target/debug/crest-browser

cargo build --locked --release -j 2
./target/release/crest-browser
```

The CEF build dependency downloads the matching Linux distribution and verifies the checksum in the upstream index. The workspace caches it under `target/cef-cache`. Cargo copies the runtime beside the executable and the executable resolves `libcef.so` from that directory. An explicit `CEF_PATH` can point to a compatible cef-rs cache for shared or offline builds. [cef-rs binary setup](https://github.com/tauri-apps/cef-rs/tree/cef-v156.4.0%2B156.0.26)

Keep the executable and CEF resources together when moving a local build.

## Ubuntu package

Run the packager from the project directory. It needs `dpkg-deb`, `dpkg-shlibdeps` from `dpkg-dev`, and Python 3.

```sh
bash packaging/package-linux.sh
```

The package appears as `dist/crest-browser_0.1.0_amd64.deb`. It includes the matching CEF runtime, third-party notices, the Crest icon, and a desktop launcher. The package uses the Chromium user-namespace sandbox. The verified host is Ubuntu 26.04.1 with unprivileged user namespaces available. This is a local developer package.

To try the exact packaged binary without installing it, extract it to a temporary directory.

```sh
crest_package_dir=$(mktemp -d)
dpkg-deb -x dist/crest-browser_0.1.0_amd64.deb "$crest_package_dir"
"$crest_package_dir/usr/bin/crest-browser"
```

## Verification

```sh
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -j 2 -- -D warnings
cargo test --locked --workspace -j 2
python3 tests/smoke.py target/debug/crest-browser
python3 tests/smoke.py target/release/crest-browser
```

The desktop smoke test needs Python 3, Xvfb, xdotool, D-Bus, and a working GTK or GTK portal save dialog. It creates its own display, HTTP fixture, profile, and download directory. It exercises JavaScript execution, navigation, live tab switching, restart and link handoff, bookmarks, layout persistence, a native download, geolocation denial, sandboxed subprocesses, and DevTools and window shutdown.
