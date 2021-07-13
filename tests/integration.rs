use assert_cmd::{cargo::CargoError, prelude::*};
use std::process::Command;

type Result<A> = std::result::Result<A, CargoError>;

#[test]
fn test_help() -> Result<()> {
    let mut cmd = Command::cargo_bin("dsc")?;
    let assert = cmd.arg("--help").assert();
    assert.success().stderr("");
    Ok(())
}
