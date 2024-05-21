use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    sync::mpsc,
};

use chrono::NaiveDate;
use okane_core::eval::load;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

struct ExampleInput {
    rootdir: PathBuf,
    rootfile: PathBuf,
}

fn prepare_root_file(rootfile: &Path, dirs: std::ops::Range<i32>) -> Result<(), std::io::Error> {
    let mut w = BufWriter::new(File::create(rootfile)?);
    for dir in dirs {
        write!(w, "include {dir}.ledger")?;
    }
    w.into_inner()?.sync_all()
}

fn prepare_middle_file(
    rootdir: &Path,
    dirname: &str,
    leaves: std::ops::Range<i32>,
) -> Result<(), std::io::Error> {
    let mut target = PathBuf::from(rootdir);
    target.push(format!("{}.ledger", dirname));
    let mut w = BufWriter::new(File::create(&target)?);
    for l in leaves {
        let leaf = leaf_file(l);
        write!(w, "include {dirname}/{leaf}")?;
    }
    w.into_inner()?.sync_all()
}

fn payee(dir: i32, year: i32, i: u32) -> String {
    format!(
        "Random payee {}",
        (dir * 31 + year * 17 + (i as i32) * 7) % 13
    )
}

fn prepare_leaf_file(target: &Path, dir: i32, year: i32) -> Result<(), std::io::Error> {
    let mut w = BufWriter::new(File::create(target)?);
    for i in 0..1000 {
        let ordinal = i * 365 / 1000 + 1;
        let date = NaiveDate::from_yo_opt(year, ordinal)
            .expect("must be a valid date")
            .format("%y/%m/%d");
        let payee = payee(dir, year, i);
        let pseudo = (dir * 31 + year * 17 + (i as i32) * 7) % 13;
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

fn dir_name(i: i32) -> String {
    format!("sub{:02}", i)
}

fn leaf_file(year: i32) -> String {
    format!("{:04}.ledger", year)
}

fn create_example() -> Result<ExampleInput, std::io::Error> {
    let rootdir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("eval_bench");
    fs::remove_dir_all(&rootdir).or_else(|e| match e.kind() {
        io::ErrorKind::NotFound => Ok(()),
        _ => Err(e),
    })?;
    fs::create_dir(&rootdir)?;
    let rootfile = rootdir.join("root.ledger");
    // Assuming you have 50 accounts & 100 years of records.
    let year_base = 2000;
    let mut tasks = Vec::new();
    let (tx, rx) = mpsc::channel();
    for i1 in 0..5 {
        let thread_tx = tx.clone();
        let rootdir = rootdir.clone();
        tasks.push(std::thread::spawn(move || {
            for i2 in 0..10 {
                let i = i1 * 10 + i2;
                let dirname = dir_name(i);
                let subdirpath = rootdir.join(&dirname);
                let ret = || -> Result<(), io::Error> {
                    fs::create_dir(&subdirpath)?;
                    for j in 0..100 {
                        let year: i32 = j + year_base;
                        prepare_leaf_file(&subdirpath.join(leaf_file(j)), i, year)?;
                    }
                    prepare_middle_file(&rootdir, &dirname, 0..1000)
                }();
                thread_tx.send(ret).expect("send must not fail");
            }
        }));
    }
    for _ in 0..tasks.len() {
        rx.recv_timeout(std::time::Duration::from_secs(150))
            .expect("Can't wait 1 minute on the recv task")?;
    }
    prepare_root_file(&rootfile, 0..50)?;
    for jh in tasks.into_iter() {
        jh.join().expect("thread join must not fail");
    }
    Ok(ExampleInput { rootdir, rootfile })
}

impl Drop for ExampleInput {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.rootdir).unwrap()
    }
}

fn load_benchmark(c: &mut Criterion) {
    let before_input = std::time::Instant::now();
    let input = create_example().unwrap();
    let after_input = std::time::Instant::now();
    let duration = after_input - before_input;
    log::info!("input creation took {:.3} seconds", duration.as_secs_f64());
    c.bench_function("load simple", |b| {
        b.iter(|| black_box(load(&input.rootfile)))
    });
}

#[ctor::ctor]
fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

criterion_group!(benches, load_benchmark);
criterion_main!(benches);
