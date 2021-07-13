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
        .args(&["-c", "demo2", "-l", "demo2", "--password", "test"])
        .output()?;

    let out: BasicResult = serde_json::from_slice(out.stdout.as_slice())?;
    assert!(out.success);
    Ok(())
}

#[test]
fn remote_upload_web() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let assert = cmd.arg("upload").arg("README.md").assert();
    assert.success().stderr("");
    Ok(())
}

#[test]
fn remote_upload_int_endpoint() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let assert = cmd
        .arg("upload")
        .args(&[
            "-c",
            "demo",
            "-i",
            "--header",
            "Docspell-Integration:test123",
        ])
        .arg("README.md")
        .assert();
    assert.success().stderr("");
    Ok(())
}
