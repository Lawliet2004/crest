use crate::{HOME, SearchEngine, is_page_url, is_web_url};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Write},
    path::Path,
};

pub const MAX_TABS: usize = 24;
const MAX_HISTORY: usize = 500;
const MAX_BOOKMARKS: usize = 250;
const MAX_PROFILE_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TabLayout {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Settings {
    pub tab_layout: TabLayout,
    pub search_engine: SearchEngine,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Page {
    pub url: String,
    pub title: String,
}

impl Page {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            title: String::new(),
        }
    }

    pub fn label(&self) -> &str {
        if self.title.trim().is_empty() {
            if self.url == HOME || self.url == crate::BLANK {
                "New tab"
            } else {
                &self.url
            }
        } else {
            &self.title
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Profile {
    pub settings: Settings,
    pub tabs: Vec<Page>,
    pub selected: usize,
    pub history: Vec<Page>,
    pub bookmarks: Vec<Page>,
}

impl Profile {
    pub fn load(path: &Path) -> io::Result<Self> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
        };
        if file.metadata()?.len() > MAX_PROFILE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "profile is too large",
            ));
        }
        let mut profile: Self = serde_json::from_reader(BufReader::new(file))?;
        profile.tabs.truncate(MAX_TABS);
        for page in &mut profile.tabs {
            if !is_page_url(&page.url) {
                *page = Page::new(HOME);
            }
        }
        profile.selected = profile.selected.min(profile.tabs.len().saturating_sub(1));
        profile.history.retain(|page| is_web_url(&page.url));
        profile.history.truncate(MAX_HISTORY);
        profile.bookmarks.retain(|page| is_web_url(&page.url));
        profile.bookmarks.truncate(MAX_BOOKMARKS);
        Ok(profile)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "profile path needs a directory",
            )
        })?;
        let mut temp = tempfile::NamedTempFile::new_in(parent)?;
        let mut writer = BufWriter::new(temp.as_file_mut());
        serde_json::to_writer(&mut writer, self)?;
        writer.flush()?;
        drop(writer);
        temp.as_file().sync_all()?;
        temp.persist(path)?;
        File::open(parent)?.sync_all()
    }

    pub fn record_visit(&mut self, page: &Page) {
        if is_web_url(&page.url) {
            if let Some(index) = self.history.iter().position(|entry| entry.url == page.url) {
                let existing = self.history.remove(index);
                if page.title.is_empty() {
                    self.history.insert(0, existing);
                    return;
                }
            }
            self.history.insert(0, page.clone());
            self.history.truncate(MAX_HISTORY);
        }
    }

    pub fn update_title(&mut self, page: &Page) {
        if let Some(entry) = self.history.iter_mut().find(|entry| entry.url == page.url) {
            entry.title.clone_from(&page.title);
        }
    }

    pub fn is_bookmarked(&self, url: &str) -> bool {
        self.bookmarks.iter().any(|entry| entry.url == url)
    }

    pub fn toggle_bookmark(&mut self, page: &Page) -> Result<bool, &'static str> {
        if !is_web_url(&page.url) {
            return Err("Open a website before bookmarking");
        }
        if let Some(index) = self
            .bookmarks
            .iter()
            .position(|entry| entry.url == page.url)
        {
            self.bookmarks.remove(index);
            return Ok(false);
        }
        if self.bookmarks.len() == MAX_BOOKMARKS {
            return Err("Bookmark limit reached");
        }
        self.bookmarks.insert(0, page.clone());
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_round_trip_preserves_state_and_replaces_previous_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profile.json");
        assert!(Profile::load(&path).unwrap().tabs.is_empty());
        let page = Page::new("https://example.org/");
        let mut profile = Profile {
            tabs: vec![Page::new(HOME), page.clone()],
            selected: 1,
            ..Profile::default()
        };
        profile.record_visit(&page);
        profile.record_visit(&page);
        assert!(profile.toggle_bookmark(&page).unwrap());
        profile.save(&path).unwrap();
        profile.settings.tab_layout = TabLayout::Vertical;
        profile.save(&path).unwrap();

        let restored = Profile::load(&path).unwrap();
        assert_eq!(restored.tabs, profile.tabs);
        assert_eq!(restored.selected, 1);
        assert_eq!(restored.settings.tab_layout, TabLayout::Vertical);
        assert_eq!(restored.history, std::slice::from_ref(&page));
        assert_eq!(restored.bookmarks, std::slice::from_ref(&page));
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn invalid_persisted_urls_are_dropped_or_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profile.json");
        let invalid = Page::new("file:///etc/passwd");
        let profile = Profile {
            tabs: vec![invalid.clone()],
            selected: usize::MAX,
            bookmarks: vec![invalid],
            ..Profile::default()
        };
        profile.save(&path).unwrap();
        let restored = Profile::load(&path).unwrap();
        assert_eq!(restored.tabs, [Page::new(HOME)]);
        assert_eq!(restored.selected, 0);
        assert!(restored.bookmarks.is_empty());
    }
}
