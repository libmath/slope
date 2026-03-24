use std::fs;
use std::path::{Path, PathBuf};

#[allow(unused)]
fn is_lakefile(filepath: &Path) -> bool {
    if let Some("lakefile.toml" | "lakefile.lean") = filepath.to_str() {
        return true;
    }
    false
}

fn find_lake_root(cwd: &Path) -> Option<PathBuf> {
    let mut cursor = cwd.to_path_buf();
    let mut i: u8 = 64; // maximum number of `cd ..`s.
    while i > 0 {
        i -= 1;
        for filename in ["lakefile.lean", "lakefile.toml"] {
            cursor.push(filename);
            let metadata = fs::metadata(&cursor);
            cursor.pop();
            match metadata {
                Ok(m) if m.is_file() => return Some(cursor),
                _ => {}
            }
        }
        if !cursor.pop() {
            break;
        }
    }
    None
}

/// A cached filesytem manager.
pub struct FilesystemManager {
    /// Absolute path to current directory (cached).
    cwd: Option<PathBuf>,

    /// Absolute path to the root of the lake project (cached).
    /// The first parent directory that contains `lakefile.lean`.
    absolute_lake_root: Option<PathBuf>,
}

impl FilesystemManager {
    pub fn new() -> Self {
        Self { cwd: None, absolute_lake_root: None }
    }

    /// Absolute path to current directory (cached).
    pub fn cwd(&mut self) -> PathBuf {
        if let None = self.cwd {
            self.cwd = std::env::current_dir().ok();
        }
        self.cwd.as_ref().unwrap().to_path_buf()
    }

    /// Absolute path to the root of the lake project (cached).
    /// The first parent directory (relative to CWD) that contains `lakefile.lean`.
    pub fn absolute_lake_root(&mut self) -> PathBuf {
        if let None = self.absolute_lake_root {
            self.absolute_lake_root = find_lake_root(&self.cwd())
        }
        let root = self.absolute_lake_root.as_ref();
        root.expect("Unable to locate lake root.").to_path_buf()
    }
}

/// Recursively obtains all the files with extension "lean" under `root`, while
/// ignoring directories that are contained in `ignore_dirs`. Also, ignores
/// "lakefile.lean", and ignores symlinks.
pub fn get_lean_files_in_dir(
    root: &Path,
    ignore_dirs: &[&str],
) -> Vec<PathBuf> {
    let mut lean_files = vec![];
    let it = walkdir::WalkDir::new(root);
    let it = it.into_iter().filter_entry(|entry| {
        let file_name = entry.file_name();
        !ignore_dirs.iter().any(|&v| v == file_name)
    });
    for entry in it {
        let Ok(entry) = entry else { continue };
        let path = entry.path();

        // Skip "lakefile.lean" and "lakefile.toml".
        let Some(file_stem) = path.file_stem() else { continue };
        if file_stem == "lakefile" {
            continue;
        }

        // Keep only files that are "*.lean".
        let Some(extension) = path.extension() else { continue };
        if extension != "lean" {
            continue;
        }

        if entry.path_is_symlink() {
            continue;
        }

        lean_files.push(entry.into_path())
    }
    lean_files
}
