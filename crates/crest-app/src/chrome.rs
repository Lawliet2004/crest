use crate::{
    icons::{Icon, render},
    window::{Action, Menu, TabId, WindowState},
};
use cef::{sys::cef_color_id_t::*, *};
use crest_core::{BLANK, HOME, Page, TabLayout};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc as Shared, Weak},
};
use url::Url;

const RAIL: u32 = 0xff0e1d23;
const TOOLBAR: u32 = 0xff1a2d35;
const ADDRESS: u32 = 0xff102229;
const DIVIDER: u32 = 0xff304851;
const ACTIVE: u32 = 0xff25464b;
const HOVER: u32 = 0xff2c454d;
const TEXT: u32 = 0xffe7f4f5;
const MUTED: u32 = 0xffa6bec4;
const DISABLED: u32 = 0xff647b82;
const ACCENT: u32 = 0xff80e0d1;
const TOOL_SIZE: i32 = 34;
const TAB_HEIGHT: i32 = 33;
const RAIL_WIDTH: i32 = 58;

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
    (Action::ToggleLayout, b'Y' as i32, 1, 1, 0),
    (Action::DevTools, 0x7b, 0, 0, 0),
    (Action::DevTools, b'I' as i32, 1, 1, 0),
];

struct TabControl {
    id: TabId,
    button: LabelButton,
    selected: Shared<Cell<bool>>,
}

pub(crate) struct Chrome {
    pub address: Textfield,
    pub deck: Panel,
    back: LabelButton,
    forward: LabelButton,
    reload: LabelButton,
    bookmark: LabelButton,
    layout_button: LabelButton,
    reload_icon: Image,
    stop_icon: Image,
    bookmark_icon: Image,
    starred_icon: Image,
    new_tab_icon: Image,
    status: Textfield,
    horizontal: Panel,
    horizontal_tabs: Panel,
    vertical: Panel,
    vertical_tabs: Panel,
    layout: Cell<TabLayout>,
    buttons: RefCell<Vec<TabControl>>,
}

