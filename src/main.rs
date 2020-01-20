use std::error::Error;

use aaacs::accounts::SerializedAccounts;
use aaacs::gui_iced;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    let _prog_name = args.next();
    for accounts_path in args {
        println!("Processing accounts for {}:", accounts_path);

        let accounts_file = std::fs::File::open(&accounts_path)?;
        let accounts: SerializedAccounts =
            serde_yaml::from_reader(accounts_file)?;
        let accounts = accounts.parse()?;
        accounts.print_balances(2);
    }

    gui_iced::run();

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
