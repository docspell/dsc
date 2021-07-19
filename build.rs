use std::path::PathBuf;
use vergen::{vergen, Config};

fn main() {
    // Generate the default 'cargo:' instruction output
    let mut vergen_cfg = Config::default();
    let dot_git = PathBuf::from(".git");
    *vergen_cfg.git_mut().enabled_mut() = dot_git.exists();
    vergen(vergen_cfg).unwrap();

    if !dot_git.exists() {
        println!("cargo:rustc-env=VERGEN_GIT_SHA=unknown");
    }
}
