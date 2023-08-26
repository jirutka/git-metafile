use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

pub fn repo_root() -> io::Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;

    let path = str::from_utf8(&output.stdout)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        .map(str::trim)
        .map(PathBuf::from)?;

    if path.is_dir() {
        Ok(path)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("directory {:?} does not exist", path),
        ))
    }
}

pub fn staged_files() -> io::Result<BTreeSet<PathBuf>> {
    let deleted_files = BTreeSet::from_iter(ls_files(&["--deleted"])?);
    let mut paths = BTreeSet::new();

    for mut path in ls_files(&[""; 0])? {
        // Skip files that have been deleted from FS.
        if deleted_files.contains(&path) {
            continue;
        }
        // Insert also all intermediate directories.
        loop {
            paths.insert(path.clone());
            if !path.pop() {
                break;
            }
        }
    }
    paths.remove(Path::new(""));

    Ok(paths)
}

fn ls_files<S: AsRef<OsStr>>(args: &[S]) -> io::Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .args(["ls-files", "--full-name", "-z"])
        .args(args)
        .env("LC_ALL", "C")
        .output()?;

    let files = output
        .stdout
        .split(|b| *b == 0)
        .map(OsStr::from_bytes)
        .map(PathBuf::from)
        .collect::<Vec<PathBuf>>();

    Ok(files)
}
