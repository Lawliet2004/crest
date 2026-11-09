use crate::{
    chrome::{Chrome, SHORTCUTS, display_text},
    engine::{CrestClient, PageView},
};
use cef::*;
use crest_core::{HOME, MAX_TABS, Page, Profile, SearchEngine, TabLayout, is_page_url, resolve};
use std::{
    cell::{Cell, OnceCell, RefCell},
    collections::HashSet,
    path::PathBuf,
    rc::{Rc as Shared, Weak},
};

pub(crate) type TabId = u32;

#[derive(Clone)]
pub(crate) enum Action {
    NewTab,
    CloseTab,
    CloseWindow,
    Detach(TabId),
    Select(TabId),
    NextTab,
    PreviousTab,
    FocusAddress,
    ResetAddress,
    Submit(String),
    Navigate(String),
    OpenInTab(String, bool),
    Back,
    Forward,
    Reload,
    Home,
    Bookmark,
    Layout(TabLayout),
    Search(SearchEngine),
    DevTools,
    Show(Menu),
}

#[derive(Clone, Copy)]
pub(crate) enum Menu {
    Library,
    Downloads,
    Settings,
}

struct Tab {
    id: TabId,
    page: Page,
    view: Option<BrowserView>,
    browser_id: Option<i32>,
}

#[derive(Clone)]
struct Download {
    id: u32,
    name: String,
    status: String,
}

#[derive(Clone)]
enum Deferred {
    Run(Action),
    Paint,
    Save,
}

pub(crate) struct WindowState {
    self_ref: Weak<Self>,
    chrome: OnceCell<Chrome>,
    profile: RefCell<Profile>,
    profile_path: PathBuf,
    allow_save: bool,
    tabs: RefCell<Vec<Tab>>,
    selected: Cell<TabId>,
    next_id: Cell<TabId>,
    live: RefCell<HashSet<i32>>,
    downloads: RefCell<Vec<Download>>,
    closing: Cell<bool>,
    destroyed: Cell<bool>,
    paint_pending: Cell<bool>,
    save_pending: Cell<bool>,
}

impl WindowState {
    pub(crate) fn new(profile_path: PathBuf, input: Option<&str>) -> Result<Shared<Self>, String> {
        let (mut profile, allow_save) = match Profile::load(&profile_path) {
            Ok(profile) => (profile, true),
            Err(error) => {
                eprintln!(
                    "Crest Browser could not read {}: {error}",
                    profile_path.display()
                );
                (Profile::default(), false)
            }
        };
        let mut pages = std::mem::take(&mut profile.tabs);
        let mut selected = profile.selected;
        if let Some(input) = input {
            let url = resolve(input, profile.settings.search_engine)?;
            if pages.len() == MAX_TABS {
                return Err("Close a tab before opening another from the command line".into());
            }
            selected = pages.len();
            pages.push(Page::new(url));
        }
        if pages.is_empty() {
            pages.push(Page::new(HOME));
        }
        let tabs: Vec<_> = pages
            .into_iter()
            .enumerate()
            .map(|(index, page)| Tab {
                id: index as TabId,
                page,
                view: None,
                browser_id: None,
            })
            .collect();
        Ok(Shared::new_cyclic(|self_ref| Self {
            self_ref: self_ref.clone(),
            chrome: OnceCell::new(),
            profile: RefCell::new(profile),
            profile_path,
            allow_save,
            selected: Cell::new(tabs[selected].id),
            next_id: Cell::new(tabs.len() as TabId),
            tabs: RefCell::new(tabs),
            live: RefCell::new(HashSet::new()),
            downloads: RefCell::new(Vec::new()),
            closing: Cell::new(false),
            destroyed: Cell::new(false),
            paint_pending: Cell::new(false),
            save_pending: Cell::new(false),
        }))
    }

    pub(crate) fn show(&self) -> bool {
        let mut delegate = MainWindow::new(self.self_ref.clone());
        window_create_top_level(Some(&mut delegate)).is_some() && self.chrome.get().is_some()
    }

    pub(crate) fn relaunch(&self, input: Option<&str>) {
        if self.closing.get() {
            return;
        }
        if let Some(window) = self.chrome().address.window() {
            if window.is_minimized() != 0 {
                window.restore();
            }
            window.activate();
        }
        if let Some(input) = input {
            let search = self.profile.borrow().settings.search_engine;
            match resolve(input, search) {
                Ok(url) if self.can_open_tab() => self.dispatch(Action::OpenInTab(url, true)),
                Ok(_) => self.status("Tab limit reached"),
                Err(error) => self.status(error),
            }
        }
    }

