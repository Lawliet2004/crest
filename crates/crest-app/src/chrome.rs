use crate::window::{Action, Menu, TabId, WindowState};
use cef::*;
use crest_core::{BLANK, HOME, Page, TabLayout};
use std::{cell::RefCell, rc::Weak};

const CHROME: u32 = 0xffedf4f2;
const PAPER: u32 = 0xfff7faf9;
const SELECTED: u32 = 0xffcfe8e1;
const INK: u32 = 0xff173b36;
const TAB_WIDTH: i32 = 150;
const TAB_HEIGHT: i32 = 34;
const SIDEBAR_WIDTH: i32 = 206;

pub(crate) const SHORTCUTS: &[(Action, i32, i32, i32, i32)] = &[
    (Action::FocusAddress, b'L' as i32, 0, 1, 0),
    (Action::NewTab, b'T' as i32, 0, 1, 0),
    (Action::CloseTab, b'W' as i32, 0, 1, 0),
    (Action::CloseWindow, b'W' as i32, 1, 1, 0),
    (Action::Bookmark, b'D' as i32, 0, 1, 0),
    (Action::Reload, b'R' as i32, 0, 1, 0),
    (Action::Reload, 0x74, 0, 0, 0),
    (Action::Back, 0x25, 0, 0, 1),
    (Action::Forward, 0x27, 0, 0, 1),
    (Action::NextTab, 0x09, 0, 1, 0),
    (Action::PreviousTab, 0x09, 1, 1, 0),
    (Action::DevTools, 0x7b, 0, 0, 0),
    (Action::DevTools, b'I' as i32, 1, 1, 0),
];

pub(crate) struct Chrome {
    pub address: Textfield,
    pub deck: Panel,
    back: LabelButton,
    forward: LabelButton,
    reload: LabelButton,
    bookmark: LabelButton,
    status: Textfield,
    horizontal: ScrollView,
    horizontal_tabs: Panel,
    vertical: ScrollView,
    vertical_tabs: Panel,
    buttons: RefCell<Vec<(TabId, LabelButton)>>,
}

impl Chrome {
    pub(crate) fn build(window: &Window, state: Weak<WindowState>) -> Option<Self> {
        window.set_background_color(PAPER);
        let root = box_layout(window, false, 0, Insets::default())?;

        let horizontal = scroll_view_create(Some(&mut ViewSize::new(Size {
            width: 0,
            height: TAB_HEIGHT + 14,
        })))?;
        let horizontal_tabs = panel_create(None)?;
        horizontal_tabs.set_background_color(CHROME);
        horizontal.set_background_color(CHROME);
        horizontal.set_content_view(Some(&mut View::from(&horizontal_tabs)));
        window.add_child_view(Some(&mut View::from(&horizontal)));

        let toolbar = panel_create(None)?;
        toolbar.set_background_color(PAPER);
        let tools = box_layout(&toolbar, true, 5, insets(6, 10))?;
        let back = button("←", "Back (Alt+Left)", Action::Back, &state)?;
        let forward = button("→", "Forward (Alt+Right)", Action::Forward, &state)?;
        let reload = button("↻", "Reload (Ctrl+R)", Action::Reload, &state)?;
        let home = button("⌂", "Home", Action::Home, &state)?;
        for control in [&back, &forward, &reload, &home] {
            toolbar.add_child_view(Some(&mut View::from(control)));
        }

        let address = textfield_create(Some(&mut AddressInput::new(state.clone())))?;
        address.set_placeholder_text(Some(&CefString::from("Search or enter an address")));
        address.set_accessible_name(Some(&CefString::from("Address and search bar")));
        address.set_font_list(Some(&CefString::from("sans-serif, 14px")));
        let mut field = View::from(&address);
        toolbar.add_child_view(Some(&mut field));
        tools.set_flex_for_view(Some(&mut field), 1);

        let bookmark = button("☆", "Bookmark this page (Ctrl+D)", Action::Bookmark, &state)?;
        let library = button(
            "Library",
            "Bookmarks and history",
            Action::Show(Menu::Library),
            &state,
        )?;
        let downloads = button(
            "Downloads",
            "Recent downloads",
            Action::Show(Menu::Downloads),
            &state,
        )?;
        let settings = button(
            "Menu",
            "Browser settings and tools",
            Action::Show(Menu::Settings),
            &state,
        )?;
        for control in [&bookmark, &library, &downloads, &settings] {
            toolbar.add_child_view(Some(&mut View::from(control)));
        }
        window.add_child_view(Some(&mut View::from(&toolbar)));

        let body = panel_create(None)?;
        let body_layout = box_layout(&body, true, 0, Insets::default())?;
        let vertical = scroll_view_create(Some(&mut ViewSize::new(Size {
            width: SIDEBAR_WIDTH,
            height: 0,
        })))?;
        let vertical_tabs = panel_create(None)?;
        vertical_tabs.set_background_color(CHROME);
        vertical.set_background_color(CHROME);
        vertical.set_content_view(Some(&mut View::from(&vertical_tabs)));
        body.add_child_view(Some(&mut View::from(&vertical)));

        let deck = panel_create(None)?;
        deck.set_background_color(0xfff4f9f7);
        deck.set_to_fill_layout()?;
        let mut deck_view = View::from(&deck);
        body.add_child_view(Some(&mut deck_view));
        body_layout.set_flex_for_view(Some(&mut deck_view), 1);
        let mut body_view = View::from(&body);
        window.add_child_view(Some(&mut body_view));
        root.set_flex_for_view(Some(&mut body_view), 1);

        let status = textfield_create(None)?;
        status.set_read_only(1);
        status.set_focusable(0);
        status.set_accessible_name(Some(&CefString::from("Browser status")));
        status.set_font_list(Some(&CefString::from("sans-serif, 12px")));
        status.set_background_color(PAPER);
        window.add_child_view(Some(&mut View::from(&status)));
        for (index, (_, key, shift, ctrl, alt)) in SHORTCUTS.iter().enumerate() {
            window.set_accelerator(index as i32 + 1, *key, *shift, *ctrl, *alt, 1);
        }

        Some(Self {
            address,
            deck,
            back,
            forward,
            reload,
            bookmark,
            status,
            horizontal,
            horizontal_tabs,
            vertical,
            vertical_tabs,
            buttons: RefCell::new(Vec::new()),
        })
    }

