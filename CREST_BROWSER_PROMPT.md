# MASTER DEVELOPMENT PROMPT
## Project Crest Browser — A Rust-First, AI-Native, Ultra-Efficient Web Browser

You are acting as an elite multidisciplinary browser engineering team with expertise in:

- Rust systems engineering and asynchronous programming
- Chromium architecture, browser process models, Blink and V8
- Browser security, authentication and privacy
- Modern desktop UI/UX and GPU-accelerated interfaces
- Chrome extension APIs and Manifest V3
- Model Context Protocol (MCP), Chrome DevTools Protocol (CDP), WebDriver BiDi and browser automation
- AI agent orchestration and isolation
- Linux desktop integration and performance optimization
- Automated testing, benchmarking and software architecture

Your assignment is to **research, architect, implement, test and progressively deliver a functional desktop browser called Crest Browser.**

This is not a landing-page prototype, a simulated browser interface or a WebView demonstration.

The eventual product must be a real, general-purpose browser with a distinctive AI-native architecture.

**Primary platform:** Linux, especially Ubuntu/Kubuntu, x86-64.

**Primary implementation language:** Rust.

**Rendering strategy:** Chromium-compatible production architecture, while keeping custom browser components Rust-first.

**Defining innovation:** Isolated, persistent AI Agent Spaces that can operate independently from the user's personal browser workspace.

**Product philosophy:** Beautiful by default, powerful when needed, lightweight by design.

---

# 1. NON-NEGOTIABLE ENGINEERING PRINCIPLES

## 1.1 Rust-first development

Use Rust for virtually every newly authored subsystem where technically practical:

- Browser application state
- Tab and workspace orchestration
- Native UI state and widgets
- Settings and profiles
- Session persistence
- History and bookmarks
- Downloads management
- Search engine management
- Resource scheduling
- Agent workspace orchestration
- Permission enforcement
- MCP services and automation routing
- Local databases and configuration
- Extension and widget management interfaces
- Performance monitoring and telemetry
- Security-sensitive application services

Use existing C++ Chromium components where necessary for production-grade rendering, JavaScript, extensions, sandboxing, networking and browser compatibility.

**Do not rewrite mature Chromium subsystems solely to increase the Rust percentage.**

Evaluate the official Chromium Rust build integration, including GN Rust targets and supported FFI mechanisms.

Maintain a source-code inventory that reports:

1. Percentage of newly written Crest application code authored in Rust.
2. Percentage of all included code authored in Rust, including the engine.
3. Non-Rust dependencies and why they remain necessary.

Do not claim the full browser is mostly Rust if the integrated Chromium engine makes that statement false.

## 1.2 Performance before gimmicks

Rust alone does not guarantee speed or low memory consumption.

Every performance decision must be evaluated through benchmarks.

Optimize:

- Cold startup
- Warm startup
- Idle CPU
- Idle RAM
- RAM per active tab
- Background tab overhead
- Scrolling smoothness
- Input latency
- Tab switching responsiveness
- Browser UI frame pacing
- Battery consumption
- Memory under many Agent Spaces
- Session restoration speed

Avoid introducing Electron or an additional large browser engine merely to render Crest's interface.

Do not disable browser sandboxing, site isolation, TLS verification or critical security features to improve benchmarks.

## 1.3 Build real functionality

Do not produce an impressive-looking UI whose controls are nonfunctional.

Every implemented feature must have:

- Working code
- Integration tests where practical
- Error handling
- Persistence when relevant
- Documentation
- A clearly defined completion status

No placeholders may be represented as completed features.

---

# 2. RESEARCH AND ARCHITECTURE PHASE

Before major implementation, inspect the latest maintained source code and documentation for:

### Browser engines and foundations

- Chromium: https://chromium.googlesource.com/chromium/src
- Servo: https://github.com/servo/servo
- Brave: https://github.com/brave/brave-browser
- BrowserOS: https://github.com/browseros-ai/BrowserOS
- Chromium Embedded Framework: https://github.com/chromiumembedded/cef
- Rust CEF bindings: https://github.com/tauri-apps/cef-rs

### Agent automation and browser debugging

- Chrome DevTools MCP: https://github.com/ChromeDevTools/chrome-devtools-mcp
- Playwright: https://github.com/microsoft/playwright
- Playwright MCP: https://github.com/microsoft/playwright-mcp
- Official Rust MCP SDK: https://github.com/modelcontextprotocol/rust-sdk
- Chrome DevTools Protocol: https://chromedevtools.github.io/devtools-protocol/
- WebDriver BiDi: https://w3c.github.io/webdriver-bidi/

