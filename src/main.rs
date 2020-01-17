use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

use comptes_vl::rational::ParseRationalError;
use comptes_vl::rational::{rational_from_str, rational_to_string};

#[derive(Debug, Clone)]
enum ParseError {
    UnknownUser(String),
    RationalParsingFailed(ParseRationalError),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnknownUser(user) => {
                write!(f, "Unknown user: {}", user)
            }
            ParseError::RationalParsingFailed(error) => {
                write!(f, "Could not parse rational: {}.", error)
            }
        }
    }
}

impl Error for ParseError {}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Purchase {
    descr: String,
    who: String,
    amount: String,
    benef_to_shares: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SerializedAccounts {
    users: Vec<String>,
    purchases: Vec<Purchase>,
}

impl SerializedAccounts {
    pub fn parse(self) -> Result<ParsedAccounts, ParseError> {
        let mut users = self.users;
        users.sort();
        users.dedup();
        let mut purchases = Vec::with_capacity(self.purchases.len());
        for purchase in self.purchases {
            let user_id = users
                .binary_search(&purchase.who)
                .or(Err(ParseError::UnknownUser(purchase.who)))?;
            let amount = rational_from_str(&purchase.amount)
                .map_err(|e| ParseError::RationalParsingFailed(e))?;
            let mut benef_to_shares = vec![Rational64::new(0, 1); users.len()];
            for (benef_id, benef) in users.iter().enumerate() {
                if let Some(shares) = purchase.benef_to_shares.get(benef) {
                    benef_to_shares[benef_id] = rational_from_str(shares)
                        .map_err(|e| ParseError::RationalParsingFailed(e))?;
                }
            }
            purchases.push(ParsedPurchase {
                descr: purchase.descr,
                who_paid: user_id,
                amount,
                benef_to_shares,
            });
        }
        Ok(ParsedAccounts { users, purchases })
    }
}

#[derive(Debug)]
struct ParsedPurchase {
    descr: String,
    who_paid: usize,
    amount: Rational64,
    benef_to_shares: Vec<Rational64>,
}

#[derive(Debug)]
struct ParsedAccounts {
    users: Vec<String>,
    purchases: Vec<ParsedPurchase>,
}

impl ParsedAccounts {
    /// Compute the balance for each user
    fn user_balances(&self) -> Vec<Rational64> {
        let zero = Rational64::new(0, 1);
        let mut balances = vec![zero; self.users.len()];
        for purchase in &self.purchases {
            let total_shares: Rational64 =
                purchase.benef_to_shares.iter().sum();
            if total_shares == zero {
                eprintln!(
                    "Warning, transaction {:?} is ignored: shares sum to zero",
                    purchase,
                );
                continue;
            }
            for (user_id, share) in purchase.benef_to_shares.iter().enumerate()
            {
                balances[user_id] += purchase.amount * share / total_shares;
            }
            balances[purchase.who_paid] -= purchase.amount;
        }
        balances
    }

    pub fn print_balances(&self, nb_max_decimals: u8) {
        let balances = self.user_balances();
        for (user, balance) in self.users.iter().zip(&balances) {
            println!(
                "{} has a balance of: {}",
                user,
                rational_to_string(*balance, nb_max_decimals),
            );
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let accounts_path = "./input.yml";
    let mut accounts_file = std::fs::File::open(&accounts_path)?;
    let accounts: SerializedAccounts = serde_yaml::from_reader(accounts_file)?;
    dbg!(&accounts);
    let accounts = accounts.parse()?;
    dbg!(&accounts);
    dbg!(accounts.user_balances());
    accounts.print_balances(2);

    let accounts = SerializedAccounts {
        users: vec![
            "Simon".to_string(),
            "Shuba".to_string(),
            "Eska".to_string(),
        ],
        purchases: vec![Purchase {
            descr: "jambon".to_string(),
            who: "Simon".to_string(),
            amount: "15".to_string(),
            benef_to_shares: [
                ("Simon".to_string(), "1".to_string()),
                ("Shuba".to_string(), "2".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
        }],
    };
    let yaml = serde_yaml::to_string(&accounts)?;
    println!("{}", yaml);
    Ok(())
}