    fn chrome(&self) -> &Chrome {
        self.chrome.get().expect("window controls were initialized")
    }

    fn active_view(&self) -> Option<BrowserView> {
        self.tabs
            .borrow()
            .iter()
            .find(|tab| tab.id == self.selected.get())?
            .view
            .clone()
    }

    fn active_browser(&self) -> Option<Browser> {
        self.active_view()?.browser()
    }

    fn active_page(&self) -> Option<Page> {
        self.tabs
            .borrow()
            .iter()
            .find(|tab| tab.id == self.selected.get())
            .map(|tab| tab.page.clone())
    }

    fn rebuild_tabs(&self) {
        let ids: Vec<_> = self.tabs.borrow().iter().map(|tab| tab.id).collect();
        let layout = self.profile.borrow().settings.tab_layout;
        self.chrome().build_tabs(layout, &ids, &self.self_ref);
    }

    fn refresh(&self, reset_address: bool) {
        let Some(chrome) = self.chrome.get() else {
            return;
        };
        let pages: Vec<_> = self
            .tabs
            .borrow()
            .iter()
            .map(|tab| (tab.id, tab.page.clone()))
            .collect();
        chrome.update_tabs(&pages, self.selected.get());
        let browser = self.active_browser();
        if let Some((_, page)) = pages.iter().find(|(id, _)| *id == self.selected.get()) {
            if reset_address || chrome.address.has_focus() == 0 {
                chrome.show_address(&page.url);
            }
            let bookmarked = self.profile.borrow().is_bookmarked(&page.url);
            chrome.show_navigation(browser.as_ref(), bookmarked);
            if let Some(window) = chrome.address.window() {
                let title = if page.url == HOME {
                    "Crest Browser".into()
                } else {
                    format!("Crest Browser — {}", display_text(page.label(), 90))
                };
                window.set_title(Some(&CefString::from(title.as_str())));
            }
        }
    }

    fn select(&self, id: TabId, focus_address: bool) {
        if !self.tabs.borrow().iter().any(|tab| tab.id == id) {
            return;
        }
        let previous = self.selected.replace(id);
        if !self.ensure_view(id) {
            self.selected.set(previous);
            self.status("Could not open the tab");
            return;
        }
        let views: Vec<_> = self
            .tabs
            .borrow()
            .iter()
            .filter_map(|tab| Some((tab.id, tab.view.clone()?)))
            .collect();
        for (tab, view) in views {
            view.set_visible(i32::from(tab == id));
        }
        self.refresh(true);
        self.schedule_save();
        if focus_address {
            self.focus_address();
        } else if let Some(view) = self.active_view() {
            view.request_focus();
        }
    }

    fn ensure_view(&self, id: TabId) -> bool {
        let url = {
            let tabs = self.tabs.borrow();
            let Some(tab) = tabs.iter().find(|tab| tab.id == id) else {
                return false;
            };
            if tab.view.is_some() {
                return true;
            }
            tab.page.url.clone()
        };
        let mut client = CrestClient::new(self.self_ref.clone());
        let mut delegate = PageView::new(self.self_ref.clone(), Some(id));
        let Some(view) = browser_view_create(
            Some(&mut client),
            Some(&CefString::from(url.as_str())),
            Some(&BrowserSettings::default()),
            None,
            None,
            Some(&mut delegate),
        ) else {
            return false;
        };
        if let Some(tab) = self.tabs.borrow_mut().iter_mut().find(|tab| tab.id == id) {
            tab.view = Some(view.clone());
        }
        self.chrome()
            .deck
            .add_child_view(Some(&mut View::from(&view)));
        self.chrome().deck.layout();
        true
    }

    fn insert_tab(&self, page: Page, view: Option<BrowserView>, browser_id: Option<i32>) -> TabId {
        let id = self.next_id.get();
        self.next_id.set(id.wrapping_add(1));
        self.tabs.borrow_mut().push(Tab {
            id,
            page,
            view,
            browser_id,
        });
        self.rebuild_tabs();
        id
    }

