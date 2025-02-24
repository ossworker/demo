use std::path::PathBuf;

use clap::{
    Args, Command, CommandFactory, Parser, Subcommand, ValueHint, arg,
    builder::{Styles, styling::AnsiColor},
};
use clap_complete::{Generator, Shell, generate};

/// demo CLI
#[derive(Parser, Debug)]
#[command(author, version, about,long_about = None, before_help="帮助之前", after_help = "帮助之后")]
#[command(next_line_help = false)]
#[command(propagate_version = true)]
#[command(styles = CLAP_STYLING)]
struct Cli {
    // If provided, outputs the completion file for given shell
    #[arg(long = "generate", value_enum)]
    generator: Option<Shell>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(visible_alias = "hint")]
    ValueHint(ValueHintOpt),
}

#[derive(Args, Debug)]
struct ValueHintOpt {
    // Showcasing all possible ValueHints:
    #[arg(long, value_hint = ValueHint::Unknown)]
    unknown: Option<String>,
    #[arg(long, value_hint = ValueHint::Other)]
    other: Option<String>,
    #[arg(short, long, value_hint = ValueHint::AnyPath)]
    path: Option<PathBuf>,
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    file: Option<PathBuf>,
    #[arg(short, long, value_hint = ValueHint::DirPath)]
    dir: Option<PathBuf>,
    #[arg(short, long, value_hint = ValueHint::ExecutablePath)]
    exe: Option<PathBuf>,
    #[arg(long, value_hint = ValueHint::CommandName)]
    cmd_name: Option<String>,
    #[arg(short, long, value_hint = ValueHint::CommandString)]
    cmd: Option<String>,
    // Command::trailing_var_ar is required to use ValueHint::CommandWithArguments
    #[arg(trailing_var_arg = true, value_hint = ValueHint::CommandWithArguments)]
    command_with_args: Vec<String>,
    #[arg(short, long, value_hint = ValueHint::Username)]
    user: Option<String>,
    #[arg(long, value_hint = ValueHint::Hostname)]
    host: Option<String>,
    #[arg(long, value_hint = ValueHint::Url)]
    url: Option<String>,
    #[arg(long, value_hint = ValueHint::EmailAddress)]
    email: Option<String>,
}

// See also `clap_cargo::style::CLAP_STYLING`
pub const CLAP_STYLING: clap::builder::styling::Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default())
    .usage(AnsiColor::Green.on_default())
    .literal(AnsiColor::Green.on_default())
    .placeholder(AnsiColor::Green.on_default())
    .error(AnsiColor::Red.on_default());

fn print_completions<G: Generator>(r#gen: G, cmd: &mut Command) {
    generate(
        r#gen,
        cmd,
        cmd.get_name().to_string(),
        &mut std::io::stdout(),
    );
}

fn main() {
    let cli = Cli::parse();

    if let Some(generator) = cli.generator {
        let mut cmd = Cli::command();
        eprintln!("Generating completion file for {generator:?}...");
        print_completions(generator, &mut cmd);
    } else {
        println!("{cli:#?}");
    }
}
