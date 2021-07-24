use clap::{App, ArgEnum, Clap};
use clap_generate::generators::{Bash, Elvish, Fish, PowerShell, Zsh};
use clap_generate::{generate, Generator};

/// Generates completions for some shells.
///
#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    #[clap(long, arg_enum)]
    pub shell: GeneratorChoice,
}

#[derive(ArgEnum, Debug, PartialEq)]
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
            GeneratorChoice::Bash => generate_completions::<Bash>(app),
            GeneratorChoice::Elvish => generate_completions::<Elvish>(app),
            GeneratorChoice::Fish => generate_completions::<Fish>(app),
            GeneratorChoice::PowerShell => generate_completions::<PowerShell>(app),
            GeneratorChoice::Zsh => generate_completions::<Zsh>(app),
        }
    }
}

fn generate_completions<G: Generator>(app: &mut App) {
    generate::<G, _>(app, "dsc", &mut std::io::stdout());
}