    fn new_tab(&self) {
        if !self.can_open_tab() {
            self.status("Tab limit reached");
            return;
        }
        let id = self.insert_tab(Page::new(HOME), None, None);
        self.select(id, true);
    }

    fn open_in_tab(&self, url: String, activate: bool) {
        if !self.can_open_tab() || !is_page_url(&url) {
            return;
        }
        let id = self.insert_tab(Page::new(url), None, None);
        if activate {
            self.select(id, false);
        } else {
            self.refresh(false);
            self.schedule_save();
        }
    }

    pub(crate) fn can_open_tab(&self) -> bool {
        !self.closing.get() && self.tabs.borrow().len() < MAX_TABS
    }

    pub(crate) fn attach_popup(&self, view: &BrowserView) -> bool {
        if !self.can_open_tab() {
            if let Some(host) = view.browser().and_then(|browser| browser.host()) {
                host.close_browser(1);
            }
            return true;
        }
        let Some(browser) = view.browser() else {
            return false;
        };
        let url = browser
            .main_frame()
            .map(|frame| CefString::from(&frame.url()).to_string())
            .filter(|url| is_page_url(url))
            .unwrap_or_else(|| HOME.into());
        let id = self.insert_tab(
            Page::new(url),
            Some(view.clone()),
            Some(browser.identifier()),
        );
        self.chrome()
            .deck
            .add_child_view(Some(&mut View::from(view)));
        self.chrome().deck.layout();
        self.select(id, false);
        true
    }

    pub(crate) fn set_browser_id(&self, id: TabId, browser_id: i32) {
        if let Some(tab) = self.tabs.borrow_mut().iter_mut().find(|tab| tab.id == id) {
            tab.browser_id = Some(browser_id);
        }
    }

    pub(crate) fn tab_id(&self, browser_id: i32) -> Option<TabId> {
        self.tabs
            .borrow()
            .iter()
            .find(|tab| tab.browser_id == Some(browser_id))
            .map(|tab| tab.id)
    }

    fn detach_tab(&self, id: TabId) {
        let (closed, next) = {
            let mut tabs = self.tabs.borrow_mut();
            let Some(index) = tabs.iter().position(|tab| tab.id == id) else {
                return;
            };
            let closed = tabs.remove(index);
            let next = tabs
                .get(index.min(tabs.len().saturating_sub(1)))
                .map(|tab| tab.id);
            (closed, next)
        };
        if let Some(view) = closed.view {
            self.chrome()
                .deck
                .remove_child_view(Some(&mut View::from(&view)));
        }
        self.rebuild_tabs();
        if self.selected.get() == id {
            if let Some(next) = next {
                self.select(next, false);
            } else {
                self.new_tab();
            }
        } else {
            self.refresh(false);
            self.schedule_save();
        }
    }

    fn step_tab(&self, backwards: bool) {
        let next = {
            let tabs = self.tabs.borrow();
            let Some(index) = tabs.iter().position(|tab| tab.id == self.selected.get()) else {
                return;
            };
            let offset = if backwards { tabs.len() - 1 } else { 1 };
            tabs[(index + offset) % tabs.len()].id
        };
        self.select(next, false);
    }

    pub(crate) fn browser_created(&self, id: i32) {
        self.live.borrow_mut().insert(id);
    }

    pub(crate) fn browser_closed(&self, id: i32) {
        self.live.borrow_mut().remove(&id);
        self.maybe_quit();
    }

    pub(crate) fn window_closing(&self) -> bool {
        self.closing.get()
    }

    fn maybe_quit(&self) {
        if self.destroyed.get() && self.live.borrow().is_empty() {
            quit_message_loop();
        }
    }

    fn close_dev_tools(&self) {
        let views: Vec<_> = self
            .tabs
            .borrow()
            .iter()
            .filter_map(|tab| tab.view.clone())
            .collect();
        for view in views {
            if let Some(host) = view.browser().and_then(|browser| browser.host()) {
                host.close_dev_tools();
            }
        }
    }

    fn focus_address(&self) {
        let address = &self.chrome().address;
        address.request_focus();
        address.select_all(0);
    }

    fn navigate(&self, url: &str) {
        if let Some(view) = self.active_view()
            && let Some(frame) = view.browser().and_then(|browser| browser.main_frame())
        {
            self.chrome().show_address(url);
            view.request_focus();
            self.status("");
            frame.load_url(Some(&CefString::from(url)));
        }
    }

