mod common;
use crate::common::{mk_cmd, Result};
use assert_cmd::prelude::*;

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
