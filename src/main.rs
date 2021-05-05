use std::error::Error;

use aaacs::accounts::ParsedAccounts;
use aaacs::gui_iced;
use structopt::StructOpt;

/// Automated Accurate Accounting Collaborative System
///
/// A simple application to handle accounts between friends
#[derive(StructOpt, Debug)]
#[structopt(name = "aaacs")]
struct Args {
    /// If present on the command line, only compute the reports of the
    /// passed files.
    #[structopt(long)]
    cli: bool,

    /// Number of decimal points to print
    #[structopt(long, default_value = "2")]
    precision: u8,

    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<std::path::PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::from_args();
    if args.cli {
        for accounts_path in args.files {
            println!(
                "Processing accounts for {}:",
                accounts_path.to_string_lossy()
            );

            let accounts_file = std::fs::File::open(&accounts_path)?;
            let accounts = ParsedAccounts::from_yaml_reader(accounts_file)?;
            accounts.print_balances(args.precision);
        }
    } else {
        gui_iced::run();
    }

    Ok(())
}

// This should be gracefully handled by Iced in the future. Probably using our
// own proc macro, or maybe the whole process is streamlined by `wasm-pack` at
// some point.
#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub fn run() {
        super::main().unwrap()
    }
}
