use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    sync::mpsc,
};

use chrono::NaiveDate;
use okane_core::load;

/// Metadata containing the reference to the generated input.
pub struct ExampleInput<T: FileSink> {
    rootdir: PathBuf,
    rootfile: PathBuf,
    cleanup: bool,
    sink: T,
}

impl<T: FileSink> Drop for ExampleInput<T> {
    fn drop(&mut self) {
        if self.cleanup {
            self.sink.cleanup(&self.rootdir);
        } else {
            log::info!("{} is not set, leave the test data as-is", CLEANUP_KEY);
        }
    }
}

/// Error when creating [ExampleInput].
#[derive(Debug, thiserror::Error)]
pub enum ExampleInputError {
    #[error("failed to perform IO")]
    IO(#[from] std::io::Error),
    #[error("invlaid environment variable")]
    EnvVar(#[from] std::env::VarError),
}

const CLEANUP_KEY: &str = "OKANE_BENCH_CLEANUP";

const YEAR_BEGIN: i32 = 2015;
const YEAR_END: i32 = 2025;
const NUM_SUB_FILES: usize = 16;
const NUM_THREADS: usize = 2;
const NUM_TRANSACTIONS_PER_FILE: usize = 500;

pub fn num_transactions() -> u64 {
    ((YEAR_END - YEAR_BEGIN) as u64) * (NUM_SUB_FILES as u64) * (NUM_TRANSACTIONS_PER_FILE as u64)
}

pub fn new_example<T: FileSink + Send>(
    subdir: &Path,
) -> Result<ExampleInput<T>, ExampleInputError> {
    ExampleInput::<T>::new(subdir)
}

impl<T: FileSink + Send> ExampleInput<T> {
    /// Creates an example used for benchmarks.
    /// Created example is left as-is, unless `OKANE_BENCH_CLEANUP` is set.
    /// If `OKANE_BENCH_CLEANUP` is set,
    /// * Always recreate the input.
    /// * Clean up the created input.
    pub fn new(subdir: &Path) -> Result<Self, ExampleInputError> {
        let cleanup = !std::env::var(CLEANUP_KEY)
            .unwrap_or_else(|err| match err {
                std::env::VarError::NotPresent => "".to_string(),
                std::env::VarError::NotUnicode(_) => {
                    log::warn!("CLENAUP_KEY was invalid unicode: {}", err);
                    // invalid unicode = not empty.
                    "1".to_string()
                }
            })
            .is_empty();
        let rootdir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(subdir);
        let rootfile = rootdir.join("root.ledger");
        if !cleanup && T::shortcut(&rootfile) {
            return Ok(Self {
                rootdir,
                rootfile,
                cleanup,
                sink: T::new(),
            });
        }
        let before_input = std::time::Instant::now();
        fs::remove_dir_all(&rootdir).or_else(|e| match e.kind() {
            io::ErrorKind::NotFound => Ok(()),
            _ => Err(e),
        })?;
        fs::create_dir_all(&rootdir)?;
        // Assuming you have 50 accounts & 100 years of records.
        let mut tasks = Vec::new();
        let (tx, rx) = mpsc::channel();
        for i1 in 0..NUM_THREADS {
            let thread_tx = tx.clone();
            let rootdir = rootdir.clone();
            tasks.push(std::thread::spawn(move || {
                let mut years = Vec::with_capacity((YEAR_END - YEAR_BEGIN) as usize);
                for j in YEAR_BEGIN..YEAR_END {
                    let year = Year(j);
                    years.push(year);
                }
                let years = years.as_slice();
                let ret = || -> Result<T, io::Error> {
                    let mut sink = T::new();
                    for i2 in 0..NUM_SUB_FILES / NUM_THREADS {
                        let i = i1 * NUM_SUB_FILES / NUM_THREADS + i2;
                        let dirname = dir_name(i);
                        let subdirpath = rootdir.join(&dirname);
                        fs::create_dir(&subdirpath)?;
                        for year in years {
                            prepare_leaf_file(
                                &mut sink,
                                &subdirpath.join(leaf_file(*year)),
                                i,
                                *year,
                            )?;
                        }
                        prepare_middle_file(&mut sink, &rootdir, &dirname, years)?;
                    }
                    Ok(sink)
                }();
                thread_tx.send(ret).expect("send must not fail");
            }));
        }
        let mut sink = T::new();
        for _ in 0..tasks.len() {
            let another = rx
                .recv_timeout(std::time::Duration::from_secs(150))
                .expect("Can't wait 150 seconds deadline on the recv task")?;
            sink.merge(another);
        }
        prepare_root_file(&mut sink, &rootfile, 0..NUM_SUB_FILES)?;
        for jh in tasks.into_iter() {
            jh.join().expect("thread join must not fail");
        }
        let after_input = std::time::Instant::now();
        let duration = after_input - before_input;
        log::info!("input creation took {:.3} seconds", duration.as_secs_f64());
        Ok(ExampleInput {
            rootdir,
            rootfile,
            cleanup,
            sink,
        })
    }

    pub fn rootpath(&self) -> &Path {
        &self.rootfile
    }

    pub fn new_loader(&self) -> load::Loader<T::FileSystem> {
        load::Loader::new(self.rootfile.clone(), self.sink.clone_as_filesystem())
    }
}

pub trait FileSink: Send + Sized + 'static {
    type FileSystem: load::FileSystem;

    fn new_example(subdir: &Path) -> Result<ExampleInput<Self>, ExampleInputError> {
        ExampleInput::<Self>::new(subdir)
    }

    fn new() -> Self;

    fn clone_as_filesystem(&self) -> Self::FileSystem;

    fn shortcut(rootfile: &Path) -> bool;

    fn merge(&mut self, other: Self);

    /// Gives the writer for the given path.
    fn writer<'a>(
        &'a mut self,
        path: &Path,
    ) -> Result<Box<dyn std::io::Write + 'a>, std::io::Error>;

