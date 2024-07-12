use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    sync::mpsc,
};

use chrono::NaiveDate;

/// Metadata containing the reference to the generated input.
pub struct ExampleInput {
    rootdir: PathBuf,
    rootfile: PathBuf,
    cleanup: bool,
}

impl Drop for ExampleInput {
    fn drop(&mut self) {
        if self.cleanup {
            let _ignore = std::fs::remove_dir_all(&self.rootdir).inspect_err(|x| {
                log::error!(
                    "failed to clean up the directory {}: {}",
                    self.rootdir.to_string_lossy(),
                    x
                );
            });
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

impl ExampleInput {
    /// Creates an example used for benchmarks.
    /// Created example is left as-is, unless `OKANE_BENCH_CLEANUP` is set.
    /// If `OKANE_BENCH_CLEANUP` is set,
    /// * Always recreate the input.
    /// * Clean up the created input.
    pub fn new(subdir: &Path) -> Result<ExampleInput, ExampleInputError> {
        let cleanup = !std::env::var(CLEANUP_KEY)
            .or_else(|x| {
                if let std::env::VarError::NotPresent = &x {
                    Ok("".to_string())
                } else {
                    Err(x)
                }
            })?
            .is_empty();
        let rootdir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(subdir);
        let rootfile = rootdir.join("root.ledger");
        if !cleanup {
            match fs::metadata(&rootfile) {
                Err(error) => log::warn!(
                    "std::fs::metadata() failed on {}, retry creation: {}",
                    rootfile.to_string_lossy(),
                    error
                ),
                Ok(_) => {
                    return Ok(ExampleInput {
                        rootdir,
                        rootfile,
                        cleanup,
                    })
                }
            }
        }
        let before_input = std::time::Instant::now();
        fs::remove_dir_all(&rootdir).or_else(|e| match e.kind() {
            io::ErrorKind::NotFound => Ok(()),
            _ => Err(e),
        })?;
        fs::create_dir(&rootdir)?;
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
                for i2 in 0..NUM_SUB_FILES / NUM_THREADS {
                    let i = i1 * NUM_SUB_FILES / NUM_THREADS + i2;
                    let dirname = dir_name(i);
                    let subdirpath = rootdir.join(&dirname);
                    let ret = || -> Result<(), io::Error> {
                        fs::create_dir(&subdirpath)?;
                        for year in years {
                            prepare_leaf_file(&subdirpath.join(leaf_file(*year)), i, *year)?;
                        }
                        prepare_middle_file(&rootdir, &dirname, years)
                    }();
                    thread_tx.send(ret).expect("send must not fail");
                }
            }));
        }
        for _ in 0..tasks.len() {
            rx.recv_timeout(std::time::Duration::from_secs(150))
                .expect("Can't wait 1 minute on the recv task")?;
        }
        prepare_root_file(&rootfile, 0..NUM_SUB_FILES)?;
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
        })
    }

    pub fn rootpath(&self) -> &Path {
        &self.rootfile
    }
}

fn prepare_root_file(rootfile: &Path, dirs: std::ops::Range<usize>) -> Result<(), std::io::Error> {
    let mut w = BufWriter::new(File::create(rootfile)?);
    for dir in dirs {
        writeln!(w, "include {}.ledger", dir_name(dir))?;
    }
    w.into_inner()?.sync_all()
}

fn prepare_middle_file(
    rootdir: &Path,
    dirname: &str,
    years: &[Year],
) -> Result<(), std::io::Error> {
    let mut target = PathBuf::from(rootdir);
    target.push(format!("{}.ledger", dirname));
    let mut w = BufWriter::new(File::create(&target)?);
    for year in years {
        let leaf = leaf_file(*year);
        writeln!(w, "include {dirname}/{leaf}")?;
    }
    w.into_inner()?.sync_all()
}

fn payee(dir: usize, year: Year, i: usize) -> String {
    format!(
        "Random payee {}",
        ((dir as i32) * 31 + year.0 * 17 + (i as i32) * 7) % 13
    )
}

fn prepare_leaf_file(target: &Path, dir: usize, year: Year) -> Result<(), std::io::Error> {
    let mut w = BufWriter::new(File::create(target)?);
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
    w.into_inner()?.sync_all()
}

fn dir_name(i: usize) -> String {
    format!("sub-{:02}", i)
}

fn leaf_file(year: Year) -> String {
    format!("{:04}.ledger", year.0)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Year(i32);
