use std::error::Error;

use aaacs::accounts::SerializedAccounts;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    let _prog_name = args.next();
    for accounts_path in args {
        println!("Processing accounts for {}:", accounts_path);

        let accounts_file = std::fs::File::open(&accounts_path)?;
        let accounts: SerializedAccounts = serde_yaml::from_reader(accounts_file)?;
        let accounts = accounts.parse()?;
        accounts.print_balances(2);
    }

    Ok(())
}
