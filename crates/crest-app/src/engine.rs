use crate::{
    chrome::SHORTCUTS,
    window::{Action, TabId, WindowState},
};
use cef::*;
use crest_core::{BLANK, HOME, is_page_url, is_web_url};
use std::{path::Path, rc::Weak};
use url::Url;

const HOME_WALLPAPER: &str = "crest://home/deep-water.webp";
const HOME_SEARCH: &str = "crest://home/search?";

fn home_query(url: &str) -> Option<String> {
    let query = url.strip_prefix(HOME_SEARCH)?;
    if query.len() > 32 * 1024 || query.contains('#') {
        return None;
    }
    let mut params = url::form_urlencoded::parse(query.as_bytes());
    let (name, value) = params.next()?;
    (name == "q" && params.next().is_none()).then(|| value.into_owned())
}

#[cfg(test)]
mod tests {
    use super::home_query;

    #[test]
    fn only_accepts_a_single_query_on_the_internal_search_route() {
        assert_eq!(
            home_query("crest://home/search?q=two+words%26"),
            Some("two words&".into())
        );
        for input in [
            "https://home/search?q=external",
            "crest://home/searching?q=wrong-route",
            "crest://home/search?q=one&q=two",
            "crest://home/search?other=parameter",
            "crest://home/search?q=fragment#extra",
        ] {
            assert!(home_query(input).is_none(), "{input}");
        }
    }
}

fn browser_id(browser: Option<&mut Browser>) -> Option<i32> {
    browser.map(|browser| browser.identifier())
}

fn web_download(url: &str) -> bool {
    is_web_url(url) || url.strip_prefix("blob:").is_some_and(is_web_url)
}

wrap_client! {
    pub(crate) struct CrestClient { state: Weak<WindowState> }
    impl Client {
        fn display_handler(&self) -> Option<DisplayHandler> {
            Some(DisplayEvents::new(self.state.clone()))
        }
        fn load_handler(&self) -> Option<LoadHandler> {
            Some(LoadEvents::new(self.state.clone()))
        }
        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(BrowserLifetime::new(self.state.clone(), false))
        }
        fn request_handler(&self) -> Option<RequestHandler> {
            Some(NavigationPolicy::new(self.state.clone()))
        }
        fn permission_handler(&self) -> Option<PermissionHandler> {
            Some(SitePermissions::new(self.state.clone()))
        }
        fn download_handler(&self) -> Option<DownloadHandler> {
            Some(DownloadEvents::new(self.state.clone()))
        }
        fn keyboard_handler(&self) -> Option<KeyboardHandler> {
            Some(PageKeys::new(self.state.clone()))
        }
    }
}

wrap_browser_view_delegate! {
    pub(crate) struct PageView { state: Weak<WindowState>, tab: Option<TabId> }
    impl ViewDelegate {}
    impl BrowserViewDelegate {
        fn browser_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::ALLOY
        }

        fn on_browser_created(&self, _view: Option<&mut BrowserView>, browser: Option<&mut Browser>) {
            if let (Some(state), Some(tab), Some(id)) = (self.state.upgrade(), self.tab, browser_id(browser)) {
                state.set_browser_id(tab, id);
            }
        }

        fn delegate_for_popup_browser_view(
            &self,
            _view: Option<&mut BrowserView>,
            _settings: Option<&BrowserSettings>,
            _client: Option<&mut Client>,
            is_devtools: i32,
        ) -> Option<BrowserViewDelegate> {
            (is_devtools == 0).then(|| Self::new(self.state.clone(), None))
        }

        fn on_popup_browser_view_created(
            &self,
            _view: Option<&mut BrowserView>,
            popup: Option<&mut BrowserView>,
            is_devtools: i32,
        ) -> i32 {
            if is_devtools != 0 {
                return 0;
            }
            let (Some(state), Some(popup)) = (self.state.upgrade(), popup) else { return 0 };
            i32::from(state.attach_popup(popup))
        }
    }
}