### Rust GUI and performance

- GPUI: https://github.com/zed-industries/zed
- egui: https://github.com/emilk/egui
- winit: https://github.com/rust-windowing/winit
- Tokio: https://github.com/tokio-rs/tokio
- tracing: https://github.com/tokio-rs/tracing

### Browser customization

- Chrome Extensions: https://developer.chrome.com/docs/extensions/
- Chrome Web Store documentation: https://developer.chrome.com/docs/webstore/
- Mue dashboard: https://github.com/mue/mue
- Arc, Zen, Vivaldi and Firefox for interaction and UI inspiration

Study architecture and implementation patterns, not just screenshots.

Verify each project's license, maintenance status, platform support and compatibility.

Do not blindly copy incompatible open-source code.

Produce an Architecture Decision Record comparing:

**Option A:** Maintainable Chromium fork with custom Rust services.

**Option B:** Rust GUI using CEF or cef-rs for rapid prototyping.

**Option C:** Servo-based experimental browser.

Evaluate each against:

- Google authentication
- Modern Google apps
- Chrome extension support
- Browser developer tools
- Multi-profile isolation
- Agent automation
- Linux packaging
- Rust integration
- Memory efficiency
- Maintenance burden
- Upstream security updates

**Preferred production candidate: Option A.**

Option B is acceptable for an early prototype only after clearly documenting that Google login, full extensions and browser functionality may not work.

Option C may be developed as a separate research backend, not a substitute for production compatibility.

Record the decision before proceeding.

---

# 3. CORE PRODUCT ARCHITECTURE

Design Crest around the following logical components:

```text
CREST BROWSER
│
├── Native Browser Interface
│   ├── Horizontal Tabs
│   ├── Vertical Tabs
│   ├── Navigation Toolbar
│   ├── Command Palette
│   ├── Home Dashboard
│   ├── Split View
│   ├── Settings
│   └── Extension Interface
│
├── Rust Browser Services
│   ├── Tab Manager
│   ├── Workspace Manager
│   ├── Profile Manager
│   ├── History / Bookmarks
│   ├── Download Manager
│   ├── Search Provider Manager
│   ├── Permission Broker
│   ├── Resource Scheduler
│   ├── Widget Runtime
│   └── Local Storage Services
│
├── Agent Control Plane — Rust
│   ├── MCP Server
│   ├── CDP / BiDi Adapter
│   ├── Agent Session Registry
│   ├── Agent Space Manager
│   ├── Task Scheduler
│   ├── Credential Authorization
│   ├── Activity Recorder
│   ├── Approval System
│   └── Isolation Enforcement
│
├── Chromium Engine
│   ├── Blink
│   ├── V8
│   ├── Network Stack
│   ├── Extension Platform
│   ├── DevTools
│   ├── Media / WebRTC
│   ├── Browser Sandboxing
│   └── Site Isolation
│
└── Linux Integration
    ├── GPU / Window System
    ├── KWallet / Secret Service
    ├── Filesystem
    ├── Process Monitoring
    ├── cgroup v2
    └── Desktop Notifications
```

These are logical boundaries, not necessarily one operating-system process per component.

Use a well-defined IPC and FFI design.

Never expose raw C++ pointers to untrusted plugin or agent code.

Implement versioned interfaces between major subsystems.

---

# 4. VISUAL DESIGN — A PREMIUM MODERN BROWSER

Crest must look like a polished consumer product from its first usable release.

Do not create a generic developer dashboard with basic buttons.

Follow these principles:

- Clean, spacious and highly intentional composition
- Modern typography
- Subtle rounded corners
- Consistent spacing
- Restrained animations
- Strong visual hierarchy
- Excellent dark and light themes
- Keyboard-first interactions
- Accessible contrast and focus indicators
- High-DPI support
- Responsive window resizing
- GPU-efficient transitions

Use Arc, Zen, Vivaldi and modern productivity applications as design references, but develop an original visual identity.

### Browser chrome

Implement:

- Unified address/search bar
- Back, forward, reload and stop
- Bookmark button
- Downloads button
- Extension toolbar
- Profile switcher
- Space switcher
- Compact settings menu
- Omnibox suggestions
- Keyboard navigation
- Full-screen mode
- Optional minimal/distraction-free mode

### Vertical and horizontal tabs

Both must be first-class supported modes.

Horizontal tabs:

- Conventional tab strip
- Favicons and titles
- Pinned tabs
- Tab groups
- Drag-and-drop ordering
- Close and mute controls
- Overflow handling
- Tab search

Vertical tabs:

- Collapsible sidebar
- Adjustable width
- Compact icon-only mode
- Pinned favorites
- Nested or grouped tabs
- Workspace sections
- Smooth transition between layouts

Add a setting to switch between horizontal and vertical tabs at any time.

Switching must preserve the user's open pages, scroll position and tab state.

Do not recreate web renderers unnecessarily during layout transitions.

### Workspace customization

Allow the user to:

- Create multiple personal workspaces
- Name and color them
- Choose icons
- Pin favorite tabs
- Assign keyboard shortcuts
- Choose independent tab layouts
- Restore prior sessions
- Configure workspace-specific home screens

The interface must not interfere with the engine's security boundaries.

---

# 5. EXCEPTIONAL CUSTOMIZABLE HOME SCREEN

The initial new-tab page should be beautiful, polished and useful immediately after installation.

Create an elegant default dashboard with:

- Large, optional clock
- Date
- Personalized greeting
- Search bar
- Quick website shortcuts
- Recently visited websites
- Pinned favorites
- Agent Space status widget
- Custom background
- Quick access to settings

It should resemble a premium, carefully designed productivity dashboard rather than a crowded collection of boxes.

### Native widget system

Implement a widget framework supporting:

- Digital clock
- Analog clock
- Multiple world clocks
- Calendar
- Weather
- Search
- Quick links
- Notes
- To-do list
- Bookmarks
- Recently visited sites
- Download status
- CPU and RAM monitoring
- Agent task monitoring
- GitHub activity
- Development-server status
- Custom widgets

Support:

- Drag-and-drop positioning
- Resizable widgets
- Grid snapping
- Multiple dashboard pages or layouts
- Light/dark widget themes
- Widget-specific settings
- Widget import/export
- Widget disable/remove
- Local persistence

Build native widgets primarily in Rust.

### Custom search engines

Support Google, DuckDuckGo, Bing, Brave Search and custom providers.

Implement:

- Default engine selection
- Search engine shortcuts
- User-defined URL templates
- Private-search shortcuts
- Address-bar keyword search
- Optional engine icons

Use properly encoded query parameters.

Do not send user keystrokes to remote suggestion providers without an appropriate privacy setting.

### Home screen efficiency

Widgets should consume almost no CPU while idle.

Use event-driven updates.

Pause expensive work when the dashboard is not visible.

Cache weather and network-driven widget responses.

Avoid multiple independent timers when a shared scheduler can perform the same work.

---

# 6. AGENT-GENERATED WIDGETS, MODS AND EXTENSIONS

One of Crest's defining capabilities must be **AI-assisted browser customization**.

Codex, Antigravity, Claude Code and other compatible agents should be able to develop and install custom browser functionality with user approval.

Examples:

- Create a custom clock
- Build a GitHub widget
- Make a development dashboard
- Create a productivity sidebar
- Add a custom keyboard shortcut
- Build a new-tab theme
- Develop a browser extension
- Create a task-specific Agent Space layout

Implement a documented Crest Developer SDK.

### Supported customization types

**Type A: Native declarative widgets**

Use a versioned widget manifest containing:

- Widget ID
- Name
- Version
- Entry point or component type
- Layout constraints
- Settings schema
- Required permissions
- Supported APIs

Provide Rust-backed widget services.

Allow declarative layouts and configurable native components.

**Type B: Sandboxed custom widgets**

Support a constrained WebAssembly or sandboxed web-content runtime where appropriate.

Do not load arbitrary untrusted native dynamic libraries directly into the privileged browser process.

Enforce resource limits and narrowly scoped capabilities.

**Type C: Standard browser extensions**

Support compatible Manifest V3 Chrome extensions through Chromium's extension system.

Expose documented Crest extension APIs separately.

Keep compatibility with standard extension behavior wherever possible.

### Agent-created customization workflow

1. Agent receives a customization request.
2. Agent reads Crest's SDK documentation.
3. Agent creates a widget or extension project.
4. Agent runs compilation and validation.
5. Crest displays requested permissions.
6. User reviews and approves installation.
7. Crest installs the component.
8. Agent tests the component in an isolated development profile.
9. User can disable, rollback or uninstall it.

Provide preview, rollback and version history.

Agents must not silently gain new permissions through generated plugins.

---

# 7. GOOGLE ECOSYSTEM COMPATIBILITY

Google compatibility is a major product requirement.

Create an automated compatibility matrix for:

### Google applications

- Google Search
- Google Search AI Mode, where available
- Gemini
- Gmail
- Google Drive
- Google Docs
- Google Sheets
- Google Slides
- Google Forms
- Google Calendar
- Google Meet
- Google Maps
- YouTube
- Google Photos
- Google Keep
- Google Translate
- Google Contacts
- Google Classroom
- Google account management
- Google Password Manager website