    pub(crate) fn address_changed(&self, browser_id: i32, url: String) {
        if self.closing.get() || !is_page_url(&url) {
            return;
        }
        let page = {
            let mut tabs = self.tabs.borrow_mut();
            let Some(tab) = tabs
                .iter_mut()
                .find(|tab| tab.browser_id == Some(browser_id))
            else {
                return;
            };
            if tab.page.url == url {
                tab.page.clone()
            } else {
                tab.page = Page::new(url);
                tab.page.clone()
            }
        };
        self.profile.borrow_mut().record_visit(&page);
        self.schedule_refresh();
        self.schedule_save();
    }

    pub(crate) fn title_changed(&self, browser_id: i32, title: &str) {
        if self.closing.get() {
            return;
        }
        let page = {
            let mut tabs = self.tabs.borrow_mut();
            let Some(tab) = tabs
                .iter_mut()
                .find(|tab| tab.browser_id == Some(browser_id))
            else {
                return;
            };
            tab.page.title = display_text(title, 512);
            tab.page.clone()
        };
        self.profile.borrow_mut().update_title(&page);
        self.schedule_refresh();
        self.schedule_save();
    }

    pub(crate) fn status(&self, status: &str) {
        if !self.closing.get()
            && let Some(chrome) = self.chrome.get()
        {
            chrome.set_status(status);
        }
    }

    pub(crate) fn page_status(&self, browser_id: i32, status: &str) {
        if self.tab_id(browser_id) == Some(self.selected.get()) {
            self.status(status);
        }
    }

    pub(crate) fn download_changed(&self, id: u32, name: String, status: String) {
        self.status(&format!("{name} · {status}"));
        let mut downloads = self.downloads.borrow_mut();
        downloads.retain(|download| download.id != id);
        downloads.insert(0, Download { id, name, status });
        downloads.truncate(32);
    }

    pub(crate) fn schedule_refresh(&self) {
        if !self.closing.get() && !self.paint_pending.replace(true) {
            self.defer(Deferred::Paint, 0);
        }
    }

    fn schedule_save(&self) {
        if self.allow_save && !self.closing.get() && !self.save_pending.replace(true) {
            self.defer(Deferred::Save, 350);
        }
    }

    fn save(&self) {
        if !self.allow_save {
            return;
        }
        let mut profile = self.profile.borrow_mut();
        let tabs = self.tabs.borrow();
        profile.tabs = tabs.iter().map(|tab| tab.page.clone()).collect();
        profile.selected = tabs
            .iter()
            .position(|tab| tab.id == self.selected.get())
            .unwrap_or(0);
        if let Err(error) = profile.save(&self.profile_path) {
            eprintln!(
                "Crest Browser could not save {}: {error}",
                self.profile_path.display()
            );
            self.status("Could not save the Crest profile");
        }
    }

    pub(crate) fn dispatch(&self, action: Action) {
        self.defer(Deferred::Run(action), 0);
    }

    fn defer(&self, work: Deferred, delay_ms: i64) {
        let mut task = WindowTask::new(self.self_ref.clone(), work);
        if delay_ms == 0 {
            post_task(ThreadId::UI, Some(&mut task));
        } else {
            post_delayed_task(ThreadId::UI, Some(&mut task), delay_ms);
        }
    }

