use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::io::{Error, ErrorKind, Result};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;


pub fn repo_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()?;

    let path = str::from_utf8(&output.stdout)
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))
        .map(str::trim)
        .map(PathBuf::from)?;

    if path.is_dir() {
        Ok(path)
    } else {
        Err(Error::new(ErrorKind::InvalidData,
            format!("directory {:?} does not exist", path)))
    }
}

pub fn staged_files() -> Result<BTreeSet<PathBuf>> {
    ls_files(&[""; 0])
}

//pub fn unstaged_files() -> Result<BTreeSet<PathBuf>> {
//    ls_files(&["--others", "--exclude-standard"])
//}

fn ls_files<S: AsRef<OsStr>>(args: &[S]) -> Result<BTreeSet<PathBuf>> {
    let output = Command::new("git")
        .args(&["ls-files", "--full-name", "-z"])
        .args(args)
        .env("LC_ALL", "C")
        .output()?;

    let files = output.stdout
        .split(|b| *b == 0)
        .map(OsStr::from_bytes)
        .map(PathBuf::from);

    let mut paths = BTreeSet::new();
    for mut path in files {
        // Insert also all intermediate directories.
        loop {
            paths.insert(path.clone());
            if !path.pop() { break; }
        }
    }
    paths.remove(Path::new(""));

    Ok(paths)
}
