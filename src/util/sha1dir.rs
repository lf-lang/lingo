// Full credit goes to David Tolnay https://github.com/dtolnay/sha1dir
// I submitted a pull request to make the function available as a library
// function, but this pull request (#19) was rejected, hence this file.

#![allow(
    clippy::cast_possible_truncation,
    clippy::let_underscore_untyped,
    clippy::needless_collect,
    clippy::needless_pass_by_value,
    clippy::uninlined_format_args,
    clippy::unnecessary_wraps,
    clippy::unseparated_literal_suffix
)]

use parking_lot::Mutex;
use rayon::{Scope, ThreadPoolBuilder};
use sha1::{Digest, Sha1};
use std::error::Error;
use std::fmt::{self, Display};
use std::fs::{self, File, Metadata};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Once;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn die<P: AsRef<Path>, E: Display>(path: P, error: E) -> ! {
    static DIE: Once = Once::new();

    DIE.call_once(|| {
        let path = path.as_ref().display();
        let _ = writeln!(io::stderr(), "{}: {}", path, error,);
        process::exit(1);
    });

    unreachable!()
}

pub fn configure_thread_pool(threads: usize) {
    let result = ThreadPoolBuilder::new().num_threads(threads).build_global();

    // This is the only time the thread pool is initialized.
    result.unwrap();
}

pub fn canonicalize<P: AsRef<Path>>(path: P) -> PathBuf {
    match fs::canonicalize(&path) {
        Ok(canonical) => canonical,
        Err(error) => die(path, error),
    }
}

pub struct Checksum {
    bytes: Mutex<[u8; 20]>,
}

impl Checksum {
    fn new() -> Self {
        Checksum {
            bytes: Mutex::new([0u8; 20]),
        }
    }
}

impl Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in self.bytes.lock().as_ref() {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

impl Checksum {
    fn put(&self, rhs: Sha1) {
        for (lhs, rhs) in self.bytes.lock().iter_mut().zip(rhs.finalize()) {
            *lhs ^= rhs;
        }
    }
}

fn get_file_as_byte_vec(filename: &Path) -> Vec<u8> {
    let mut f = File::open(filename).expect("no file found");
    let metadata = fs::metadata(filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer).expect("buffer overflow");

    buffer
}

pub fn checksum_current_dir(label: &Path, ignore_unknown_filetypes: bool) -> Checksum {
    let checksum = Checksum::new();
    rayon::scope(|scope| {
        if let Err(error) = (|| -> Result<()> {
            for child in Path::new(".").read_dir()? {
                let child = child?;
                scope.spawn({
                    let checksum = &checksum;
                    move |scope| {
                        entry(
                            scope,
                            label,
                            checksum,
                            Path::new(&child.file_name()),
                            ignore_unknown_filetypes,
                        );
                    }
                });
            }
            Ok(())
        })() {
            die(label, error);
        }
    });
    checksum
}

fn entry<'scope>(
    scope: &Scope<'scope>,
    base: &'scope Path,
    checksum: &'scope Checksum,
    path: &Path,
    ignore_unknown_filetypes: bool,
) {
    let metadata = match path.symlink_metadata() {
        Ok(metadata) => metadata,
        Err(error) => die(base.join(path), error),
    };

    let file_type = metadata.file_type();
    let result = if file_type.is_file() {
        file(checksum, path, metadata)
    } else if file_type.is_symlink() {
        symlink(checksum, path)
    } else if file_type.is_dir() {
        dir(scope, base, checksum, path, ignore_unknown_filetypes)
    } else if ignore_unknown_filetypes {
        Ok(())
    } else {
        die(base.join(path), "Unsupported file type");
    };

    if let Err(error) = result {
        die(base.join(path), error);
    }
}

fn file(checksum: &Checksum, path: &Path, metadata: Metadata) -> Result<()> {
    let mut sha = begin(path, b'f');

    // Enforced by memmap: "memory map must have a non-zero length"
    if metadata.len() > 0 {
        sha.update(get_file_as_byte_vec(path));
    }

    checksum.put(sha);

    Ok(())
}

fn symlink(checksum: &Checksum, path: &Path) -> Result<()> {
    let mut sha = begin(path, b'l');
    sha.update(path.read_link()?.as_os_str().as_encoded_bytes());
    checksum.put(sha);

    Ok(())
}

fn dir<'scope>(
    scope: &Scope<'scope>,
    base: &'scope Path,
    checksum: &'scope Checksum,
    path: &Path,
    ignore_unknown_filetypes: bool,
) -> Result<()> {
    let sha = begin(path, b'd');
    checksum.put(sha);

    for child in path.read_dir()? {
        let child = child?.path();
        scope.spawn(move |scope| entry(scope, base, checksum, &child, ignore_unknown_filetypes));
    }

    Ok(())
}

fn begin(path: &Path, kind: u8) -> Sha1 {
    let mut sha = Sha1::new();
    let path_bytes = path.as_os_str().as_encoded_bytes();
    sha.update([kind]);
    sha.update((path_bytes.len() as u32).to_le_bytes());
    sha.update(path_bytes);
    sha
}

#[test]
fn test_cli() {
    <Opt as clap::CommandFactory>::command().debug_assert();
}
