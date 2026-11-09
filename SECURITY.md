# Crest Browser security

## Phase 1 boundaries

- Chromium's Linux sandbox and TLS certificate verification stay enabled. Crest explicitly enables site-per-process isolation and disallows Chromium browser switches from its application command line. The browser delegates process isolation and web security to the pinned CEF runtime. [Chromium Linux sandbox](https://chromium.googlesource.com/chromium/src/+/main/sandbox/linux/)
- Normal tab navigation accepts HTTP and HTTPS URLs without embedded credentials and the two internal destinations `crest://home/` and `about:blank`. Crest filters website-initiated popups by user gesture and URL before attaching them to a tab.
- The new-tab document and artwork are embedded and display-isolated. Its search form submits to a same-origin route that accepts a user gesture from the active new-tab page and uses the Rust address resolver. Integrated DevTools runs locally through the browser window.
- Site permission callbacks deny requests such as geolocation, notifications, camera, microphone, and screen access. Downloads use a save-location dialog and stay under the user's control.
- The Chromium data directory is rooted under the Crest profile. The Chromium process singleton serializes use of the same engine profile, and Crest writes its local state with a same-directory atomic rename. Invalid state files are reported and preserved. Profile contents are local files protected by the user's OS permissions.

The prototype has one personal browsing context per launched profile. Agent Spaces, credential delegation, package signing, and production security updates require their own design and verification.
