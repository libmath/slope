mod cli;
mod colors;
mod error;
mod filesystem;
mod fmt;
mod lean;

use error::*;
use filesystem::Fsm;
use lean::Lean;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::process::{Command, ExitCode};
use std::time::Instant;

/// Splits a string by the first utf8 character.
fn split_first_utf8_char(x: &str) -> (&str, &str) {
    let mut i = 1;
    loop {
        match x.split_at_checked(i) {
            Some(v) => return v,
            None => i += 1,
        }
    }
}

fn _main(mut fsm: Fsm) -> Result<()> {
    use cli::{Subcommand as S, *};

    let _t = Instant::now();

    match cli::parse().subcommand {
        S::Generate(Generate { target }) => {
            let lake_root = fsm.absolute_lake_root();
            // Get all the *.lean files under `lake_root`, the root of this
            // project.
            let lean_files = filesystem::get_lean_files_in_dir(
                &lake_root,
                &[".git", ".github", ".lake", "target"],
            )?;
            // Convert these into `Vec<Lean>`
            let mut lean_files: Vec<_> = lean_files
                .iter()
                .filter_map(|v| v.strip_prefix(&lake_root).ok())
                .map(|v| v.with_extension(""))
                .map(|v| Lean::new(lake_root.to_path_buf(), v))
                .collect();
            let target = match target.ends_with('.') {
                true => target.to_string(),
                false => format!("{target}."),
            };
            // Keep only those that starts with the generate target.
            lean_files.retain(|v| match v.import().strip_prefix(&target) {
                Some("") | None => false,
                _ => true,
            });
            if lean_files.is_empty() {
                return Ok(());
            }
            let mut filepath = lake_root.to_path_buf();
            filepath.extend(target.split('.'));
            let filepath = filepath.with_extension("lean");
            let mut f = std::fs::File::create(&filepath)?;
            lean_files.sort_by(|a, b| a.import().cmp(&b.import()));
            for lean_file in lean_files {
                writeln!(f, "import {}", lean_file.import())?;
            }
            writeln!(f, "-- WARNING: THIS FILE IS AUTO-GENERATED.")?;
        }
        S::Check(Check {}) => {
            let lake_root = fsm.absolute_lake_root();
            // Get all the *.lean files under `lake_root`, the root of this
            // project.
            let lean_files = filesystem::get_lean_files_in_dir(
                &lake_root,
                &[".git", ".github", ".lake", "target"],
            )?;

            let mut text = String::with_capacity(2048);
            for lean_file in &lean_files {
                let mut f = std::fs::File::open(lean_file)?;
                text.clear();
                f.read_to_string(&mut text)?;
                let text_stripped = fmt::strip_trailing_whitespace(&text);
                if text_stripped != text {
                    std::fs::write(lean_file, &text_stripped)?;
                    let _ = core::mem::replace(&mut text, text_stripped);
                }
                let text = text.as_str();
                fmt::space_between_import_libraries(lean_file, text)?;
                fmt::line_starts_with(
                    lean_file,
                    text,
                    &["import", "universe", "variable", "namespace"],
                )?;
                // fmt::line_starts_with_theorem_lemma(lean_file, text)?;
            }
        }
        S::Build(Build { targets }) => {
            let lake_root = fsm.absolute_lake_root();
            // Get all the *.lean files under `lake_root`, the root of this
            // project.
            let lean_files = filesystem::get_lean_files_in_dir(
                &lake_root,
                &[".git", ".github", ".lake", "target"],
            )?;
            // Convert these into `Vec<Lean>`
            let mut lean_files: Vec<_> = lean_files
                .iter()
                .filter_map(|v| v.strip_prefix(&lake_root).ok())
                .map(|v| v.with_extension(""))
                .map(|v| Lean::new(lake_root.to_path_buf(), v))
                .collect();

            // Keep only those that are targeted.
            if !targets.is_empty() {
                lean_files.retain(|v| {
                    let import = v.import();
                    targets.iter().any(|t| import.starts_with(t))
                });
            }

            let mut cmd = Command::new("lake");
            cmd.args(["build", "--log-level=warning"]);
            for lf in lean_files {
                cmd.arg(lf.abs_path());
            }

            if !cmd.spawn()?.wait()?.success() {
                return Err(Error::Custom(
                    "lake compilation has errors".to_string(),
                ));
            }
        }
        S::Ripgrep(_) => {
            let cwd = fsm.cwd();
            // Get all the *.lean files under `lake_root`, the root of this
            // project.
            let lean_files = filesystem::get_lean_files_in_dir(
                &cwd,
                &[".git", ".github", ".lake", "target"],
            )?;

            let mut buffer = String::with_capacity(2048);
            for lean_file in lean_files {
                let path = lean_file.strip_prefix(&cwd).unwrap().display();
                let mut lnum = 0;
                let mut f = std::fs::File::open(&lean_file)?;
                buffer.clear();
                f.read_to_string(&mut buffer)?;
                for line in buffer.as_str().lines() {
                    lnum += 1;
                    let Some(line) = line.strip_prefix("--*") else { continue };
                    println!("{path}:{lnum}:{}", line.trim_start());
                }
            }
        }
        S::Graph(_) => {
            let lake_root = fsm.absolute_lake_root();
            let mathlib_dir = lake_root.join(".lake/packages/mathlib");
            let mathlib_lean_files = filesystem::get_lean_files_in_dir(
                &mathlib_dir.join("Mathlib"),
                &[],
            )?;
            let mathlib_lean_files: Vec<_> = mathlib_lean_files
                .iter()
                .filter_map(|v| v.strip_prefix(&mathlib_dir).ok())
                .map(|v| v.with_extension(""))
                .map(|v| Lean::new(mathlib_dir.to_path_buf(), v))
                .collect();

            let mut graph: HashMap<String, Vec<String>> = mathlib_lean_files
                .iter()
                .map(|lf| (lf.import(), vec![]))
                .collect();

            for lf in mathlib_lean_files {
                let key = lf.import();
                let text = std::fs::read_to_string(&lf.abs_path())?;
                for line in text.lines() {
                    let Some(v) = line.strip_prefix("import") else { continue };
                    let line = v.trim().to_string();
                    let adj_list = graph.get_mut(&key).unwrap();
                    if !adj_list.contains(&line) {
                        adj_list.push(line);
                    }
                }
            }
            for (key, val) in graph {
                println!("{key}: {val:?}");
            }
        }
        S::Search(_) => {
            let cwd = fsm.cwd();
            let lean_files = filesystem::get_lean_files_in_dir(&cwd, &[])?;

            let mut raw_buffer = String::new();
            let mut text_buffer = String::new();
            for lf in &lean_files {
                let Some(lf_str) = lf.to_str() else { continue };
                if !lf_str.contains("Torsion") {
                    continue;
                }
                let mut f = std::fs::File::open(lf)?;
                f.read_to_string(&mut raw_buffer)?;

                // Clear out the `/- ... -/` comments in lean.
                let mut raw = raw_buffer.as_str();
                let mut is_comment = false;
                while !raw.is_empty() {
                    match is_comment {
                        true if raw.starts_with("-/") => {
                            is_comment = false;
                            raw = &raw[2..];
                        }
                        true => raw = split_first_utf8_char(raw).1,
                        false if raw.starts_with("/-") => {
                            is_comment = true;
                            raw = &raw[2..];
                        }
                        false => {
                            let (a, b) = split_first_utf8_char(raw);
                            text_buffer.push_str(a);
                            raw = b;
                        }
                    }
                }

                core::mem::swap(&mut raw_buffer, &mut text_buffer);
                text_buffer.clear();

                // Clear out the `-- ...` comments in Lean.
                for line in raw_buffer.lines() {
                    match line.split_once("--") {
                        Some((content, _comment)) => {
                            text_buffer.push_str(content)
                        }
                        None => text_buffer.push_str(line),
                    }
                    text_buffer.push('\n');
                }

                println!("{text_buffer}");

                break;
            }
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let fsm = Fsm::new();
    if let Err(err) = _main(fsm) {
        println!("{err}");
        return ExitCode::FAILURE;
    } else {
        ExitCode::SUCCESS
    }
}