Test real application behavior, not merely successful page loading.

Examples of required tests:

**Gmail:** login, inbox navigation, compose, send in a controlled test account, attachments.

**Docs:** open, edit, save, clipboard, keyboard shortcuts, offline behavior where supported.

**Meet:** microphone, camera, device permissions, WebRTC and screen sharing.

**YouTube:** playback, audio, fullscreen, captions and supported codecs.

**Gemini:** prompts, streaming responses, attachments and interface interactions.

**AI Mode:** access and interaction for eligible accounts and regions.

### Authentication

Support standards-compliant web authentication:

- Cookies
- SameSite policies
- TLS
- WebAuthn
- Passkeys where supported
- OAuth 2.0 / OpenID Connect where appropriate
- Secure storage

Do not claim complete Google Chrome integration merely because an application loads.

Google can block embedded browsers and automation-controlled sessions.

Follow Google's supported browser and OAuth policies.

Never disguise the browser to bypass security controls.

### Google Password Manager

Separate the following concepts:

1. Opening the Google Password Manager website.
2. Importing passwords through officially supported user-controlled mechanisms.
3. Crest's own password vault and autofill.
4. Google's Chrome-specific password synchronization.

Build Crest's secure password-management foundation using vetted storage mechanisms and platform credential services.

Never claim access to Google's proprietary password-sync backend without an authorized supported API.

Never scrape passwords from Google services.

### Sync

Build an independent optional Crest synchronization system for:

- Bookmarks
- Settings
- Themes
- Home screen layouts
- Non-sensitive workspace metadata

Do not rely on Google's private Chrome Sync APIs.

Do not put secrets into ordinary synchronization payloads.

---

# 8. CHROME WEB STORE AND EXTENSIONS

Users should be able to use widely adopted Chrome extensions, subject to compatibility and distribution restrictions.

Retain Chromium's mature extension architecture where practical.

Prioritize Manifest V3.

Support applicable:

- Extension service workers
- Content scripts
- Extension popup UI
- Browser action controls
- Storage APIs
- Tabs APIs
- Runtime messaging
- Context menus
- Declarative network request APIs
- Permissions and host permissions
- Extension options pages
- New-tab overrides

Add a native extension manager for:

- Enable/disable
- Remove
- View permissions
- Manage site access
- Pin to toolbar
- Inspect extension errors
- Update installed extensions
- Separate extension policies for Agent Spaces

Investigate Chrome Web Store installation and update mechanisms, including current legal, technical and platform restrictions.

Do not assume all Chrome-exclusive APIs or store distribution workflows are available in Chromium forks.

Use a verified compatibility strategy and provide supported fallback installation mechanisms when permitted.

Never bypass store authorization or silently install extensions.

### New-tab extension conflicts

If an extension wants to replace Crest's home screen, ask the user.

Allow switching between:

- Crest native dashboard
- Extension-provided new-tab page
- Blank tab
- Custom URL

---

# 9. THE MAIN INNOVATION — AGENT SPACES

This is the most important differentiator.

Crest must separate personal browsing from AI-driven browsing.

## 9.1 My Space

The user's normal browser environment.

It contains:

- Personal tabs
- Bookmarks
- Saved sessions
- Password manager
- Installed extensions
- Browsing history
- Google accounts
- Personal widgets and settings

Agents must not automatically gain access to My Space.

## 9.2 Agent Spaces

Each AI agent can request an isolated workspace.

For example:

**Agent Space A — Codex**

- Tests a local React website
- Opens multiple tabs
- Inspects console errors
- Tests responsive layouts
- Submits controlled test forms

**Agent Space B — Claude Code**

- Tests documentation
- Navigates API references
- Inspects accessibility
- Generates screenshots and reports

**Agent Space C — Antigravity**

- Tests a newly developed web application
- Checks login workflows
- Inspects network requests
- Runs browser automation scripts

These environments must remain independent from My Space.

## 9.3 Workspace isolation

Each persistent Agent Space requires:

- Unique workspace ID
- Dedicated tab registry
- Separate storage partition or browser profile
- Isolated cookies and local storage
- Independent session lifecycle
- Permission policy
- Resource budget
- Activity log
- Independent downloads area
- Extension policy
- Persistent or ephemeral mode

Where strong isolation is required, use separate Chromium profiles or processes, supplemented by OS-level sandboxing.

Do not treat tab grouping alone as security isolation.

Protect all agent-control connections with explicit workspace authorization.

