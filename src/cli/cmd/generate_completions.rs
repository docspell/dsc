use clap::{App, ArgEnum, Parser};
//use clap_complete::generators::{Bash, Elvish, Fish, PowerShell, Zsh};
use clap_complete::{generate, Generator, Shell};

/// Generates completions for some shells.
///
#[derive(Parser, std::fmt::Debug)]
pub struct Input {
    #[clap(long, arg_enum)]
    pub shell: GeneratorChoice,
}

#[derive(ArgEnum, Clone, Debug, PartialEq)]
pub enum GeneratorChoice {
    Bash,
    Elvish,
    Fish,
    #[clap(name = "powershell")]
    PowerShell,
    Zsh,
}

impl Input {
    pub fn print_completions(&self, app: &mut App) {
        match &self.shell {
            GeneratorChoice::Bash => generate_completions(Shell::Bash, app),
            GeneratorChoice::Elvish => generate_completions(Shell::Elvish, app),
            GeneratorChoice::Fish => generate_completions(Shell::Fish, app),
            GeneratorChoice::PowerShell => generate_completions(Shell::PowerShell, app),
            GeneratorChoice::Zsh => generate_completions(Shell::Zsh, app),
        }
    }
}

fn generate_completions<G: Generator>(gen: G, app: &mut App) {
    generate(gen, app, "dsc", &mut std::io::stdout());
}
