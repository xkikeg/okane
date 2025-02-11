use std::path::PathBuf;

use bumpalo::Bump;
use maplit::hashmap;
use rstest::rstest;

use okane_core::{load, report};

pub mod testing;

#[ctor::ctor]
fn init() {
    env_logger::builder()
        .filter_level(log::LevelFilter::max())
        .init();
}

fn new_loader(input: &str) -> Result<load::Loader<load::FakeFileSystem>, std::io::Error> {
    let src = testing::read_as_utf8(input)?;
    let fs = hashmap! {
        PathBuf::from("/path/to/").join(input) => src.into_bytes(),
    };
    let fs: load::FakeFileSystem = fs.into();
    Ok(
        load::Loader::new(PathBuf::from("/path/to/").join(input), fs)
            .with_error_renderer(annotate_snippets::Renderer::plain()),
    )
}

#[rstest]
#[case("error/same_commodity_cost.ledger")]
#[case("error/undeducible.ledger")]
#[case("error/zero_cost.ledger")]
#[case("error/zero_lot.ledger")]
#[case("error/zero_posting_with_lot.ledger")]
fn report_error_string(#[case] input: &str) {
    let mut golden = input.to_string();
    golden.push_str(".error.txt");
    let arena = Bump::new();
    let mut ctx = report::ReportContext::new(&arena);
    let golden = testing::Golden::new(&golden).unwrap();

    let got_err = report::process(
        &mut ctx,
        new_loader(input).unwrap(),
        &report::ProcessOptions::default(),
    )
    .unwrap_err();

    golden.assert(&format!("{}\n", got_err));
}