## 9.4 Background operation

Agents must continue working when the user switches back to My Space.

They must not:

- Steal keyboard focus
- Move the user's mouse pointer
- Replace the current tab
- Open windows in front of personal work
- Access personal tabs without authorization
- Interrupt user typing
- Close unrelated sessions

Use headless or background-capable rendering modes where feasible.

Allow users to enter an Agent Space and observe its live state.

When an agent requires human interaction, send a nonintrusive notification.

## 9.5 Human observation and control

Each Agent Space needs a monitoring panel with:

- Agent name
- Current task
- Active tabs
- Current URL
- Task timeline
- Screenshots or live view
- Console output
- Network logs where authorized
- Resource usage
- Permission requests
- Last completed action
- Errors
- Pause/resume
- Stop
- Revoke access
- Delete workspace

Support user takeover where necessary.

Provide clear labeling when an agent, rather than a human, controls the session.

---

# 10. AGENT COMPATIBILITY — MCP FIRST

Crest should support external agents, including:

- OpenAI Codex
- Claude Code
- Google Antigravity
- Cursor
- VS Code-compatible MCP clients
- OpenCode
- Other standards-compliant agent frameworks

Do not hardcode dependencies on one AI model vendor.

### MCP server

Build an official Crest MCP server primarily in Rust, using the supported Rust MCP SDK.

Implement supported transports such as stdio and secured local HTTP, based on the current MCP specification.

Expose agent tools such as:

```text
crest.list_spaces
crest.create_space
crest.delete_space
crest.get_space
crest.list_tabs
crest.open_tab
crest.navigate
crest.get_accessibility_tree
crest.click
crest.type
crest.fill_form
crest.select_option
crest.screenshot
crest.evaluate_javascript
crest.get_console_logs
crest.get_network_events
crest.inspect_element
crest.start_trace
crest.stop_trace
crest.get_task_status
crest.pause_task
crest.resume_task
crest.request_permission
crest.create_widget_project
crest.validate_widget
crest.preview_widget
crest.request_widget_install
```

Tool names are proposed Crest APIs, not claims that these tools already exist.

Define typed JSON schemas for each tool.

Attach a workspace-scoped capability token to authorized operations.

Include stable IDs, structured errors, timeouts and cancellation support.

### CDP and browser debugging

Expose controlled Chrome DevTools Protocol access to authorized Agent Spaces.

Investigate integration with Chrome DevTools MCP and Playwright MCP.

Never expose unrestricted browser-wide remote debugging that could inspect My Space.

Bind local endpoints securely.

Do not expose CDP ports to the public network by default.

### Agent discovery

Create an onboarding panel that helps users connect external coding agents.

Generate current configuration examples for supported clients.

Detect installed tools only with user approval.

Include installation and troubleshooting instructions.

Do not invent configuration paths or command-line flags. Verify them against each agent's current documentation.

### Direct CLI and SDK

Provide:

- Rust client library
- CLI interface
- MCP interface
- JSON-schema tool definitions
- Example projects
- Agent skills/documentation
- Integration tests

This allows a coding assistant to use Crest for website testing without requiring a special proprietary agent plugin.

---

# 11. AGENT CREDENTIALS, LOGIN AND OAUTH

Agent Spaces may need authenticated access to development websites, dashboards and permitted third-party services.

Build a secure authorization workflow.

### Preferred workflow

1. User creates an Agent Space.
2. User selects the websites and permissions the agent can use.
3. User signs in interactively where needed.
4. The authenticated session remains in that Agent Space.
5. The agent operates within the approved scope.
6. The user can revoke access or delete the workspace.

Use official OAuth flows and scoped tokens where available.

Use OS-supported secret storage for credentials.

Do not expose raw passwords to models by default.

Avoid transmitting session cookies to external AI providers.

Do not duplicate personal-profile credentials automatically.

Separate permission to read a website from permission to modify, upload, delete or submit information.

Treat sending messages, making purchases, publishing content, modifying production systems and submitting consequential forms as high-impact actions requiring appropriate approval.

Never bypass CAPTCHA, MFA or third-party account security mechanisms.

Acknowledge that isolated browser profiles cannot prevent conflicts when agents modify the same remote account or document.

---

# 12. MULTI-AGENT RESOURCE MANAGEMENT

Crest should support multiple concurrent Agent Spaces while prioritizing the user's interactive experience.

Create a Rust resource scheduler.

### Scheduling policy

Priority order:

1. Personal foreground interaction
2. Active media and critical user sessions
3. Agent tasks requiring timely execution
4. Background agent operations
5. Dormant sessions and cached pages

### Optimizations

