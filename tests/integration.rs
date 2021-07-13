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


#[test]
fn test_version() -> Result<()> {
    let mut cmd = Command::cargo_bin("dsc")?;
    let assert = cmd.arg("-d").arg("http://localhost:7880").arg("version").assert();
    assert.success().stderr("");
    Ok(())
}
