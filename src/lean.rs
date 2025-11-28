use core::fmt;
use std::path::PathBuf;

#[derive(PartialEq, Eq)]
pub struct Lean {
    /// The absolute path to the directory where the `path` can bring us the
    /// rest of the way.
    root: PathBuf,

    /// The import path of the Lean file, relative to the root.
    relpath: PathBuf,
}

impl fmt::Debug for Lean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.import())
    }
}

impl Lean {
    pub fn new(root: PathBuf, relpath: PathBuf) -> Self {
        Self { root, relpath }
    }

    /// Get the corresponding `import` string of this file. Really, it's just
    /// converting all '/' to '.', and then stripping the ".lean" suffix.
    pub fn import(&self) -> String {
        // self.relpath.to_str().unwrap().replace('/', ".")
        let components: Vec<_> = self
            .relpath
            .components()
            .filter_map(|v| match v.as_os_str().to_str()? {
                "." | ".." => None,
                v => Some(v),
            })
            .collect();
        components.join(".")
    }

    pub fn abs_path(&self) -> PathBuf {
        self.root.join(self.relpath.with_extension("lean"))
    }
}
