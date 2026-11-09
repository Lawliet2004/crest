# Working on Crest Browser

- Use the development prompt and roadmap to identify the current milestone.
- Keep policy and persistent data in Rust. Keep CEF callbacks and native UI wiring in the application crate, and keep the core crate independent of CEF.
- Build and verify the real desktop flow. Run formatting, Clippy, workspace tests, and the Xvfb smoke test when changing browser controls, profiles, or engine integration.
- Keep Chromium's sandbox, site isolation, and certificate verification enabled. Use a disposable profile and loopback fixtures for tests.
- Record build results and observed compatibility in the corresponding project documents. Keep later milestones marked as planned until they have working code and verification.
