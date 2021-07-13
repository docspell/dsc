mod common;
use crate::common::{mk_cmd, Result};
use assert_cmd::prelude::*;

#[test]
fn remote_login() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let assert = cmd.arg("login").args(&["--password", "test"]).assert();
    assert.success().stderr("");
    Ok(())
}
