use ahash::AHashMap;
use once_cell::sync::Lazy;

pub type Result<'a> = std::result::Result<Option<&'a str>, ()>;

pub struct Database<'a> {
    inner: AHashMap<&'a str, Option<&'a str>>,
}

impl<'a> Database<'a> {
    pub fn shared() -> &'static Self {
        static DATABASE: Lazy<Database<'_>> = Lazy::new(|| {
            #[cfg(windows)]
            let data = include!("./win32.inc.rs");

            #[cfg(any(target_os = "linux", target_os = "android"))]
            let data = include!("./linux.inc.rs");

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            let data = include!("./darwin.inc.rs");

            let mut map = AHashMap::with_capacity(data.len());
            for (item, doc) in data {
                map.insert(item, doc);
            }
            Database { inner: map }
        });
        &DATABASE
    }

    pub fn try_path(&self, path: &str) -> Result<'a> {
        self.inner.get(path).copied().ok_or(())
    }

    pub fn try_module_item(&self, module: &str, item: &str) -> Result<'a> {
        self.try_path(&format!("{}.{}", module, item))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_module_item() {
        let doc = Database::shared()
            .try_module_item("array", "_array_reconstructor")
            .unwrap();
        assert!(doc.is_some());
    }
}
