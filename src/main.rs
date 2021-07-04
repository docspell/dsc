use dsc::DscError;
use std::process;

fn main() {
    env_logger::init();
    let result = dsc::execute();
    if let Err(err) = result {
        eprintln!("Error: {:#?}", err);
        process::exit(exit_code(&err));
    }
}

fn exit_code(err: &DscError) -> i32 {
    match err {
        DscError::Config(_) => 1,
        DscError::Cmd(_) => 2,
    }
}
