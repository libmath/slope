use std::path::{Path, PathBuf};
use std::{fs, io};

/// A cached filesytem manager.
pub struct Fsm {
    /// Absolute path to current directory (cached).
    cwd: Option<PathBuf>,

    /// Absolute path to the root of the lake project (cached).
    /// The first parent directory that contains `lakefile.lean`.
    absolute_lake_root: Option<PathBuf>,
}

impl Fsm {
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
            self.absolute_lake_root =
                Some(first_parent_that_contains(&self.cwd(), "lakefile.lean"));
        }
        self.absolute_lake_root.as_ref().unwrap().to_path_buf()
    }
}

/// Gets the the absolute path of the first parent directory that contains a
/// particular file. Search upwards from the `cwd` parameter.
fn first_parent_that_contains(cwd: &Path, filename: &str) -> PathBuf {
    let mut buf = cwd.join(filename);
    let mut i: u8 = 9;
    while i > 0 {
        i -= 1;
        let Ok(m) = fs::metadata(&buf) else {
            buf.pop();
            buf.set_file_name(filename);
            continue;
        };
        if m.is_file() {
            buf.pop();
            return buf;
        }
    }
    panic!("{filename} not found in parent dirs.");
}

/// Recursively walks through the contents of a directory. Ignores a
/// fixed set of directories.
pub fn walk<F>(root: &Path, mut action: F, ignore: &[&str]) -> io::Result<()>
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
            if ignore.contains(&filename) {
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

pub fn get_lean_files_in_dir(
    root: &Path,
    ignore: &[&str],
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
        ignore,
    )?;
    Ok(lean_files)
}
