use std::{
    collections::HashMap,
    fmt::Display,
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
    params: InputParams,
}

impl<T: FileSink> Drop for ExampleInput<T> {
    fn drop(&mut self) {
        if self.cleanup {
            self.sink.cleanup(&self.rootdir);
        } else {
            log::debug!("{} is not set, leave the test data as-is", CLEANUP_KEY);
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
const NUM_THREADS: usize = 2;

#[derive(Debug, Clone, Copy)]
pub struct InputParams {
    pub num_years: i32,
    pub num_sub_files: usize,
    pub num_transactions_per_file: usize,
    pub sample_size: Option<usize>,
}

impl Display for InputParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "num_years={},num_sub_files={},num_transactions_per_file={}",
            self.num_years, self.num_sub_files, self.num_transactions_per_file
        )
    }
}

impl InputParams {
    pub fn all() -> impl Iterator<Item = Self> {
        [Self::small(), Self::middle(), Self::large()].into_iter()
    }

    pub fn small() -> Self {
        Self {
            num_years: 5,
            num_sub_files: 10,
            num_transactions_per_file: 200,
            sample_size: None,
        }
    }

    pub fn middle() -> Self {
        Self {
            num_years: 10,
            num_sub_files: 16,
            num_transactions_per_file: 500,
            sample_size: None,
        }
    }

    pub fn large() -> Self {
        Self {
            num_years: 30,
            num_sub_files: 30,
            num_transactions_per_file: 800,
            sample_size: Some(20),
        }
    }

    pub fn num_transactions(&self) -> u64 {
        (self.num_years as u64)
            * (self.num_sub_files as u64)
            * (self.num_transactions_per_file as u64)
    }

    fn years(&self) -> impl Iterator<Item = Year> {
        (YEAR_BEGIN..YEAR_BEGIN + self.num_years).map(Year)
    }
}

impl<T: FileSink + Send> ExampleInput<T> {
    /// Creates an example used for benchmarks.
    /// Created example is left as-is, unless `OKANE_BENCH_CLEANUP` is set.
    /// If `OKANE_BENCH_CLEANUP` is set,
    /// * Always recreate the input.
    /// * Clean up the created input.
    pub fn new(subdir: &Path, params: InputParams) -> Result<Self, ExampleInputError> {
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
                params,
            });
        }
        let before_input = std::time::Instant::now();
        T::initialize(&rootdir)?;
        let mut tasks = Vec::new();
        let (tx, rx) = mpsc::channel();
        for i1 in 0..NUM_THREADS {
            let thread_tx = tx.clone();
            let rootdir = rootdir.clone();
            tasks.push(std::thread::spawn(move || {
                let ret = || -> Result<T, io::Error> {
                    let mut sink = T::new();
                    for i2 in 0..params.num_sub_files / NUM_THREADS {
                        let i = i1 * params.num_sub_files / NUM_THREADS + i2;
                        let dirname = dir_name(i);
                        let subdirpath = rootdir.join(&dirname);
                        T::create_subdir(&subdirpath)?;
                        for year in params.years() {
                            prepare_leaf_file(
                                &mut sink,
                                &subdirpath.join(leaf_file(year)),
                                &params,
                                i,
                                year,
                            )?;
                        }
                        prepare_middle_file(&mut sink, &rootdir, &dirname, &params)?;
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
        prepare_root_file(&mut sink, &rootfile, 0..params.num_sub_files)?;
        for jh in tasks.into_iter() {
            jh.join().expect("thread join must not fail");
        }
        let after_input = std::time::Instant::now();
        let duration = after_input - before_input;
        log::debug!("input creation took {:.3} seconds", duration.as_secs_f64());
        Ok(ExampleInput {
            rootdir,
            rootfile,
            cleanup,
            sink,
            params,
        })
    }

    pub fn rootpath(&self) -> &Path {
        &self.rootfile
    }

    pub fn new_loader(&self) -> load::Loader<T::FileSystem> {
        load::Loader::new(self.rootfile.clone(), self.sink.clone_as_filesystem())
    }

    pub fn num_transactions(&self) -> u64 {
        self.params.num_transactions()
    }
}

pub trait FileSink: Send + Sized + 'static {
    type FileSystem: load::FileSystem;

    fn new_example(
        subdir: &Path,
        params: InputParams,
    ) -> Result<ExampleInput<Self>, ExampleInputError> {
        ExampleInput::<Self>::new(subdir, params)
    }

    fn new() -> Self;

    fn clone_as_filesystem(&self) -> Self::FileSystem;

    fn initialize(rootdir: &Path) -> Result<(), std::io::Error>;

    fn create_subdir(subdir: &Path) -> Result<(), std::io::Error>;

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

    fn initialize(rootdir: &Path) -> Result<(), std::io::Error> {
        fs::remove_dir_all(&rootdir).or_else(|e| match e.kind() {
            io::ErrorKind::NotFound => Ok(()),
            _ => Err(e),
        })?;
        fs::create_dir_all(&rootdir)
    }

    fn create_subdir(subdir: &Path) -> Result<(), std::io::Error> {
        fs::create_dir(subdir)
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

    fn initialize(_rootdir: &Path) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn create_subdir(_subdir: &Path) -> Result<(), std::io::Error> {
        Ok(())
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
    params: &InputParams,
) -> Result<(), std::io::Error> {
    let mut target = PathBuf::from(rootdir);
    target.push(format!("{}.ledger", dirname));
    let mut w = sink.writer(&target)?;
    for year in params.years() {
        let leaf = leaf_file(year);
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
    params: &InputParams,
    dir: usize,
    year: Year,
) -> Result<(), std::io::Error> {
    let mut w = sink.writer(&target)?;
    for i in 0..params.num_transactions_per_file {
        let ordinal = (i * 365 / params.num_transactions_per_file + 1) as u32;
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