Implement, where technically feasible:

- Lazy tab creation
- Tab discard and restoration
- Process reuse consistent with security isolation
- Bounded agent concurrency
- Reduced background rendering
- Memory-pressure response
- Shared caches without shared credentials
- Reduced timer activity
- Deferred nonessential work
- On-demand screenshots
- Adaptive task scheduling
- Linux cgroup-based CPU/memory limits where appropriate

Do not suspend tabs currently executing necessary agent operations.

A browser tab running JavaScript, processing media or completing a form may require substantial resources. Do not promise near-zero RAM for actively executing tasks.

### Resource settings

Offer:

**Eco Mode**

Conservative parallelism, aggressive suspension of idle sessions, low background priority.

**Balanced Mode**

Normal parallelism, user browsing prioritized.

**Performance Mode**

Higher concurrency, subject to available hardware.

Automatically adapt to total available RAM and CPU cores.

Avoid arbitrary fixed memory limits that destabilize Chromium.

### Agent session restoration

Persist:

- Workspace metadata
- Tab URLs
- Selected non-sensitive state
- Task checkpoints
- Permissions
- Execution status

Restore only what can safely be restored.

Clearly distinguish resumed sessions from restarted tasks.

---

# 13. DEVELOPER TOOLS

Crest must be suitable for serious frontend and web application development.

Retain and expose Chromium DevTools functionality, including:

- DOM inspector
- CSS inspector
- JavaScript console
- Network panel
- Sources debugger
- Breakpoints
- Performance profiler
- Memory profiler
- Responsive design mode
- Accessibility inspection
- Storage inspection
- Security inspection
- Console error reporting

Add an agent-oriented developer workflow.

Example:

A developer starts a local website at localhost:3000.

They right-click and choose:

"Test this website with an agent."

Crest creates an isolated Agent Space.

The agent then:

1. Opens the website.
2. Tests navigation.
3. Checks console and network errors.
4. Performs accessibility checks.
5. Tests responsive layouts.
6. Takes screenshots.
7. Runs approved form workflows.
8. Produces a structured report.

For code changes, an external coding agent may then inspect the repository and fix problems through its separately authorized development environment.

Browser automation permissions must not implicitly grant filesystem or shell access.

---

# 14. SECURITY ARCHITECTURE

Treat webpages, extension code, downloaded files and agent output as untrusted.

Threat-model:

- Prompt injection from webpages
- Malicious browser extensions
- Credential theft
- Session hijacking
- Local MCP endpoint attacks
- Cross-workspace data leakage
- Arbitrary JavaScript execution
- Unsafe widget plugins
- Agent authorization bypasses
- Malicious downloaded files
- Privileged browser-process compromise

Requirements:

- Maintain Chromium sandboxing and site isolation.
- Keep secrets out of logs.
- Require explicit permission for sensitive operations.
- Use origin and capability checks.
- Implement auditable agent actions.
- Separate privileged browser code from untrusted renderers.
- Protect against cross-site request forgery on local control endpoints.
- Prevent unauthorized local clients from accessing MCP/CDP.
- Validate extension and widget manifests.
- Support reliable permission revocation.
- Never silently expose personal browser sessions to agents.
- Use least-privilege access for all automation tools.

Security testing must cover both the personal workspace and multiple Agent Spaces.

---

# 15. PERFORMANCE BENCHMARKING

Create repeatable benchmarks comparing Crest against installed Chromium and Firefox versions.

Use identical test sites, hardware, system state and test conditions.

Benchmarks should include:

- Browser cold startup
- Warm startup
- One blank tab
- Five normal websites
- Ten normal websites
- Twenty normal websites
- Multiple Google applications
- Three Agent Spaces
- Multiple active agents
- Restored dormant sessions
- Scrolling stress tests
- Video playback
- Extension-heavy workloads

Record:

- Total browser process-tree RSS
- Proportional set size where available
- CPU usage
- GPU usage where measurable
- Frame pacing
- Startup latency
- Tab-switch latency
- Energy consumption where supported

Distinguish per-process RSS from true total memory consumption.

Do not double-count shared memory pages.

Performance improvements must be presented with reproducible measurements, not marketing percentages.

Do not silently cripple websites, security or active agent tasks to win benchmarks.

---

# 16. LINUX DESKTOP INTEGRATION

Target Kubuntu first.

Support:

- KDE Plasma
- X11 and Wayland as feasible
- High-DPI scaling
- Native notifications
- File chooser
- Clipboard
- Desktop shortcuts
- Default browser registration
- Downloads folders
- KWallet or Secret Service
- Microphone and camera permissions
- Screen sharing through appropriate Linux portals
- App icon and desktop entry
- Proper application shutdown
- Crash recovery

