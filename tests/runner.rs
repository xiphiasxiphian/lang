use std::path::Path;

use assert_cmd::{Command, assert::Assert};

const VALID_PATH: &str = "./tests/programs/valid";
const INVALID_PATH: &str = "./tests/programs/invalid";

const FILE_PATTERN: &str = r"^.*\.az$";

fn run_path(path: &Path) -> Result<Assert, assert_cmd::cargo::CargoError>
{
    Ok(Command::cargo_bin("lang")?
        .arg("--fail")
        .arg("100") // Used to filter out compiler panics or config errors
        .arg(path)
        .assert())
}

fn valids(path: &Path) -> datatest_stable::Result<()>
{
    run_path(path)?.success();

    Ok(())
}

fn invalids(path: &Path) -> datatest_stable::Result<()>
{
    run_path(path)?.code(100).failure();

    Ok(())
}

datatest_stable::harness! {
    {test = valids, root = VALID_PATH, pattern = FILE_PATTERN},
    {test = invalids, root = INVALID_PATH, pattern = FILE_PATTERN}
}
