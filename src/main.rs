mod git;
mod iter_ext;
#[macro_use]
mod macros;
mod metafile;

use std::fs::{self, Permissions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};

use argp::FromArgs;
use nix::unistd;

use metafile::{Metafile, MetafileEntry};

const PRG_VERSION: &str = concat![env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION")];
const DEFAULT_FILE_NAME: &str = ".metafile";

static QUIET_ENABLED: AtomicBool = AtomicBool::new(false);

/// Store and restore files metadata (mode, owner and group) in a git
/// repository.
#[derive(FromArgs)]
struct Args {
    /// Path of the metafile (default is ".metafile" in git's top-level
    /// directory).
    #[argp(option, short = 'f', default = "Default::default()")]
    file: PathBuf,

    /// Be quite, suppress info messages.
    #[argp(switch, short = 'q')]
    quiet: bool,

    /// Switch parser to strict mode, i.e. exit after first error.
    #[argp(switch, short = 's')]
    strict: bool,

    /// Show version and exit.
    #[argp(switch, short = 'V')]
    version: bool,

    /// save or apply
    #[argp(positional, default = "Default::default()")]
    command: String,
}

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
                let path = &expected.path.display();

                if expected.mode != current.mode {
                    info!(
                        "{}: change mode {:o} -> {:o}",
                        &path, current.mode, expected.mode
                    );
                    fs::set_permissions(&expected.path, Permissions::from_mode(expected.mode))
                        .unwrap_or_else(|e| err!("{}: {}", &path, e));
                }
                if expected.uid != current.uid {
                    info!(
                        "{}: change owner {} -> {}",
                        &path, current.uid, expected.uid
                    );
                    unistd::chown(&expected.path, Some(expected.uid), None)
                        .unwrap_or_else(|e| err!("{}: {}", &path, e));
                }
                if expected.gid != current.gid {
                    info!(
                        "{}: change group {} -> {}",
                        &path, current.gid, expected.gid
                    );
                    unistd::chown(&expected.path, None, Some(expected.gid))
                        .unwrap_or_else(|e| err!("{}: {}", &path, e));
                }
            }
            Err(e) => err!("warning: {}", e),
        }
    }
}

fn save(mf_path: &Path) {
    let entries = git::staged_files()
        .unwrap_or_else(|e| die!("failed to get list of staged files: {}", e))
        .into_iter()
        .map(|path| {
            MetafileEntry::from_path(&path).unwrap_or_else(|e| die!("{}: {}", e, &path.display()))
        })
        .collect();

    Metafile::new(entries)
        .write(mf_path)
        .unwrap_or_else(|e| die!("failed to write metafile {}: {}", mf_path.display(), e));
}

fn main() {
    let mut args: Args = argp::parse_args_or_exit(argp::DEFAULT);

    if args.version {
        println!("{}", PRG_VERSION);
        exit(0);
    }

    QUIET_ENABLED.store(args.quiet, Ordering::Relaxed);

    if args.file.as_os_str().is_empty() {
        args.file = match git::repo_root() {
            Ok(path) => path.join(DEFAULT_FILE_NAME),
            Err(e) => die!("could not find git's top-level directory: {}", e),
        }
    }

    match &*args.command {
        "apply" => apply(&args.file, args.strict),
        "save" => save(&args.file),
        "" => die!("no command specified"),
        _ => die!("unknown command {}", args.command),
    }
}
