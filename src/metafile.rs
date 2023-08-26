use std::convert::From;
use std::fmt::{self, Display};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Error as IoError, Read, Write};
use std::num::ParseIntError;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;

use boolinator::Boolinator;
use nix::unistd::{Gid, Uid};
use quick_error::quick_error;

use crate::iter_ext::IteratorExt;

const METAFILE_VERSION: u32 = 1;
const METAFILE_HEADER: &str = "#%GIT-METAFILE";

pub type Result<T> = StdResult<T, MetafileError>;

quick_error! {
    #[derive(Debug)]
    pub enum MetafileError {
        Io(err: IoError) {
            from()
            source(err)
            display("{}", err)
        }
        Malformed(msg: String) {
            from(err: ParseIntError) -> (format!("{}", err))
            display("malformed metafile: {}", msg)
        }
        UnsupportedVersion(version: u32) {
            display("unsupported metafile version \"{}\"", version)
        }
    }
}

#[derive(Debug)]
pub struct Metafile {
    pub entries: Vec<MetafileEntry>,
}

#[derive(Debug, PartialEq)]
pub struct MetafileEntry {
    pub path: PathBuf,
    pub mode: u32,
    pub uid: Uid,
    pub gid: Gid,
}

impl Metafile {
    pub fn new(entries: Vec<MetafileEntry>) -> Metafile {
        Metafile { entries }
    }

    pub fn read<P: AsRef<Path>>(path: P, strict: bool) -> Result<Metafile> {
        File::open(path)
            .map_err(MetafileError::from)
            .and_then(|f| Self::parse(f, strict))
    }

    pub fn parse<R: Read>(source: R, strict: bool) -> Result<Metafile> {
        let mut lines = BufReader::new(source)
            .lines()
            .enumerate()
            // (usize, Result<T, E>) -> Result<(usize, T), (usize, E)>
            .map(|(i, r)| r.map(|s| (i, s)).map_err(|e| (i, e)))
            // Result<(usize, T), (usize, E)> -> Ok((usize, T))
            .filter_map(StdResult::ok); // XXX: report encoding errors?

        let version = lines
            .next()
            .and_then(|(_, s)| s.starts_with(METAFILE_HEADER).as_some(s))
            .and_then(|s| {
                s.trim_start_matches(METAFILE_HEADER)
                    .trim()
                    .parse::<u32>()
                    .ok()
            })
            .ok_or_else(|| MetafileError::Malformed("missing or malformed header".into()))?;

        if version != 1 {
            return Err(MetafileError::UnsupportedVersion(version));
        }

        let entries = lines
            .filter(|(_, ref s)| !s.is_empty() && !s.starts_with('#'))
            // (usize, &str) -> Result<Metafile, (usize, MetafileError)>
            .map(|(i, s)| MetafileEntry::parse(s).map_err(|e| (i, e)))
            // TODO: refactor to be pure
            .inspect_err(|&(i, ref e)| err!("{} at line {}", &e, i + 1));

        let entries = if strict {
            entries
                .map(|r| r.map_err(|(_, e)| e))
                .collect::<Result<_>>()?
        } else {
            entries.filter_map(StdResult::ok).collect::<Vec<_>>()
        };

        Ok(Metafile { entries })
    }

    pub fn write<P: AsRef<Path>>(self, path: P) -> Result<()> {
        File::create(path)
            .map_err(MetafileError::from)
            .map(BufWriter::new)
            .and_then(|mut buf| self.dump(&mut buf))
    }

    pub fn dump(self, dest: &mut dyn Write) -> Result<()> {
        write!(
            dest,
            "{} {}\n# <path>\t<mode>\t<uid>\t<gid>\n",
            METAFILE_HEADER, METAFILE_VERSION
        )?;

        for entry in self.entries {
            entry.dump(dest)?;
        }
        dest.write_all(b"# vim: set ts=16")?;
        dest.flush()?;

        Ok(())
    }
}

impl MetafileEntry {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<MetafileEntry> {
        let path = path.as_ref();

        path.symlink_metadata()
            .map_err(MetafileError::from)
            .map(|meta| MetafileEntry {
                path: path.into(),
                mode: meta.mode(),
                uid: Uid::from_raw(meta.uid()),
                gid: Gid::from_raw(meta.gid()),
            })
    }

    pub fn parse<S: Into<String>>(line: S) -> Result<MetafileEntry> {
        let line = line.into();
        let fields = line.split('\t').collect::<Vec<_>>();

        if fields.len() < 4 {
            return Err(MetafileError::Malformed("expected 4 fields".into()));
        }

        Ok(MetafileEntry {
            path: fields[0].into(),
            mode: u32::from_str_radix(fields[1], 8)?,
            uid: Uid::from_raw(fields[2].parse()?),
            gid: Gid::from_raw(fields[3].parse()?),
        })
    }

    pub fn dump(&self, dest: &mut dyn Write) -> Result<()> {
        writeln!(dest, "{}", self).map_err(MetafileError::from)
    }
}

impl Display for MetafileEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\t{:o}\t{}\t{}",
            self.path.display(),
            self.mode,
            self.uid,
            self.gid
        )
    }
}