wrap_life_span_handler! {
    struct BrowserLifetime { state: Weak<WindowState>, devtools: bool }
    impl LifeSpanHandler {
        fn on_before_popup(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _popup_id: i32,
            target_url: Option<&CefString>,
            _target_frame: Option<&CefString>,
            _disposition: WindowOpenDisposition,
            user_gesture: i32,
            _features: Option<&PopupFeatures>,
            _info: Option<&mut WindowInfo>,
            _client: Option<&mut Option<Client>>,
            _settings: Option<&mut BrowserSettings>,
            _extra: Option<&mut Option<DictionaryValue>>,
            _no_js_access: Option<&mut i32>,
        ) -> i32 {
            let url = target_url.map(ToString::to_string).unwrap_or_default();
            let allow = !self.devtools && user_gesture != 0
                && (url.is_empty() || is_web_url(&url) || url == BLANK)
                && self.state.upgrade().is_some_and(|state| state.can_open_tab());
            i32::from(!allow)
        }

        fn on_before_dev_tools_popup(
            &self,
            _browser: Option<&mut Browser>,
            _info: Option<&mut WindowInfo>,
            client: Option<&mut Option<Client>>,
            _settings: Option<&mut BrowserSettings>,
            _extra: Option<&mut Option<DictionaryValue>>,
            use_default_window: Option<&mut i32>,
        ) {
            if let Some(client) = client {
                *client = Some(DevToolsClient::new(self.state.clone()));
            }
            if let Some(use_default_window) = use_default_window {
                *use_default_window = 1;
            }
        }

        fn on_after_created(&self, browser: Option<&mut Browser>) {
            if let (Some(state), Some(id)) = (self.state.upgrade(), browser_id(browser)) {
                state.browser_created(id);
            }
        }

        fn do_close(&self, browser: Option<&mut Browser>) -> i32 {
            let (Some(state), Some(id)) = (self.state.upgrade(), browser_id(browser)) else { return 0 };
            if !state.window_closing() && let Some(tab) = state.tab_id(id) {
                // Chromium has completed unload. Detach this view after the callback returns.
                state.dispatch(Action::Detach(tab));
                return 1;
            }
            0
        }

        fn on_before_close(&self, browser: Option<&mut Browser>) {
            if let (Some(state), Some(id)) = (self.state.upgrade(), browser_id(browser)) {
                state.browser_closed(id);
            }
        }
    }
}

wrap_client! {
    struct DevToolsClient { state: Weak<WindowState> }
    impl Client {
        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(BrowserLifetime::new(self.state.clone(), true))
        }
        fn download_handler(&self) -> Option<DownloadHandler> {
            Some(DownloadEvents::new(self.state.clone()))
        }
        fn permission_handler(&self) -> Option<PermissionHandler> {
            Some(SitePermissions::new(self.state.clone()))
        }
    }
}

wrap_display_handler! {
    struct DisplayEvents { state: Weak<WindowState> }
    impl DisplayHandler {
        fn on_address_change(&self, browser: Option<&mut Browser>, frame: Option<&mut Frame>, url: Option<&CefString>) {
            if frame.is_some_and(|frame| frame.is_main() != 0)
                && let (Some(state), Some(id), Some(url)) = (self.state.upgrade(), browser_id(browser), url)
            {
                state.address_changed(id, url.to_string());
            }
        }
        fn on_title_change(&self, browser: Option<&mut Browser>, title: Option<&CefString>) {
            if let (Some(state), Some(id), Some(title)) = (self.state.upgrade(), browser_id(browser), title) {
                state.title_changed(id, &title.to_string());
            }
        }
        fn on_favicon_urlchange(&self, browser: Option<&mut Browser>, icons: Option<&mut CefStringList>) {
            let (Some(state), Some(browser), Some(icons)) = (self.state.upgrade(), browser, icons) else { return };
            let (Some(frame), Some(host)) = (browser.main_frame(), browser.host()) else { return };
            let page_url = CefString::from(&frame.url()).to_string();
            let Ok(page) = Url::parse(&page_url) else { return };
            let origin = page.origin();
            // The CEF iterator uses the mutable list wrapper supplied to this callback.
            let Some(icon) = std::mem::take(icons).into_iter().find(|icon| {
                is_web_url(icon) && Url::parse(icon).is_ok_and(|icon| icon.origin() == origin)
            }) else { return };
            let id = browser.identifier();
            if state.begin_favicon(id, &page_url) {
                let mut callback = Favicon::new(self.state.clone(), id, page_url);
                host.download_image(Some(&CefString::from(icon.as_str())), 1, 16, 0, Some(&mut callback));
            }
        }
    }
}

wrap_download_image_callback! {
    struct Favicon { state: Weak<WindowState>, browser_id: i32, page_url: String }
    impl DownloadImageCallback {
        fn on_download_image_finished(
            &self,
            _image_url: Option<&CefString>,
            _status: i32,
            image: Option<&mut Image>,
        ) {
            if let (Some(state), Some(image)) = (self.state.upgrade(), image)
                && image.is_empty() == 0
            {
                state.favicon_loaded(self.browser_id, &self.page_url, image.clone());
            }
        }
    }
}

