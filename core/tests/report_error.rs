use std::{
    ffi::OsStr,
    path::{Component, Path, PathBuf},
};

use bumpalo::Bump;
use maplit::hashmap;
use rstest::rstest;

use okane_core::{
    load::{self, FileSystem},
    report,
};

#[ctor::ctor]
fn init() {
    env_logger::builder()
        .filter_level(log::LevelFilter::max())
        .init();
}

fn as_test_filepath(input: &Path) -> Result<PathBuf, std::io::Error> {
    // we need to fake file path to avoid having the real full path for golden tests.
    // also we cannot use /rooted/path because of Unix / Windows difference.
    let mut result = PathBuf::from("okane");
    for part in input
        .components()
        .skip_while(|c| !matches!(c, Component::Normal(x) if *x == OsStr::new("testdata")))
    {
        result.push(part);
    }
    Ok(load::FakeFileSystem::canonicalize_path(&result).into_owned())
}

fn new_loader(input: PathBuf) -> Result<load::Loader<load::FakeFileSystem>, std::io::Error> {
    let src = std::fs::read_to_string(&input)?;
    let filepath = as_test_filepath(&input)?;
    let fs = hashmap! {
        filepath.clone() => src.into_bytes(),
    };
    let fs: load::FakeFileSystem = fs.into();
    Ok(load::Loader::new(filepath, fs).with_error_renderer(annotate_snippets::Renderer::plain()))
}

#[rstest]
fn report_error_string(
    #[base_dir = "../"]
    #[files("testdata/error/*.ledger")]
    input: PathBuf,
) {
    let mut golden = input.clone();
    assert!(
        golden.set_extension("ledger.error.txt"),
        "failed to add extension to golden file {}",
        golden.display()
    );
    let arena = Bump::new();
    let mut ctx = report::ReportContext::new(&arena);
    let golden = okane_golden::Golden::new(golden).unwrap();

    let got_err = report::process(
        &mut ctx,
        new_loader(input).unwrap(),
        &report::ProcessOptions::default(),
    )
    .unwrap_err();

    golden.assert(&format!("{}\n", got_err));
}
