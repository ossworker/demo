use std::path::PathBuf;

use clap::{
    builder::{styling::AnsiColor, Styles},
    Parser, Subcommand,
};

/// demo CLI
#[derive(Parser, Debug)]
#[command(author, version, about,long_about = None, before_help="帮助之前", after_help = "帮助之后")]
#[command(next_line_help = false)]
#[command(propagate_version = true)]
#[command(styles = CLAP_STYLING)]
struct Cli {
    /// Optional name 操作
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// turn debugging on
    #[arg(short, long , action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// does testing
    #[command(arg_required_else_help = true)]
    Test(TestCommandOpts),
}

#[derive(Parser, Debug)]
struct TestCommandOpts {
    /// list test value
    #[arg(short, long)]
    list: bool,
}

// See also `clap_cargo::style::CLAP_STYLING`
pub const CLAP_STYLING: clap::builder::styling::Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default())
    .usage(AnsiColor::Green.on_default())
    .literal(AnsiColor::Green.on_default())
    .placeholder(AnsiColor::Green.on_default())
    .error(AnsiColor::Red.on_default());

fn main() {
    let cli = Cli::parse();
    // println!("{:#?}", &cli);
    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }
    match cli.debug {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }
    match &cli.command {
        Some(Commands::Test(TestCommandOpts { list })) => {
            if *list {
                println!("Printing testing lists...");
            } else {
                println!("Not printing testing lists...");
            }
        }
        None => {}
    }
}