impl Chrome {
    pub(crate) fn build(window: &Window, state: Weak<WindowState>) -> Option<Self> {
        Self::apply_palette(window);
        let root = box_layout(window, false, 0, Insets::default(), AxisAlignment::STRETCH)?;

        let toolbar = surface(TOOLBAR, 0, 0)?;
        let toolbar_layout =
            box_layout(&toolbar, true, 0, Insets::default(), AxisAlignment::STRETCH)?;
        let brand = surface(RAIL, RAIL_WIDTH, 46)?;
        box_layout(&brand, false, 0, insets(6, 0), AxisAlignment::CENTER)?;
        add(
            &brand,
            &icon_button(
                Icon::Crest,
                "Open tabs",
                Action::Show(Menu::Tabs),
                &state,
                ACCENT,
            )?,
        );
        add(&toolbar, &brand);

        let navigation = surface(TOOLBAR, 0, 0)?;
        let tools = box_layout(&navigation, true, 3, insets(5, 9), AxisAlignment::CENTER)?;
        let back = icon_button(Icon::Back, "Back (Alt+Left)", Action::Back, &state, MUTED)?;
        let forward = icon_button(
            Icon::Forward,
            "Forward (Alt+Right)",
            Action::Forward,
            &state,
            MUTED,
        )?;
        let reload = icon_button(
            Icon::Reload,
            "Reload (Ctrl+R)",
            Action::Reload,
            &state,
            MUTED,
        )?;
        let home = icon_button(Icon::Home, "Home", Action::Home, &state, MUTED)?;
        for control in [&back, &forward, &reload, &home] {
            add(&navigation, control);
        }
        add(&navigation, &surface(DIVIDER, 1, 18)?);
        add(
            &navigation,
            &icon_button(
                Icon::Search,
                "Address and search (Ctrl+L)",
                Action::FocusAddress,
                &state,
                MUTED,
            )?,
        );

        let address = textfield_create(Some(&mut AddressInput::new(state.clone())))?;
        address.set_placeholder_text(Some(&CefString::from("Search or enter URL")));
        address.set_accessible_name(Some(&CefString::from("Address and search bar")));
        address.set_font_list(Some(&CefString::from("sans-serif, 14px")));
        add_flex(&navigation, &tools, &address);

        let bookmark = icon_button(
            Icon::Star,
            "Bookmark this page (Ctrl+D)",
            Action::Bookmark,
            &state,
            MUTED,
        )?;
        let library = icon_button(
            Icon::Library,
            "Bookmarks and history",
            Action::Show(Menu::Library),
            &state,
            MUTED,
        )?;
        let downloads = icon_button(
            Icon::Download,
            "Recent downloads",
            Action::Show(Menu::Downloads),
            &state,
            MUTED,
        )?;
        let layout_button = icon_button(
            Icon::Layout,
            "Switch tab layout (Ctrl+Shift+Y)",
            Action::ToggleLayout,
            &state,
            MUTED,
        )?;
        let settings = icon_button(
            Icon::Menu,
            "Browser settings and tools",
            Action::Show(Menu::Settings),
            &state,
            MUTED,
        )?;
        for control in [&bookmark, &library, &downloads, &layout_button, &settings] {
            add(&navigation, control);
        }
        add_flex(&toolbar, &toolbar_layout, &navigation);
        add(window, &toolbar);

        let horizontal = surface(RAIL, 0, 0)?;
        let horizontal_layout =
            box_layout(&horizontal, true, 4, insets(3, 8), AxisAlignment::CENTER)?;
        let horizontal_scroll = scroll_view_create(Some(&mut ViewSize::new(
            Size {
                width: 1,
                height: TAB_HEIGHT + 3,
            },
            RAIL,
        )))?;
        let horizontal_tabs = surface(RAIL, 0, 0)?;
        horizontal_scroll.set_content_view(Some(&mut View::from(&horizontal_tabs)));
        add_flex(&horizontal, &horizontal_layout, &horizontal_scroll);
        add_tab_actions(&horizontal, &state)?;
        add(window, &horizontal);
        add(window, &surface(DIVIDER, 1, 1)?);

        let body = surface(RAIL, 0, 0)?;
        let body_layout = box_layout(&body, true, 0, Insets::default(), AxisAlignment::STRETCH)?;
        let vertical = surface(RAIL, RAIL_WIDTH, 1)?;
        let vertical_layout = box_layout(&vertical, false, 5, insets(7, 0), AxisAlignment::CENTER)?;
        add(
            &vertical,
            &icon_button(
                Icon::Plus,
                "New tab (Ctrl+T)",
                Action::NewTab,
                &state,
                ACCENT,
            )?,
        );
        add(&vertical, &surface(DIVIDER, 23, 1)?);
        let vertical_scroll = scroll_view_create(Some(&mut ViewSize::new(
            Size {
                width: RAIL_WIDTH,
                height: 1,
            },
            RAIL,
        )))?;
        let vertical_tabs = surface(RAIL, 0, 0)?;
        vertical_scroll.set_content_view(Some(&mut View::from(&vertical_tabs)));
        add_flex(&vertical, &vertical_layout, &vertical_scroll);
        add(
            &vertical,
            &icon_button(
                Icon::Close,
                "Close active tab (Ctrl+W)",
                Action::CloseTab,
                &state,
                MUTED,
            )?,
        );
        add(&body, &vertical);

        let deck = surface(ADDRESS, 0, 0)?;
        deck.set_to_fill_layout()?;
        add_flex(&body, &body_layout, &deck);
        add_flex(window, &root, &body);

        let status = textfield_create(None)?;
        status.set_read_only(1);
        status.set_focusable(0);
        status.set_accessible_name(Some(&CefString::from("Browser status")));
        status.set_font_list(Some(&CefString::from("sans-serif, 12px")));
        status.set_insets(Some(&insets(3, 12)));
        status.set_visible(0);
        add(window, &status);
        for (index, (_, key, shift, ctrl, alt)) in SHORTCUTS.iter().enumerate() {
            window.set_accelerator(index as i32 + 1, *key, *shift, *ctrl, *alt, 1);
        }
        window.theme_changed();

        Some(Self {
            address,
            deck,
            back,
            forward,
            reload,
            bookmark,
            layout_button,
            reload_icon: render(Icon::Reload, MUTED)?,
            stop_icon: render(Icon::Close, TEXT)?,
            bookmark_icon: render(Icon::Star, MUTED)?,
            starred_icon: render(Icon::Starred, ACCENT)?,
            new_tab_icon: render(Icon::Tab, ACCENT)?,
            status,
            horizontal,
            horizontal_tabs,
            vertical,
            vertical_tabs,
            layout: Cell::new(TabLayout::default()),
            buttons: RefCell::new(Vec::new()),
        })
    }

