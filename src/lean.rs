use core::fmt;
use std::path::PathBuf;

/// Represents one `*.lean` file.
#[derive(PartialEq, Eq)]
pub struct Lean {
    /// The absolute path to the directory where the `relpath` can bring us the
    /// rest of the way to the `*.lean` file in question.
    root: PathBuf,

    /// The import path of the Lean file, relative to `root`.
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
