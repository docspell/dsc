mod common;
use crate::common::{mk_cmd, Result};
use assert_cmd::prelude::*;
use dsc::types::BasicResult;
use std::process::Command;

#[test]
fn dsc_help() -> Result<()> {
    let mut cmd = Command::cargo_bin("dsc")?;
    let assert = cmd.arg("--help").assert();
    assert.success().stderr("");
    Ok(())
}

#[test]
fn remote_version() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let assert = cmd.arg("version").assert();
    assert.success().stderr("");
    Ok(())
}

#[test]
fn remote_register() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let out = cmd
        .arg("register")
        .args(&["-c", "demo", "-l", "demo", "--password", "test"])
        .output()?;

    let out: BasicResult = serde_json::from_slice(out.stdout.as_slice())?;
    assert!(out.success);
    Ok(())
}