    pub(crate) fn apply_palette(window: &Window) {
        for (key, color) in [
            (CEF_ColorPrimaryBackground, TOOLBAR),
            (CEF_ColorPrimaryForeground, TEXT),
            (CEF_ColorDialogBackground, RAIL),
            (CEF_ColorOverlayScrollbarFill, DIVIDER),
            (CEF_ColorOverlayScrollbarFillHovered, MUTED),
            (CEF_ColorOverlayScrollbarStroke, RAIL),
            (CEF_ColorOverlayScrollbarStrokeHovered, RAIL),
            (CEF_ColorFrameActive, TOOLBAR),
            (CEF_ColorFrameInactive, RAIL),
            (CEF_ColorTextfieldBackground, ADDRESS),
            (CEF_ColorTextfieldForeground, TEXT),
            (CEF_ColorTextfieldForegroundPlaceholder, MUTED),
            (CEF_ColorTextfieldOutline, DIVIDER),
            (CEF_ColorTextfieldOutlineDisabled, TOOLBAR),
            (CEF_ColorTextfieldSelectionBackground, ACCENT),
            (CEF_ColorTextfieldSelectionForeground, RAIL),
            (CEF_ColorTextfieldHover, 0),
            (CEF_ColorFocusableBorderFocused, ACCENT),
            (CEF_ColorSysStateFocusRing, ACCENT),
            (CEF_ColorTooltipBackground, ADDRESS),
            (CEF_ColorTooltipForeground, TEXT),
        ] {
            window.set_theme_color(key as i32, color);
        }
    }

    pub(crate) fn build_tabs(&self, layout: TabLayout, ids: &[TabId], state: &Weak<WindowState>) {
        let horizontal = layout == TabLayout::Horizontal;
        self.layout.set(layout);
        self.horizontal.set_visible(i32::from(horizontal));
        self.vertical.set_visible(i32::from(!horizontal));
        let toggle_hint = if horizontal {
            "Use vertical tabs (Ctrl+Shift+Y)"
        } else {
            "Use horizontal tabs (Ctrl+Shift+Y)"
        };
        describe(&self.layout_button, toggle_hint);
        self.horizontal_tabs.remove_all_child_views();
        self.vertical_tabs.remove_all_child_views();
        let strip = if horizontal {
            &self.horizontal_tabs
        } else {
            &self.vertical_tabs
        };
        let align = if horizontal {
            AxisAlignment::STRETCH
        } else {
            AxisAlignment::CENTER
        };
        let Some(_) = box_layout(strip, horizontal, 4, Insets::default(), align) else {
            return;
        };
        let mut buttons = Vec::with_capacity(ids.len());
        for &id in ids {
            let selected = Shared::new(Cell::new(false));
            let size = Size {
                width: if horizontal { 164 } else { 36 },
                height: TAB_HEIGHT,
            };
            let mut delegate = Control::new(
                state.clone(),
                Action::Select(id),
                Some(selected.clone()),
                size.clone(),
            );
            let Some(button) = label_button_create(Some(&mut delegate), None) else {
                continue;
            };
            prepare_button(&button, &size, "New tab");
            button.set_font_list(Some(&CefString::from(if horizontal {
                "sans-serif, 13px"
            } else {
                "sans-serif, Bold 14px"
            })));
            if horizontal {
                button.set_insets(Some(&insets(0, 9)));
                button.set_horizontal_alignment(HorizontalAlignment::LEFT);
            }
            add(strip, &button);
            buttons.push(TabControl {
                id,
                button,
                selected,
            });
        }
        *self.buttons.borrow_mut() = buttons;
        let mut size = strip.preferred_size();
        if !horizontal {
            size.width = RAIL_WIDTH;
        }
        strip.set_size(Some(&size));
        strip.layout();
        if let Some(window) = self.address.window() {
            window.layout();
        }
    }

