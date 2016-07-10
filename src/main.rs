extern crate argparse;
extern crate boolinator;
#[macro_use] extern crate quick_error;
extern crate nix;

mod git;
mod iter_ext;
#[macro_use] mod macros;
mod metafile;

use std::fs::{self, Permissions};
use std::io::Write;
use std::iter::{IntoIterator, Iterator};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::str;
use std::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};

use argparse::{ArgumentParser, Parse, Print, Store, StoreTrue};
use nix::unistd;

use metafile::{Metafile, MetafileEntry};


const PRG_VERSION: &'static str =
    concat![env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION")];
const DEFAULT_FILE_NAME: &'static str = ".metafile";

static VERBOSE_ENABLED: AtomicBool = ATOMIC_BOOL_INIT;


// TODO refactor and improve error handling
fn apply(mf_path: &Path, parse_strict: bool) {

    let entries = match Metafile::read(mf_path, parse_strict) {
        Ok(mf) => mf.entries.into_iter(),
        Err(e) => die!("failed to read metafile: {}", e),
    };

    for entry in entries {
        match MetafileEntry::from_path(&entry.path) {
            Ok(current) => {
                let expected = entry;
                let path = &expected.path;

                if expected.mode != current.mode {
                    info!("{:?}: change mode {:o} -> {:o}", &path, current.mode, expected.mode);
                    fs::set_permissions(path, Permissions::from_mode(expected.mode))
                        .unwrap_or_else(|e| err!("{:?}: {}", path, e));
                }
                if expected.uid != current.uid {
                    info!("{:?}: change owner {} -> {}", &path, current.uid, expected.uid);
                    unistd::chown(path, Some(expected.uid), None)
                        .unwrap_or_else(|e| err!("{:?}: {}", path, e));
                }
                if expected.gid != current.gid {
                    info!("{:?}: change group {} -> {}", &path, current.gid, expected.gid);
                    unistd::chown(path, None, Some(expected.gid))
                        .unwrap_or_else(|e| err!("{:?}: {}", path, e));
                }
            },
            Err(e) => err!("warning: {}", e),
        }
    }
}

fn save(mf_path: &Path) {
    let entries = git::staged_files()
        .unwrap_or_else(|e| die!("failed to get list of staged files: {}", e))
        .into_iter()
        .map(|path| MetafileEntry::from_path(&path)
                        .unwrap_or_else(|e| die!("{}: {}", e, &path.display())))
        .collect();

    Metafile::new(entries)
        .write(&mf_path)
        .unwrap_or_else(|e| {
            die!("failed to write metafile {}: {}", mf_path.display(), e)
        });
}


fn main() {
    let mut command = String::new();
    let mut file_path = PathBuf::new();
    let mut strict = false;
    let mut verbose = false;

    tap! { ArgumentParser::new();
        .set_description("Store and restore files metadata (mode, owner and group) in a git repository."),

        .refer(&mut command).required()
            .add_argument("COMMAND", Store, r#"save, or apply"#),

        .refer(&mut file_path)
            .add_option(&["-f", "--file"], Parse,
                r#"Path of the metafile (default is ".metafile" in git's top-level directory)"#),

        .refer(&mut strict)
            .add_option(&["-s", "--strict"], StoreTrue,
                "Switch parser to strict mode, i.e. exit after first error"),

        .refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "Be verbose"),

        .add_option(&["-V", "--version"], Print(PRG_VERSION.into()), "Show version"),

        .parse_args_or_exit(),
    };

    VERBOSE_ENABLED.store(verbose, Ordering::Relaxed);

    if file_path.as_os_str().is_empty() {
        file_path = match git::repo_root() {
            Ok(path) => path.join(DEFAULT_FILE_NAME),
            Err(e) => die!("could not find git's top-level directory: {}", e),
        }
    }

    match &*command {
        "apply" => apply(&file_path, strict),
        "save" => save(&file_path),
        _ => die!("unknown command {}", command),
    }
}
