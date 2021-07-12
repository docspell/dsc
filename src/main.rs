use dsc::error::{Error, Result};
use std::env;
use std::process;

const LOG_LEVEL: &'static str = "RUST_LOG";

fn main() {
    let error_style = console::Style::new().red().bright();
    let result = execute();
    if let Err(err) = result {
        eprintln!("{}", error_style.apply_to(&err));
        process::exit(exit_code(&err));
    }
}

fn execute() -> Result<()> {
    let opts = dsc::read_args();
    let remove_env = match opts.common_opts.verbose {
        1 => set_log_level("info"),
        n => {
            if n > 1 {
                set_log_level("debug")
            } else {
                false
            }
        }
    };
    env_logger::init();

    let cfg = dsc::read_config(&opts.config)?;
    eprintln!("Docspell at: {:}", cfg.docspell_url);
    let result = dsc::execute_cmd(cfg, opts);
    if remove_env {
        env::remove_var(LOG_LEVEL);
    }
    result
}

fn set_log_level(level: &str) -> bool {
    let current = env::var_os(LOG_LEVEL);
    if current.is_none() {
        env::set_var(LOG_LEVEL, level);
        true
    } else {
        false
    }
}

fn exit_code(err: &Error) -> i32 {
    match err {
        Error::Config { source: _ } => 1,
        Error::Cmd { source: _ } => 2,
    }
}
