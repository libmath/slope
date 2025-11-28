use std::path::{Path, PathBuf};
use std::{fs, io};

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

/// Recursively walks through the contents of a directory. Ignores a
/// fixed set of directories.
pub fn walk<F>(
    root: &Path,
    mut action: F,
    ignore_dirs: &[&str],
) -> io::Result<()>
where
    F: FnMut(PathBuf) -> (),
{
    let mut stack = vec![];
    for dir_ent in fs::read_dir(root)? {
        stack.push(dir_ent?);
    }
    loop {
        let Some(dir_ent) = stack.pop() else { return Ok(()) };
        let ft = dir_ent.file_type()?;

        if ft.is_dir() {
            let dir = dir_ent.path();
            let Some(filename) = dir.file_name().and_then(|v| v.to_str())
            else {
                continue;
            };
            if ignore_dirs.contains(&filename) {
                continue;
            }
            for dir_ent in fs::read_dir(dir)? {
                stack.push(dir_ent?);
            }
        } else if ft.is_file() {
            action(dir_ent.path());
        }
    }
}

/// Recursively obtains all the files with extension "lean" under `root`, while
/// ignoring directories that are contained in `ignore_dirs`. Also, ignores
/// "lakefile.lean".
pub fn get_lean_files_in_dir(
    root: &Path,
    ignore_dirs: &[&str],
) -> io::Result<Vec<PathBuf>> {
    let mut lean_files = vec![];
    walk(
        root,
        |file| {
            let Some(file_stem) = file.file_stem() else { return };
            if file_stem == "lakefile" {
                return;
            }
            if file.extension().map_or(false, |v| v == "lean") {
                lean_files.push(file)
            }
        },
        ignore_dirs,
    )?;
    Ok(lean_files)
}