    fn run(&self, action: Action) {
        if self.closing.get() {
            return;
        }
        match action {
            Action::NewTab => self.new_tab(),
            Action::CloseTab => {
                if let Some(host) = self.active_browser().and_then(|browser| browser.host()) {
                    host.close_browser(0);
                } else {
                    self.detach_tab(self.selected.get());
                }
            }
            Action::CloseWindow => {
                if let Some(window) = self.chrome().address.window() {
                    window.close();
                }
            }
            Action::Detach(id) => self.detach_tab(id),
            Action::Select(id) => self.select(id, false),
            Action::NextTab => self.step_tab(false),
            Action::PreviousTab => self.step_tab(true),
            Action::FocusAddress => self.focus_address(),
            Action::ResetAddress => self.refresh(true),
            Action::Submit(input) => {
                let search = self.profile.borrow().settings.search_engine;
                match resolve(&input, search) {
                    Ok(url) => self.navigate(&url),
                    Err(error) => self.status(error),
                }
            }
            Action::Navigate(url) => self.navigate(&url),
            Action::OpenInTab(url, activate) => self.open_in_tab(url, activate),
            Action::Back | Action::Forward | Action::Reload => {
                self.status("");
                if let Some(browser) = self.active_browser() {
                    match action {
                        Action::Back => browser.go_back(),
                        Action::Forward => browser.go_forward(),
                        Action::Reload if browser.is_loading() != 0 => browser.stop_load(),
                        Action::Reload => browser.reload(),
                        _ => unreachable!(),
                    }
                }
            }
            Action::Home => self.navigate(HOME),
            Action::Bookmark => {
                if let Some(page) = self.active_page() {
                    let result = self.profile.borrow_mut().toggle_bookmark(&page);
                    self.status(match result {
                        Ok(true) => "Bookmark added",
                        Ok(false) => "Bookmark removed",
                        Err(error) => error,
                    });
                    self.refresh(false);
                    self.schedule_save();
                }
            }
            Action::Layout(layout) => {
                self.profile.borrow_mut().settings.tab_layout = layout;
                self.rebuild_tabs();
                self.refresh(false);
                self.schedule_save();
            }
            Action::Search(engine) => {
                self.profile.borrow_mut().settings.search_engine = engine;
                self.status(match engine {
                    SearchEngine::Google => "Search engine set to Google",
                    SearchEngine::DuckDuckGo => "Search engine set to DuckDuckGo",
                });
                self.schedule_save();
            }
            Action::DevTools => {
                if let Some(host) = self.active_browser().and_then(|browser| browser.host()) {
                    host.show_dev_tools(
                        Some(&WindowInfo::default()),
                        None,
                        Some(&BrowserSettings::default()),
                        None,
                    );
                }
            }
            Action::Show(_) => {}
        }
    }

    pub(crate) fn show_menu(&self, menu: Menu, point: Point) {
        let Some(window) = self.chrome().address.window() else {
            return;
        };
        let actions = Shared::new(RefCell::new(Vec::new()));
        let mut delegate = MenuEvents::new(self.self_ref.clone(), actions.clone());
        let Some(mut model) = menu_model_create(Some(&mut delegate)) else {
            return;
        };
        let add = |menu: &MenuModel, label: &str, action: Option<Action>| {
            let enabled = action.is_some();
            let id = {
                let mut commands = actions.borrow_mut();
                commands.push(action);
                commands.len() as i32
            };
            let label = display_text(label, 100).replace('&', "&&");
            menu.add_item(id, Some(&CefString::from(label.as_str())));
            menu.set_enabled(id, i32::from(enabled));
        };
        match menu {
            Menu::Library => {
                let profile = self.profile.borrow();
                for (id, label, entries) in [
                    (10_001, "Bookmarks", &profile.bookmarks),
                    (10_002, "Recent history", &profile.history),
                ] {
                    if let Some(group) = model.add_sub_menu(id, Some(&CefString::from(label))) {
                        if entries.is_empty() {
                            add(&group, "Empty", None);
                        }
                        let entries = if id == 10_002 {
                            &entries[..entries.len().min(30)]
                        } else {
                            entries
                        };
                        for page in entries {
                            add(
                                &group,
                                page.label(),
                                Some(Action::Navigate(page.url.clone())),
                            );
                        }
                    }
                }
            }
            Menu::Downloads => {
                let downloads = self.downloads.borrow();
                if downloads.is_empty() {
                    add(&model, "No downloads in this session", None);
                }
                for download in &*downloads {
                    add(
                        &model,
                        &format!("{} · {}", download.name, download.status),
                        None,
                    );
                }
            }
            Menu::Settings => {
                let settings = self.profile.borrow().settings;
                let (layout_label, next_layout) = match settings.tab_layout {
                    TabLayout::Horizontal => ("Use vertical tabs", TabLayout::Vertical),
                    TabLayout::Vertical => ("Use horizontal tabs", TabLayout::Horizontal),
                };
                add(&model, layout_label, Some(Action::Layout(next_layout)));
                if let Some(group) =
                    model.add_sub_menu(10_003, Some(&CefString::from("Search engine")))
                {
                    for (label, engine) in [
                        ("Google", SearchEngine::Google),
                        ("DuckDuckGo", SearchEngine::DuckDuckGo),
                    ] {
                        let label = if settings.search_engine == engine {
                            format!("✓ {label}")
                        } else {
                            label.into()
                        };
                        add(&group, &label, Some(Action::Search(engine)));
                    }
                }
                model.add_separator();
                add(
                    &model,
                    "Developer Tools (Ctrl+Shift+I)",
                    Some(Action::DevTools),
                );
                add(&model, "Site permissions · blocked", None);
                model.add_separator();
                add(&model, "Quit Crest Browser", Some(Action::CloseWindow));
            }
        }
        window.show_menu(Some(&mut model), Some(&point), MenuAnchorPosition::TOPLEFT);
    }
}

