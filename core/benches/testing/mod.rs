use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::Display,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    sync::mpsc,
};

use chrono::NaiveDate;
use okane_core::{load, syntax::pretty_decimal::PrettyDecimal};
use rust_decimal_macros::dec;

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
const BENCH_TARGET_KEY: &str = "OKANE_BENCH_TARGET";

const YEAR_BEGIN: i32 = 2015;
const NUM_THREADS: usize = 2;

#[derive(Debug, Clone, Copy)]
pub struct InputParams {
    pub name: &'static str,
    pub num_years: i32,
    pub num_sub_files: usize,
    pub num_transactions_per_file: usize,
    pub sample_size: Option<usize>,
    pub more_commodity: bool,
}

impl Display for InputParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{0}", self.name)
    }
}

impl InputParams {
    pub fn params_from_env() -> impl Iterator<Item = Self> {
        let mut target = std::env::var(BENCH_TARGET_KEY).unwrap_or_default();
        if target == "" {
            target = Self::middle_more_commodity().name.to_string();
        }
        [
            Self::small(),
            Self::middle(),
            Self::middle_more_commodity(),
            Self::large(),
        ]
        .into_iter()
        .scan(false, move |done, item| {
            if *done {
                return None;
            }
            if item.name == &target {
                *done = true;
            }
            Some(item)
        })
    }

    pub fn small() -> Self {
        Self {
            name: "small_5y10a200t",
            num_years: 5,
            num_sub_files: 10,
            num_transactions_per_file: 200,
            sample_size: None,
            more_commodity: false,
        }
    }

    pub fn middle() -> Self {
        Self {
            name: "middle_10y16a500t",
            num_years: 10,
            num_sub_files: 16,
            num_transactions_per_file: 500,
            sample_size: None,
            more_commodity: false,
        }
    }

    pub fn middle_more_commodity() -> Self {
        Self {
            name: "middle_more_commodity_10y16a500t",
            num_years: 10,
            num_sub_files: 16,
            num_transactions_per_file: 500,
            sample_size: None,
            more_commodity: true,
        }
    }

    pub fn large() -> Self {
        Self {
            name: "large_20y30a800t",
            num_years: 20,
            num_sub_files: 30,
            num_transactions_per_file: 800,
            sample_size: Some(20),
            more_commodity: false,
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
                        let mut total: i64 = 0;
                        for year in params.years() {
                            total = prepare_leaf_file(
                                &mut sink,
                                &subdirpath.join(leaf_file(year)),
                                &params,
                                i,
                                year,
                                total,
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

struct Amount<'a>(PrettyDecimal, Cow<'a, str>);

impl<'a> Display for Amount<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{0} {1}", self.0, self.1)
    }
}

/// Creates the leaf file, returns the total amount of the asset.
fn prepare_leaf_file<T: FileSink>(
    sink: &mut T,
    target: &Path,
    params: &InputParams,
    dir: usize,
    year: Year,
    mut total: i64,
) -> Result<i64, std::io::Error> {
    let mut w = sink.writer(&target)?;
    for i in 0..params.num_transactions_per_file {
        let ordinal = (i * 365 / params.num_transactions_per_file + 1) as u32;
        let date = NaiveDate::from_yo_opt(year.0, ordinal)
            .expect("must be a valid date")
            .format("%Y/%m/%d");
        let payee = payee(dir, year, i);
        let pseudo = ((dir as i32) * 31 + year.0 * 17 + (i as i32) * 7) % 13;
        let (other_account, other_amount, amount): (Cow<str>, _, _) = match pseudo {
            0..=10 => {
                if pseudo >= 9 && params.more_commodity {
                    let pseudo2 = ((dir as i32) * 31 + year.0 * 17 + (i as i32) * 7) % 26;
                    let pseudo3 = ((dir as i32) * 31 + year.0 * 17 + (i as i32) * 7) / 26 % 26;
                    let commodity = format!(
                        "COMMODITY_{}{}",
                        nth_alphabet(pseudo2),
                        nth_alphabet(pseudo3)
                    );
                    total -= 3000;
                    (
                        format!("Expenses:Type{}", pseudo).into(),
                        Amount(PrettyDecimal::comma3dot(dec!(100)), commodity.into()),
                        Amount(PrettyDecimal::comma3dot(dec!(-3000)), "JPY".into()),
                    )
                } else {
                    total -= 1234;
                    (
                        format!("Expenses:Type{}", pseudo).into(),
                        Amount(PrettyDecimal::comma3dot(dec!(1234)), "JPY".into()),
                        Amount(PrettyDecimal::comma3dot(dec!(-1234)), "JPY".into()),
                    )
                }
            }
            11 => {
                total += 400000;
                (
                    "Income:Salary".into(),
                    Amount(
                        PrettyDecimal::comma3dot(dec!(-3000.00)),
                        Cow::Borrowed("USD"),
                    ),
                    Amount(PrettyDecimal::comma3dot(dec!(400000)), "JPY".into()),
                )
            }
            12 => {
                total -= 14000;
                (
                    "Assets:Account99".into(),
                    Amount(PrettyDecimal::comma3dot(dec!(100.00)), "CHF".into()),
                    Amount(PrettyDecimal::comma3dot(dec!(-14000)), "JPY".into()),
                )
            }
            _ => unreachable!("this can't happen"),
        };
        writeln!(w, "{date} {payee}",)?;
        writeln!(w, "  Assets:Account{dir:02}     {amount} = {total} JPY",)?;
        writeln!(w, "  {other_account}     {other_amount}",)?;
        writeln!(w, "")?;
    }
    Ok(total)
}

fn nth_alphabet(i: i32) -> char {
    char::from_u32('a' as u32 + i as u32 % 26).unwrap()
}

fn dir_name(i: usize) -> String {
    format!("sub-{:02}", i)
}

fn leaf_file(year: Year) -> String {
    format!("{:04}.ledger", year.0)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Year(i32);
