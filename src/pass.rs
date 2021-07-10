use std::io;
use std::process::Command;

pub fn pass_password(entry: &str) -> Result<String, io::Error> {
    log::debug!("Running external command `pass show {}`", entry);
    let output = Command::new("pass").arg("show").arg(entry).output()?;
    if !output.status.success() {
        let msg = String::from_utf8(output.stderr);
        log::warn!("pass exited with error {:}: {:?}", output.status, msg);
        Err(io_err(&format!(
            "Pass failed with an error ({:}): {}",
            output.status,
            msg.unwrap_or("no output".into())
        )))
    } else {
        String::from_utf8(output.stdout)
            .map_err(|_| io_err("Error decoding bytes using utf8!"))?
            .lines()
            .next()
            .map(String::from)
            .ok_or(io_err(&format!("No password found for entry: {}", entry)))
    }
}

fn io_err(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, msg)
}
