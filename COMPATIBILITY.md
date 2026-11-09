# Compatibility

## Browser milestone

| Surface | Status |
| --- | --- |
| Linux x86-64 desktop | Exercised on Ubuntu 26.04.1 under X11. A local amd64 package contains the desktop launcher and CEF runtime. Native Wayland behavior and other distributions need qualification. |
| HTTP and HTTPS | Chromium handles network and page rendering. The browser was exercised against a local HTTP fixture and the public HTTPS site `example.org`. |
| Local profile | Tabs, selected tab, recent history, bookmarks, search provider, and tab layout persist. Chromium maintains the profile's cookies and web storage. |
| Downloads | A native save dialog chooses the path. The activity menu is session-scoped. |
| Developer Tools | The current tab can open an integrated Chromium DevTools window. |
| Website permissions | The current policy blocks requests for sensitive device and site permissions. Per-site grants are a future feature. |

## Google and extensions

Google web account sign-in and signing the browser into Chrome Sync are separate compatibility surfaces. Google restricts browser-level OAuth credentials and prohibits directing Google OAuth through developer-controlled embedded user agents. Gmail, Docs, Meet, Gemini, passkeys, YouTube codecs, and Search still need a site-by-site test matrix. [Chromium API keys and sign-in](https://new.chromium.org/developers/how-tos/api-keys/) · [Google OAuth policies](https://developers.google.com/identity/protocols/oauth2/policies#secure-browsers)

Crest uses CEF Alloy-style views for its custom Rust browser controls. CEF's Chrome-style views expose a broader Chrome UI and extension surface. Manifest V3 extensions, Web Store install and update flows, toolbar actions, and `chrome.identity` require dedicated validation on the eventual production engine. [CEF runtime styles](https://github.com/chromiumembedded/cef/blob/master/docs/architecture.md) · [CEF extension UI discussion](https://github.com/chromiumembedded/cef/issues/3859) · [Chrome extension API reference](https://developer.chrome.com/docs/extensions/reference/api)
