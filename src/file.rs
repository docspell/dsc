use sha2::{Digest, Sha256};
use std::io;
use std::path::PathBuf;

const BUFFER_SIZE: usize = 1024;

pub fn digest_file_sha256(file: &PathBuf) -> Result<String, io::Error> {
    digest_file::<Sha256>(file)
}

pub fn digest_file<D: Digest + Default>(file: &PathBuf) -> Result<String, io::Error> {
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

/// Puts `suffix` in the filename before the extension.
pub fn splice_name(fname: &str, suffix: &i32) -> String {
    let p = PathBuf::from(fname);

    match p.extension() {
        Some(ext) => {
            let mut base = fname.trim_end_matches(ext.to_str().unwrap()).chars();
            base.next_back();
            format!("{}_{}.{}", base.as_str(), suffix, ext.to_str().unwrap())
        }
        None => format!("{}_{}", fname, suffix),
    }
}

/// Extracts the filename from a Content-Disposition header
pub fn filename_from_header<'a>(header_value: &'a str) -> Option<&'a str> {
    header_value
        .find("filename=")
        .map(|index| &header_value[9 + index..])
        .map(|rest| rest.trim_matches('"'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_filename_from_header() {
        assert_eq!(
            filename_from_header("inline; filename=\"test.jpg\""),
            Some("test.jpg")
        );
    }

    #[test]
    fn unit_splice_name() {
        assert_eq!(splice_name("abc.pdf", &1), "abc_1.pdf");
        assert_eq!(splice_name("abc", &1), "abc_1");
        assert_eq!(splice_name("stuff.tar.gz", &2), "stuff.tar_2.gz");
    }
}