Provide reliable development and release builds.

Use a native Linux package initially; evaluate AppImage, DEB and Flatpak later.

Never assume the machine has enough resources for a full Chromium build.

Check the available hardware, disk, memory and compiler toolchain first.

If local compilation is infeasible, provide an honest alternate build strategy such as a compatible CI runner.

Do not disable the sandbox to make packaging easier.

---

# 17. REPOSITORY ORGANIZATION

Begin with a Rust workspace and clearly defined components.

A possible repository structure:

```text
crest/
├── README.md
├── LICENSE
├── Cargo.toml
├── rust-toolchain.toml
├── crates/
│   ├── crest-app/
│   ├── crest-core/
│   ├── crest-tabs/
│   ├── crest-workspaces/
│   ├── crest-profiles/
│   ├── crest-ui/
│   ├── crest-widgets/
│   ├── crest-settings/
│   ├── crest-storage/
│   ├── crest-security/
│   ├── crest-resources/
│   ├── crest-agent/
│   ├── crest-mcp/
│   └── crest-automation/
├── engine/
│   ├── chromium-integration/
│   └── servo-experimental/
├── sdk/
│   ├── widgets/
│   ├── extensions/
│   └── agent-clients/
├── tests/
│   ├── integration/
│   ├── security/
│   ├── google-compat/
│   ├── extensions/
│   ├── agent-isolation/
│   └── performance/
├── benchmarks/
├── examples/
├── packaging/
├── docs/
└── scripts/
```

Adapt this structure to the selected engine's real build system.

Do not assume Chromium's GN/Ninja build can be replaced with Cargo.

Maintain a clear build boundary between Rust services and Chromium-specific source integration.

---

# 18. DEVELOPMENT PHASES

Do not attempt to implement the entire product in one code-generation pass.

Use milestone-based delivery.

## Phase 0 — Feasibility and architecture

Deliver:

- Hardware assessment
- Current dependency research
- License review
- Engine decision record
- Rust/C++ integration proof of concept
- Authentication risk assessment
- Extension compatibility assessment
- Architecture documentation
- Build plan

**Exit condition:** The engine strategy is technically validated, not just proposed.

## Phase 1 — Working desktop browser

Deliver:

- Launchable Linux application
- Functional browser engine
- Address bar
- Navigation
- Multiple tabs
- Vertical/horizontal tab switching
- Basic history
- Bookmarks
- Downloads
- Persistent settings
- DevTools access
- Basic security permissions

**Exit condition:** Real websites can be navigated and normal browsing works.

## Phase 2 — Premium user experience

Deliver:

- Polished dark/light themes
- Customizable dashboard
- Native widgets
- Workspace switching
- Keyboard shortcuts
- Command palette
- Tab search
- Refined animations
- Accessibility

**Exit condition:** The browser looks and behaves like a coherent desktop product.

## Phase 3 — Google and extension compatibility

Deliver:

- Google website test matrix
- Account sign-in verification
- Gemini and Search testing
- Google Workspace testing
- Media/WebRTC verification
- Extension manager
- Manifest V3 compatibility testing
- New-tab override support

**Exit condition:** Core compatibility tests pass, with unsupported proprietary features accurately documented.

## Phase 4 — Agent Spaces

Deliver:

- Separate agent profiles
- Independent tabs
- Workspace permissions
- Background execution
- Agent dashboard
- Activity logs
- Pause/resume/stop
- Resource budgets
- Authenticated session workflow

**Exit condition:** Two independent agents can use separate spaces while personal browsing remains unaffected.

## Phase 5 — External agent integration

Deliver:

- Rust MCP server
- Client onboarding
- CDP integration
- Playwright-compatible automation path
- Structured accessibility snapshots
- Website testing tools
- Form automation
- Agent status reporting
- Scoped permission system

**Exit condition:** Compatible external coding agents can control only their authorized spaces.

## Phase 6 — Agent-generated customizations

Deliver:

- Widget SDK
- Widget manifests
- Widget validation
- Agent-generated widget creation
- Preview and installation approval
- Extension development workflow
- Rollback functionality

**Exit condition:** A compatible coding agent can create, test and install a working custom home-screen widget with user approval.

## Phase 7 — Performance and hardening

Deliver:

- Resource scheduler
- Tab sleeping
- Memory-pressure management
- Linux resource controls
- Security review
- Crash recovery
- Regression tests
- Measured benchmarks
- Release packaging

**Exit condition:** Improvements are measurable and do not compromise compatibility or security.

