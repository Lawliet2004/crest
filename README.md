# Crest Browser

Crest Browser is a Rust-first desktop browser for Linux x86-64. This first build uses Chromium through CEF for web content and Rust for the browser controls and local state.

## Run

From the project directory, build and launch the optimized application.

```sh
cargo build --locked --release -j 2
./target/release/crest-browser
```

The first build downloads the pinned CEF runtime. Keep the executable alongside its runtime files. [Build and test instructions](BUILD.md)

| Task | Control |
| --- | --- |
| Search or enter an address | Address bar or **Ctrl+L** |
| Open or close a tab | **Ctrl+T** or **Ctrl+W** |
| Switch tabs | Click a tab or use **Ctrl+Tab** and **Ctrl+Shift+Tab** |
| Switch between horizontal and vertical tabs | **Menu → Use vertical tabs** or **Use horizontal tabs** |
| Bookmark a website | Star button or **Ctrl+D** |
| Open saved pages | **Library → Bookmarks** or **Recent history** |
| Choose a search provider | **Menu → Search engine** |
| Save a download | Choose a location in the save dialog. **Downloads** shows this session's activity. |
| Inspect a page | **Menu → Developer Tools** or **Ctrl+Shift+I** |

Crest restores the selected page and the open tab list on restart. The browser saves local state under `${XDG_DATA_HOME:-$HOME/.local/share}/crest-browser`. For a disposable profile, run `./target/release/crest-browser --profile "$(mktemp -d)"`. A URL argument opens a tab, including when the profile is already running.

To build the local Ubuntu amd64 package with its desktop launcher, run `bash packaging/package-linux.sh`. [Package instructions](BUILD.md#ubuntu-package)

Google account flows, Chrome extensions, and Agent Spaces are later milestones. [Development prompt](CREST_BROWSER_PROMPT.md) · [Roadmap](ROADMAP.md) · [Compatibility](COMPATIBILITY.md)