    pub(crate) fn build_tabs(&self, layout: TabLayout, ids: &[TabId], state: &Weak<WindowState>) {
        let horizontal = layout == TabLayout::Horizontal;
        self.horizontal.set_visible(i32::from(horizontal));
        self.vertical.set_visible(i32::from(!horizontal));
        self.horizontal_tabs.remove_all_child_views();
        self.vertical_tabs.remove_all_child_views();
        let strip = if horizontal {
            &self.horizontal_tabs
        } else {
            &self.vertical_tabs
        };
        let Some(_) = box_layout(strip, horizontal, 4, insets(4, 8)) else {
            return;
        };

        // Put New and Close before the page tabs.
        for (text, label, action) in [
            ("+", "New tab (Ctrl+T)", Action::NewTab),
            ("×", "Close active tab (Ctrl+W)", Action::CloseTab),
        ] {
            if let Some(control) = button(text, label, action, state) {
                strip.add_child_view(Some(&mut View::from(&control)));
            }
        }

        let width = if horizontal {
            TAB_WIDTH
        } else {
            SIDEBAR_WIDTH - 28
        };
        let mut buttons = Vec::with_capacity(ids.len());
        for &id in ids {
            if let Some(tab) = button("New tab", "New tab", Action::Select(id), state) {
                let size = Size {
                    width,
                    height: TAB_HEIGHT,
                };
                tab.set_minimum_size(Some(&size));
                tab.set_maximum_size(Some(&size));
                tab.set_horizontal_alignment(HorizontalAlignment::LEFT);
                strip.add_child_view(Some(&mut View::from(&tab)));
                buttons.push((id, tab));
            }
        }
        *self.buttons.borrow_mut() = buttons;
        let size = strip.preferred_size();
        strip.set_size(Some(&size));
        strip.layout();
        if let Some(window) = self.address.window() {
            window.layout();
        }
    }

    pub(crate) fn update_tabs(&self, pages: &[(TabId, Page)], selected: TabId) {
        for ((id, button), (_, page)) in self.buttons.borrow().iter().zip(pages) {
            let label = display_text(page.label(), 120);
            button.set_text(Some(&CefString::from(label.as_str())));
            button.set_accessible_name(Some(&CefString::from(label.as_str())));
            button.set_tooltip_text(Some(&CefString::from(label.as_str())));
            button.set_background_color(if *id == selected { SELECTED } else { CHROME });
        }
    }