    fn cleanup(&self, rootdir: &Path);
}

pub struct RealFileSink;

impl FileSink for RealFileSink {
    type FileSystem = load::ProdFileSystem;

    fn new() -> Self {
        RealFileSink
    }

    fn clone_as_filesystem(&self) -> Self::FileSystem {
        load::ProdFileSystem
    }

    fn shortcut(rootfile: &Path) -> bool {
        match fs::metadata(&rootfile) {
            Err(error) => {
                log::warn!(
                    "std::fs::metadata() failed on {}, retry creation: {}",
                    rootfile.to_string_lossy(),
                    error
                );

                false
            }
            Ok(_) => true,
        }
    }

    fn merge(&mut self, _other: Self) {}

    fn writer<'a>(
        &'a mut self,
        path: &Path,
    ) -> Result<Box<dyn std::io::Write + 'a>, std::io::Error> {
        Ok(Box::new(BufWriter::new(File::create(path)?)))
    }

    fn cleanup(&self, rootdir: &Path) {
        let _ignore = std::fs::remove_dir_all(rootdir).inspect_err(|x| {
            log::error!(
                "failed to clean up the directory {}: {}",
                rootdir.display(),
                x
            );
        });
    }
}

pub struct FakeFileSink {
    files: HashMap<PathBuf, Vec<u8>>,
}

impl FileSink for FakeFileSink {
    type FileSystem = load::FakeFileSystem;

    fn new() -> Self {
        Self {
            files: HashMap::default(),
        }
    }

    fn clone_as_filesystem(&self) -> Self::FileSystem {
        self.files.clone().into()
    }

    fn shortcut(_rootfile: &Path) -> bool {
        false
    }

    fn merge(&mut self, other: Self) {
        for (k, v) in other.files {
            self.files.insert(k, v);
        }
    }

    fn writer<'a>(
        &'a mut self,
        path: &Path,
    ) -> Result<Box<dyn std::io::Write + 'a>, std::io::Error> {
        Ok(Box::new(self.files.entry(path.to_owned()).or_default()))
    }

    fn cleanup(&self, _rootdir: &Path) {}
}

fn prepare_root_file<T: FileSink>(
    sink: &mut T,
    rootfile: &Path,
    dirs: std::ops::Range<usize>,
) -> Result<(), std::io::Error> {
    let mut w = sink.writer(rootfile)?;
    for dir in dirs {
        writeln!(w, "include {}.ledger", dir_name(dir))?;
    }
    Ok(())
}

fn prepare_middle_file<T: FileSink>(
    sink: &mut T,
    rootdir: &Path,
    dirname: &str,
    years: &[Year],
) -> Result<(), std::io::Error> {
    let mut target = PathBuf::from(rootdir);
    target.push(format!("{}.ledger", dirname));
    let mut w = sink.writer(&target)?;
    for year in years {
        let leaf = leaf_file(*year);
        writeln!(w, "include {dirname}/{leaf}")?;
    }
    Ok(())
}

fn payee(dir: usize, year: Year, i: usize) -> String {
    format!(
        "Random payee {}",
        ((dir as i32) * 31 + year.0 * 17 + (i as i32) * 7) % 13
    )
}

fn prepare_leaf_file<T: FileSink>(
    sink: &mut T,
    target: &Path,
    dir: usize,
    year: Year,
) -> Result<(), std::io::Error> {
    let mut w = sink.writer(&target)?;
    for i in 0..NUM_TRANSACTIONS_PER_FILE {
        let ordinal = (i * 365 / NUM_TRANSACTIONS_PER_FILE + 1) as u32;
        let date = NaiveDate::from_yo_opt(year.0, ordinal)
            .expect("must be a valid date")
            .format("%Y/%m/%d");
        let payee = payee(dir, year, i);
        let pseudo = ((dir as i32) * 31 + year.0 * 17 + (i as i32) * 7) % 13;
        let (other_account, other_amount, amount) = match pseudo {
            0..=10 => (
                format!("Expenses:Type{}", pseudo),
                "1,234 JPY",
                "-1,234 JPY",
            ),
            11 => ("Income:Salary".to_owned(), "-3,000.00 USD", "400,000 JPY"),
            12 => ("Assets:Account99".to_owned(), "100.00 CHF", "-14,000 JPY"),
            _ => unreachable!("this can't happen"),
        };
        write!(
            w,
            "{date} {payee}\n  Assets:Account{dir:02}  {amount}\n  {other_account}  {other_amount}\n\n",
        )?;
    }
    Ok(())
}

fn dir_name(i: usize) -> String {
    format!("sub-{:02}", i)
}

fn leaf_file(year: Year) -> String {
    format!("{:04}.ledger", year.0)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Year(i32);