---

# 19. REQUIRED END-TO-END ACCEPTANCE SCENARIOS

The project is not complete until the following scenarios have been tested.

### Scenario A: Normal browsing

Open Crest, search with Google, navigate to several sites, bookmark pages, download a file and restart while restoring the prior session.

### Scenario B: Google services

Use an authorized test account to open Gmail, Docs, Drive and Gemini. Verify their essential functionality and document unsupported capabilities.

### Scenario C: Extensions

Install a compatible Manifest V3 extension through a supported installation path, configure permissions, restart the browser and verify the extension remains functional.

### Scenario D: Vertical tabs

Open several pages, switch to vertical tabs, rearrange tabs, collapse the sidebar and switch back without losing session state.

### Scenario E: Two agents

Connect two independent MCP clients.

Give each a different Agent Space.

Run unrelated browser automation tasks.

Verify that each agent sees only its own authorized tabs and storage.

### Scenario F: Personal workspace protection

Keep a personal editing session open.

Run an agent website test in the background.

Verify no unexpected focus changes, personal tab access or workspace interference.

### Scenario G: Authenticated agent task

Manually authenticate a dedicated Agent Space with a test account.

Permit an agent to perform an approved workflow.

Revoke the agent's access and verify further automation is rejected.

### Scenario H: Agent-created widget

Ask a connected coding agent to create a custom clock or GitHub widget.

The agent must generate, compile, preview and validate the widget.

Crest then asks the user before installation.

### Scenario I: Resource stress

Create multiple Agent Spaces with several tabs.

Measure memory and CPU consumption.

Verify personal browsing remains responsive.

### Scenario J: Security

Attempt unauthorized cross-space access and confirm denial.

Test malformed MCP requests, unauthorized connections and malicious widget permissions.

---

# 20. DEVELOPMENT QUALITY RULES

Use:

- Stable Rust where practical
- Strict formatting and linting
- Meaningful unit tests
- Integration tests
- Reproducible builds
- Minimal dependency duplication
- Version pinning
- Structured logging
- Automated CI
- Security scanning
- Clear error messages
- Documented APIs

Keep Chromium updated with upstream security patches.

Do not fork large repositories unnecessarily if a smaller integration is sufficient.

Never use `unsafe` Rust without documented justification and review.

Never fabricate successful compilation, benchmarks or compatibility results.

If a requested capability is blocked by proprietary APIs or platform policy, record it transparently and implement the best authorized alternative.

---

# 21. HOW YOU MUST EXECUTE THIS TASK

You are not being asked merely to explain how to build Crest.

You must actually implement the project in the available development environment.

Start by inspecting the current environment and repository.

Then:

1. Research and verify the dependencies.
2. Produce the architecture decision.
3. Scaffold the project.
4. Implement the smallest functional browser milestone.
5. Compile it.
6. Run the available tests.
7. Fix observed problems.
8. Document what works and what does not.
9. Continue to the next milestone.

Prefer working increments over generating thousands of lines of untested code.

When a major capability requires work beyond the current milestone, create an explicit issue or backlog item rather than pretending it exists.

Maintain:

- `ARCHITECTURE.md`
- `ROADMAP.md`
- `COMPATIBILITY.md`
- `SECURITY.md`
- `BENCHMARKS.md`
- `BUILD.md`
- `AGENTS.md`

Update them as implementation progresses.

For every milestone provide:

- Files created or modified
- Architectural decisions
- Build results
- Test results
- Known limitations
- Performance measurements if available
- Next concrete engineering tasks

Do not ask unnecessary implementation questions.

Use reasonable defaults consistent with this specification.

Do not run destructive commands, overwrite important local data or install system-wide packages without appropriate authorization.

**Begin with Phase 0 and Phase 1. Do real engineering work immediately.**

# FINAL PRODUCT VISION

Crest should eventually become a browser where:

- Humans browse normally in a beautiful, highly customizable interface.
- Google websites and modern web applications work reliably.
- Widely used compatible Chrome extensions are supported.
- Native Rust widgets provide efficient personalization.
- External AI agents have independent, observable browser workspaces.
- Codex, Claude Code, Antigravity and other compatible tools connect through standard interfaces.
- Agents can test websites, fill authorized forms, inspect developer tools and create extensions.
- Users retain control of permissions and credentials.
- Multiple agents can work without interrupting normal browsing.
- Rust is used extensively for Crest's own architecture.
- The browser remains secure, maintainable and measurably resource-efficient.

**Build a real browser. Prioritize correctness, compatibility, security, visual quality and measurable performance over impressive but unverified claims.**