use assert_cmd::cargo::CargoError;
use assert_cmd::prelude::*;
use std::{io, process::Command};

#[derive(Debug)]
pub enum Error {
    Cargo(CargoError),
    IO(io::Error),
    Json(serde_json::Error),
}
impl std::convert::From<CargoError> for Error {
    fn from(e: CargoError) -> Self {
        Error::Cargo(e)
    }
}
impl std::convert::From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO(e)
    }
}
impl std::convert::From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

pub type Result<A> = std::result::Result<A, Error>;

pub fn mk_cmd() -> Result<Command> {
    let mut cmd = Command::cargo_bin("dsc")?;
    cmd.arg("-c").arg("./ci/dsc-config.toml");
    Ok(cmd)
}
