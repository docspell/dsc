use sha2::{Digest, Sha256};
use std::io;
use std::path::PathBuf;

const BUFFER_SIZE: usize = 1024;

pub fn digest_file_sha256(file: &PathBuf) -> Result<String, io::Error> {
    digest_file::<Sha256>(file)
}

pub fn digest_file<D: Digest + Default>(file: &PathBuf) -> Result<String, io::Error> {
    log::debug!("Calculating hash for file {}", file.display());
    std::fs::File::open(file).and_then(|mut f| digest::<D, _>(&mut f))
}

/// Compute digest value for given `Reader` and return it as hex string
pub fn digest<D: Digest + Default, R: io::Read>(reader: &mut R) -> Result<String, io::Error> {
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