wrap_load_handler! {
    struct LoadEvents { state: Weak<WindowState> }
    impl LoadHandler {
        fn on_loading_state_change(
            &self,
            _browser: Option<&mut Browser>,
            _loading: i32,
            _back: i32,
            _forward: i32,
        ) {
            if let Some(state) = self.state.upgrade() {
                state.schedule_refresh();
            }
        }

        fn on_load_error(
            &self,
            browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            code: Errorcode,
            text: Option<&CefString>,
            _url: Option<&CefString>,
        ) {
            if code != Errorcode::ABORTED
                && frame.is_some_and(|frame| frame.is_main() != 0)
                && let (Some(state), Some(id)) = (self.state.upgrade(), browser_id(browser))
            {
                let detail = text.map(ToString::to_string).unwrap_or_default();
                state.page_status(id, &format!("Page could not load · {detail}"));
            }
        }
    }
}

wrap_request_handler! {
    struct NavigationPolicy { state: Weak<WindowState> }
    impl RequestHandler {
        fn on_before_browse(
            &self,
            browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            request: Option<&mut Request>,
            user_gesture: i32,
            is_redirect: i32,
        ) -> i32 {
            let (Some(frame), Some(request)) = (frame, request) else { return 0 };
            if frame.is_main() == 0 {
                return 0;
            }
            let url = CefString::from(&request.url()).to_string();
            if is_page_url(&url) || url == "chrome-error://chromewebdata/" {
                return 0;
            }
            if let (Some(state), Some(id)) = (self.state.upgrade(), browser_id(browser)) {
                if user_gesture != 0
                    && is_redirect == 0
                    && CefString::from(&frame.url()).to_string() == HOME
                    && CefString::from(&request.method()).to_string() == "GET"
                    && let Some(input) = home_query(&url)
                {
                    state.submit_home_search(id, input);
                } else {
                    state.page_status(id, "Navigation to that address was blocked");
                }
            }
            1
        }

        fn on_open_urlfrom_tab(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            target: Option<&CefString>,
            disposition: WindowOpenDisposition,
            user_gesture: i32,
        ) -> i32 {
            if !matches!(
                disposition,
                WindowOpenDisposition::NEW_FOREGROUND_TAB | WindowOpenDisposition::NEW_BACKGROUND_TAB
            ) {
                return 0;
            }
            if let (Some(state), Some(target)) = (self.state.upgrade(), target)
                && user_gesture != 0
            {
                let url = target.to_string();
                if is_web_url(&url) {
                    state.dispatch(Action::OpenInTab(url, disposition == WindowOpenDisposition::NEW_FOREGROUND_TAB));
                }
            }
            1
        }

        fn on_render_process_terminated(
            &self,
            browser: Option<&mut Browser>,
            _status: TerminationStatus,
            _error_code: i32,
            _error: Option<&CefString>,
        ) {
            if let (Some(state), Some(id)) = (self.state.upgrade(), browser_id(browser)) {
                state.page_status(id, "Page renderer exited. Reload to try again.");
            }
        }
    }
}

wrap_permission_handler! {
    struct SitePermissions { state: Weak<WindowState> }
    impl PermissionHandler {
        fn on_request_media_access_permission(
            &self,
            browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _origin: Option<&CefString>,
            _permissions: u32,
            callback: Option<&mut MediaAccessCallback>,
        ) -> i32 {
            if let Some(callback) = callback {
                callback.cancel();
            }
            if let (Some(state), Some(id)) = (self.state.upgrade(), browser_id(browser)) {
                state.page_status(id, "Camera, microphone, or screen access blocked");
            }
            1
        }

        fn on_show_permission_prompt(
            &self,
            browser: Option<&mut Browser>,
            _prompt: u64,
            _origin: Option<&CefString>,
            _permissions: u32,
            callback: Option<&mut PermissionPromptCallback>,
        ) -> i32 {
            if let Some(callback) = callback {
                callback.cont(PermissionRequestResult::DENY);
            }
            if let (Some(state), Some(id)) = (self.state.upgrade(), browser_id(browser)) {
                state.page_status(id, "Site permission blocked");
            }
            1
        }
    }
}