    pub(crate) fn update_tabs(&self, pages: &[(TabId, Page, Option<Image>)], selected: TabId) {
        let horizontal = self.layout.get() == TabLayout::Horizontal;
        for tab in self.buttons.borrow().iter() {
            let Some((_, page, favicon)) = pages.iter().find(|(id, ..)| *id == tab.id) else {
                continue;
            };
            let label = display_text(page.label(), 120);
            let selected = tab.id == selected;
            let icon = favicon.as_ref().or_else(|| {
                matches!(page.url.as_str(), HOME | BLANK).then_some(&self.new_tab_icon)
            });
            let (caption, icon) = if horizontal {
                (label.clone(), icon)
            } else if icon.is_some() {
                (String::new(), icon)
            } else {
                let initial = Url::parse(&page.url)
                    .ok()
                    .and_then(|url| {
                        url.domain()?
                            .trim_start_matches("www.")
                            .chars()
                            .find(|ch| ch.is_alphanumeric())
                    })
                    .or_else(|| label.chars().find(|ch| ch.is_alphanumeric()))
                    .map_or(String::from("•"), |initial| {
                        initial.to_uppercase().collect()
                    });
                (initial, None)
            };
            tab.button
                .set_text(Some(&CefString::from(caption.as_str())));
            describe(&tab.button, &label);
            if selected {
                let accessible = CefString::from(format!("{label}, active tab").as_str());
                tab.button.set_accessible_name(Some(&accessible));
            }
            set_icon(&tab.button, ButtonState::NORMAL, icon);
            tab.selected.set(selected);
            tab.button
                .set_enabled_text_colors(if selected { ACCENT } else { TEXT });
            paint_control(&tab.button, selected);
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
        let (icon, label) = if loading {
            (&self.stop_icon, "Stop loading")
        } else {
            (&self.reload_icon, "Reload (Ctrl+R)")
        };
        set_icon(&self.reload, ButtonState::NORMAL, Some(icon));
        describe(&self.reload, label);
        let (icon, label) = if bookmarked {
            (&self.starred_icon, "Remove bookmark (Ctrl+D)")
        } else {
            (&self.bookmark_icon, "Bookmark this page (Ctrl+D)")
        };
        set_icon(&self.bookmark, ButtonState::NORMAL, Some(icon));
        describe(&self.bookmark, label);
    }

    pub(crate) fn set_status(&self, status: &str) {
        self.status
            .set_text(Some(&CefString::from(display_text(status, 250).as_str())));
        let visible = i32::from(!status.is_empty());
        if self.status.is_visible() != visible {
            self.status.set_visible(visible);
            if let Some(window) = self.status.window() {
                window.layout();
            }
        }
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

fn surface(color: u32, width: i32, height: i32) -> Option<Panel> {
    let panel = panel_create(Some(&mut Surface::new(Size { width, height }, color)))?;
    panel.set_background_color(color);
    Some(panel)
}

fn box_layout(
    panel: &impl ImplPanel,
    horizontal: bool,
    gap: i32,
    insets: Insets,
    cross: AxisAlignment,
) -> Option<BoxLayout> {
    panel.set_to_box_layout(Some(&BoxLayoutSettings {
        horizontal: i32::from(horizontal),
        between_child_spacing: gap,
        inside_border_insets: insets,
        cross_axis_alignment: cross,
        ..BoxLayoutSettings::default()
    }))
}

fn add<'a, T: ImplView>(panel: &impl ImplPanel, child: &'a T)
where
    View: From<&'a T>,
{
    panel.add_child_view(Some(&mut View::from(child)));
}

fn add_flex<'a, T: ImplView>(panel: &impl ImplPanel, layout: &BoxLayout, child: &'a T)
where
    View: From<&'a T>,
{
    let mut view = View::from(child);
    panel.add_child_view(Some(&mut view));
    layout.set_flex_for_view(Some(&mut view), 1);
}

fn add_tab_actions(panel: &Panel, state: &Weak<WindowState>) -> Option<()> {
    add(
        panel,
        &icon_button(
            Icon::Plus,
            "New tab (Ctrl+T)",
            Action::NewTab,
            state,
            ACCENT,
        )?,
    );
    add(
        panel,
        &icon_button(
            Icon::Close,
            "Close active tab (Ctrl+W)",
            Action::CloseTab,
            state,
            MUTED,
        )?,
    );
    Some(())
}

fn icon_button(
    icon: Icon,
    label: &str,
    action: Action,
    state: &Weak<WindowState>,
    color: u32,
) -> Option<LabelButton> {
    let size = Size {
        width: TOOL_SIZE,
        height: TOOL_SIZE,
    };
    let mut delegate = Control::new(state.clone(), action, None, size.clone());
    let button = label_button_create(Some(&mut delegate), None)?;
    prepare_button(&button, &size, label);
    set_icon(&button, ButtonState::NORMAL, Some(&render(icon, color)?));
    if matches!(icon, Icon::Back | Icon::Forward) {
        set_icon(
            &button,
            ButtonState::DISABLED,
            Some(&render(icon, DISABLED)?),
        );
    }
    Some(button)
}

fn prepare_button(button: &LabelButton, size: &Size, label: &str) {
    button.set_insets(Some(&insets(0, 0)));
    button.set_minimum_size(Some(size));
    button.set_maximum_size(Some(size));
    button.set_horizontal_alignment(HorizontalAlignment::CENTER);
    button.set_focusable(1);
    describe(button, label);
}

fn describe(button: &LabelButton, label: &str) {
    let label = CefString::from(label);
    button.set_accessible_name(Some(&label));
    button.set_tooltip_text(Some(&label));
}

fn set_icon(button: &LabelButton, state: ButtonState, icon: Option<&Image>) {
    button.set_image(state, icon.cloned().as_mut());
}

fn paint_control(button: &impl ImplButton, selected: bool) {
    let color = if selected {
        ACTIVE
    } else if matches!(button.state(), ButtonState::HOVERED | ButtonState::PRESSED) {
        HOVER
    } else {
        0
    };
    button.set_background_color(color);
}

pub(crate) fn display_text(text: &str, limit: usize) -> String {
    text.chars()
        .filter(|ch| {
            !ch.is_control() && !matches!(ch, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
        .take(limit)
        .collect()
}

wrap_panel_delegate! {
    struct Surface { size: Size, color: u32 }
    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            self.size.clone()
        }

        fn on_theme_changed(&self, view: Option<&mut View>) {
            if let Some(view) = view {
                view.set_background_color(self.color);
            }
        }
    }
    impl PanelDelegate {}
}

wrap_view_delegate! {
    struct ViewSize { size: Size, color: u32 }
    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            self.size.clone()
        }

        fn on_theme_changed(&self, view: Option<&mut View>) {
            if let Some(view) = view {
                view.set_background_color(self.color);
            }
        }
    }
}

wrap_button_delegate! {
    struct Control {
        state: Weak<WindowState>,
        action: Action,
        selected: Option<Shared<Cell<bool>>>,
        size: Size,
    }
    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            self.size.clone()
        }

        fn on_theme_changed(&self, view: Option<&mut View>) {
            if let Some(button) = view.and_then(|view| view.as_button()) {
                paint_control(&button, self.selected.as_ref().is_some_and(|selected| selected.get()));
            }
        }
    }
    impl ButtonDelegate {
        fn on_button_state_changed(&self, button: Option<&mut Button>) {
            if let Some(button) = button {
                paint_control(button, self.selected.as_ref().is_some_and(|selected| selected.get()));
            }
        }

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
    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 160, height: 36 }
        }
    }
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