wrap_task! {
    struct WindowTask { state: Weak<WindowState>, work: Deferred }
    impl Task {
        fn execute(&self) {
            let Some(state) = self.state.upgrade() else { return };
            if state.closing.get() {
                return;
            }
            match &self.work {
                Deferred::Run(action) => state.run(action.clone()),
                Deferred::Paint => {
                    state.paint_pending.set(false);
                    state.refresh(false);
                }
                Deferred::Save => {
                    state.save_pending.set(false);
                    state.save();
                }
            }
        }
    }
}

wrap_menu_model_delegate! {
    struct MenuEvents { state: Weak<WindowState>, actions: Shared<RefCell<Vec<Option<Action>>>> }
    impl MenuModelDelegate {
        fn execute_command(&self, _menu: Option<&mut MenuModel>, command_id: i32, _flags: EventFlags) {
            if let Some(state) = self.state.upgrade()
                && let Some(Some(action)) = self.actions.borrow().get(command_id.saturating_sub(1) as usize)
            {
                state.dispatch(action.clone());
            }
        }
    }
}

wrap_window_delegate! {
    struct MainWindow { state: Weak<WindowState> }
    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 1120, height: 740 }
        }
        fn minimum_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 620, height: 380 }
        }
    }
    impl PanelDelegate {}
    impl WindowDelegate {
        fn window_runtime_style(&self) -> RuntimeStyle { RuntimeStyle::ALLOY }
        fn can_resize(&self, _window: Option<&mut Window>) -> i32 { 1 }
        fn can_maximize(&self, _window: Option<&mut Window>) -> i32 { 1 }
        fn can_minimize(&self, _window: Option<&mut Window>) -> i32 { 1 }
        fn can_close(&self, _window: Option<&mut Window>) -> i32 { 1 }

        fn linux_window_properties(
            &self,
            _window: Option<&mut Window>,
            properties: Option<&mut LinuxWindowProperties>,
        ) -> i32 {
            let Some(properties) = properties else { return 0 };
            properties.wm_class_class = CefString::from("crest-browser");
            properties.wm_class_name = CefString::from("crest-browser");
            properties.wayland_app_id = CefString::from("crest-browser");
            1
        }

        fn on_window_created(&self, window: Option<&mut Window>) {
            let (Some(state), Some(window)) = (self.state.upgrade(), window) else { return };
            let Some(chrome) = Chrome::build(window, self.state.clone()) else {
                window.close();
                return;
            };
            let _ = state.chrome.set(chrome);
            window.set_title(Some(&CefString::from("Crest Browser")));
            state.rebuild_tabs();
            let focus_address = state.active_page().is_some_and(|page| page.url == HOME);
            state.select(state.selected.get(), focus_address);
            window.show();
            if focus_address {
                state.focus_address();
            } else if let Some(view) = state.active_view() {
                view.request_focus();
            }
            if !state.allow_save {
                state.status("Could not read the Crest profile. Changes are temporary.");
            }
        }

        fn on_accelerator(&self, _window: Option<&mut Window>, id: i32) -> i32 {
            let (Some(state), Some((action, ..))) = (
                self.state.upgrade(),
                SHORTCUTS.get(id.saturating_sub(1) as usize),
            ) else { return 0 };
            state.dispatch(action.clone());
            1
        }

        fn on_window_closing(&self, _window: Option<&mut Window>) {
            if let Some(state) = self.state.upgrade() {
                state.closing.set(true);
                state.save();
                state.close_dev_tools();
            }
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            if let Some(state) = self.state.upgrade() {
                state.destroyed.set(true);
                state.maybe_quit();
            }
        }
    }
}
