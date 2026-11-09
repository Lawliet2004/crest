mod address;
mod profile;

pub use address::{BLANK, HOME, SearchEngine, is_page_url, is_web_url, resolve};
pub use profile::{MAX_TABS, Page, Profile, Settings, TabLayout};
