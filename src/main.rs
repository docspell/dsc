use dsc::error::Error;
use std::process;

fn main() {
    let error_style = console::Style::new().red().bright();

    env_logger::init();
    let result = dsc::execute();
    if let Err(err) = result {
        eprintln!("{}", error_style.apply_to(&err));
        process::exit(exit_code(&err));
    }
}

fn exit_code(err: &Error) -> i32 {
    match err {
        Error::Config { source: _ } => 1,
        Error::Cmd { source: _ } => 2,
    }
}
