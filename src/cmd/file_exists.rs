use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::opts::ConfigOpts;
use clap::Clap;
use sha2::{Digest, Sha256};
use std::io;
use std::path::PathBuf;

/// Checks if the given files exist in docspell.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    /// Specify an optional source id. If not given, login is required.
    #[clap(long, short)]
    pub source: Option<String>,

    /// One or more files to check
    pub file: PathBuf,
    pub files: Vec<PathBuf>,
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        println!(
            "{:}: {:}",
            self.file.display(),
            digest_file::<Sha256>(&self.file).map_err(CmdError::IOError)?
        );
        println!("todo");
        Ok(())
    }
}

const BUFFER_SIZE: usize = 1024;

fn digest_file<D: Digest + Default>(file: &PathBuf) -> Result<String, io::Error> {
    std::fs::File::open(file).and_then(|mut f| digest::<D, _>(&mut f))
}

/// Compute digest value for given `Reader` and return it as hex string
fn digest<D: Digest + Default, R: io::Read>(reader: &mut R) -> Result<String, io::Error> {
    let mut sh = D::default();
    let mut buffer = [0u8; BUFFER_SIZE];
    loop {
        let n = match reader.read(&mut buffer) {
            Ok(n) => n,
            Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "Could not read file")),
        };
        sh.update(&buffer[..n]);
        if n == 0 || n < BUFFER_SIZE {
            break;
        }
    }
    Ok(hex::encode(&sh.finalize()))
}