    pub(crate) fn show_address(&self, url: &str) {
        let text = if matches!(url, HOME | BLANK) { "" } else { url };
        if CefString::from(&self.address.text()).to_string() != text {
            self.address.set_text(Some(&CefString::from(text)));
            self.address.clear_edit_history();
        }
    }

    pub(crate) fn show_navigation(&self, browser: Option<&Browser>, bookmarked: bool) {
        let loading = browser.is_some_and(|browser| browser.is_loading() != 0);
        self.back.set_enabled(i32::from(
            browser.is_some_and(|browser| browser.can_go_back() != 0),
        ));
        self.forward.set_enabled(i32::from(
            browser.is_some_and(|browser| browser.can_go_forward() != 0),
        ));
        let (symbol, label) = if loading {
            ("×", "Stop loading")
        } else {
            ("↻", "Reload (Ctrl+R)")
        };
        self.reload.set_text(Some(&CefString::from(symbol)));
        self.reload
            .set_accessible_name(Some(&CefString::from(label)));
        self.reload.set_tooltip_text(Some(&CefString::from(label)));
        self.bookmark.set_text(Some(&CefString::from(if bookmarked {
            "★"
        } else {
            "☆"
        })));
    }

    pub(crate) fn set_status(&self, status: &str) {
        self.status
            .set_text(Some(&CefString::from(display_text(status, 250).as_str())));
    }
}

fn insets(vertical: i32, horizontal: i32) -> Insets {
    Insets {
        top: vertical,
        left: horizontal,
        bottom: vertical,
        right: horizontal,
    }
}

fn box_layout(
    panel: &impl ImplPanel,
    horizontal: bool,
    gap: i32,
    insets: Insets,
) -> Option<BoxLayout> {
    panel.set_to_box_layout(Some(&BoxLayoutSettings {
        horizontal: i32::from(horizontal),
        between_child_spacing: gap,
        inside_border_insets: insets,
        cross_axis_alignment: AxisAlignment::STRETCH,
        ..BoxLayoutSettings::default()
    }))
}

fn button(
    text: &str,
    label: &str,
    action: Action,
    state: &Weak<WindowState>,
) -> Option<LabelButton> {
    let mut delegate = Control::new(state.clone(), action);
    let button = label_button_create(Some(&mut delegate), Some(&CefString::from(text)))?;
    button.set_accessible_name(Some(&CefString::from(label)));
    button.set_tooltip_text(Some(&CefString::from(label)));
    button.set_font_list(Some(&CefString::from("sans-serif, 13px")));
    button.set_enabled_text_colors(INK);
    button.set_minimum_size(Some(&Size {
        width: 31,
        height: TAB_HEIGHT,
    }));
    Some(button)
}

pub(crate) fn display_text(text: &str, limit: usize) -> String {
    text.chars()
        .filter(|ch| {
            !ch.is_control() && !matches!(ch, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
        .take(limit)
        .collect()
}

wrap_view_delegate! {
    struct ViewSize { size: Size }
    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            self.size.clone()
        }
    }
}

wrap_button_delegate! {
    struct Control { state: Weak<WindowState>, action: Action }
    impl ViewDelegate {}
    impl ButtonDelegate {
        fn on_button_pressed(&self, button: Option<&mut Button>) {
            if let Some(state) = self.state.upgrade() {
                if let Action::Show(menu) = self.action {
                    if let Some(button) = button {
                        let bounds = button.bounds_in_screen();
                        state.show_menu(menu, Point { x: bounds.x, y: bounds.y + bounds.height });
                    }
                } else {
                    state.dispatch(self.action.clone());
                }
            }
        }
    }
}

wrap_textfield_delegate! {
    struct AddressInput { state: Weak<WindowState> }
    impl ViewDelegate {}
    impl TextfieldDelegate {
        fn on_key_event(&self, field: Option<&mut Textfield>, event: Option<&KeyEvent>) -> i32 {
            let (Some(state), Some(field), Some(event)) = (self.state.upgrade(), field, event) else {
                return 0;
            };
            if !matches!(event.type_, KeyEventType::RAWKEYDOWN | KeyEventType::KEYDOWN) {
                return 0;
            }
            match event.windows_key_code {
                0x0d => state.dispatch(Action::Submit(CefString::from(&field.text()).to_string())),
                0x1b => state.dispatch(Action::ResetAddress),
                _ => return 0,
            }
            1
        }
    }
}