wrap_download_handler! {
    struct DownloadEvents { state: Weak<WindowState> }
    impl DownloadHandler {
        fn can_download(
            &self,
            _browser: Option<&mut Browser>,
            url: Option<&CefString>,
            _method: Option<&CefString>,
        ) -> i32 {
            i32::from(url.is_some_and(|url| web_download(&url.to_string())))
        }

        fn on_before_download(
            &self,
            _browser: Option<&mut Browser>,
            item: Option<&mut DownloadItem>,
            suggested_name: Option<&CefString>,
            callback: Option<&mut BeforeDownloadCallback>,
        ) -> i32 {
            let (Some(state), Some(item), Some(callback)) = (self.state.upgrade(), item, callback) else { return 1 };
            if state.window_closing() || !web_download(&CefString::from(&item.url()).to_string()) {
                return 1;
            }
            let suggested = suggested_name.map(ToString::to_string).unwrap_or_default();
            let filename = Path::new(&suggested).file_name().and_then(|name| name.to_str()).unwrap_or("download");
            let path = dirs::download_dir().map(|dir| dir.join(filename));
            let path = path.as_deref().and_then(Path::to_str).map(CefString::from);
            state.download_changed(item.id(), filename.to_owned(), "Choose a save location".into());
            callback.cont(path.as_ref(), 1);
            1
        }

        fn on_download_updated(
            &self,
            _browser: Option<&mut Browser>,
            item: Option<&mut DownloadItem>,
            _callback: Option<&mut DownloadItemCallback>,
        ) {
            let (Some(state), Some(item)) = (self.state.upgrade(), item) else { return };
            let name = CefString::from(&item.suggested_file_name()).to_string();
            let status = if item.is_complete() != 0 {
                "Complete".into()
            } else if item.is_canceled() != 0 {
                "Canceled".into()
            } else if item.is_interrupted() != 0 {
                "Interrupted".into()
            } else if item.percent_complete() >= 0 {
                format!("{}%", item.percent_complete())
            } else {
                "Downloading".into()
            };
            state.download_changed(item.id(), name, status);
        }
    }
}

wrap_keyboard_handler! {
    struct PageKeys { state: Weak<WindowState> }
    impl KeyboardHandler {
        fn on_pre_key_event(
            &self,
            _browser: Option<&mut Browser>,
            event: Option<&KeyEvent>,
            _os_event: Option<&mut sys::XEvent>,
            _shortcut: Option<&mut i32>,
        ) -> i32 {
            let (Some(state), Some(event)) = (self.state.upgrade(), event) else { return 0 };
            if !matches!(event.type_, KeyEventType::RAWKEYDOWN | KeyEventType::KEYDOWN)
                || event.modifiers & sys::cef_event_flags_t::EVENTFLAG_ALTGR_DOWN.0 != 0
            {
                return 0;
            }
            let modifiers = event.modifiers;
            let shift = i32::from(modifiers & sys::cef_event_flags_t::EVENTFLAG_SHIFT_DOWN.0 != 0);
            let ctrl = i32::from(modifiers & sys::cef_event_flags_t::EVENTFLAG_CONTROL_DOWN.0 != 0);
            let alt = i32::from(modifiers & sys::cef_event_flags_t::EVENTFLAG_ALT_DOWN.0 != 0);
            for (action, key, need_shift, need_ctrl, need_alt) in SHORTCUTS {
                if (event.windows_key_code, shift, ctrl, alt) == (*key, *need_shift, *need_ctrl, *need_alt) {
                    state.dispatch(action.clone());
                    return 1;
                }
            }
            0
        }
    }
}

pub(crate) fn register_home() -> bool {
    register_scheme_handler_factory(
        Some(&CefString::from("crest")),
        Some(&CefString::from("home")),
        Some(&mut HomeFactory::new()),
    ) != 0
}

wrap_scheme_handler_factory! {
    struct HomeFactory;
    impl SchemeHandlerFactory {
        fn create(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _scheme: Option<&CefString>,
            request: Option<&mut Request>,
        ) -> Option<ResourceHandler> {
            use cef::wrapper::{
                byte_read_handler::{ByteReadHandler, ByteStream},
                stream_resource_handler::StreamResourceHandler,
            };
            use std::sync::{Arc, Mutex};

            let request = request?;
            if CefString::from(&request.method()).to_string() != "GET" {
                return None;
            }
            let (mime, bytes): (&str, &[u8]) = match CefString::from(&request.url()).to_string().as_str() {
                HOME => ("text/html", include_bytes!("home.html")),
                HOME_WALLPAPER => ("image/webp", include_bytes!("deep-water.webp")),
                _ => return None,
            };
            let mut reader = ByteReadHandler::new(Arc::new(Mutex::new(ByteStream::new(bytes.to_vec()))));
            let stream = stream_reader_create_for_handler(Some(&mut reader))?;
            let mut headers = CefStringMultimap::new();
            headers.append("Content-Security-Policy", "default-src 'none'; style-src 'unsafe-inline'; img-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'");
            headers.append("X-Content-Type-Options", "nosniff");
            Some(StreamResourceHandler::new(200, "OK".into(), mime.into(), Some(headers), Some(stream)))
        }
    }
}
