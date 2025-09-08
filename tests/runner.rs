use std::path::Path;

use assert_cmd::Command;

const VALID_PATH: &str = "./tests/programs/valid";
const INVALID_PATH: &str = "./tests/programs/invalid";

const FILE_PATTERN: &str = r"^.*\.az$";

fn valids(path: &Path) -> datatest_stable::Result<()>
{
    Command::cargo_bin("lang")?.arg(path).assert().success();

    Ok(())
}

fn invalids(path: &Path) -> datatest_stable::Result<()>
{
    Command::cargo_bin("lang")?.arg(path).assert().failure();

    Ok(())
}

datatest_stable::harness! {
    {test = valids, root = VALID_PATH, pattern = FILE_PATTERN},
    {test = invalids, root = INVALID_PATH, pattern = FILE_PATTERN}
}
