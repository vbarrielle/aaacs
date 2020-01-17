use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
enum ParseError {
    UnknownUser(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnknownUser(user) => {
                write!(f, "Unknown user: {}", user)
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
    pub fn parse(self) -> Result<ParsedAccounts, Box<dyn Error>> {
        let mut users = self.users;
        users.sort();
        users.dedup();
        let mut purchases = Vec::with_capacity(self.purchases.len());
        for purchase in self.purchases {
            let user_id = users
                .binary_search(&purchase.who)
                .or(Err(ParseError::UnknownUser(purchase.who)))?;
            let amount = rational_from_str(&purchase.amount)?;
            let mut benef_to_shares = vec![Rational64::new(0, 1); users.len()];
            for (benef_id, benef) in users.iter().enumerate() {
                if let Some(shares) = purchase.benef_to_shares.get(benef) {
                    benef_to_shares[benef_id] = rational_from_str(shares)?;
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

fn rational_from_str(rat_str: &str) -> Result<Rational64, Box<dyn Error>> {
    let mut parts_iter = rat_str.split('.');
    let integral_part = parts_iter.next().ok_or("Empty string")?;
    let integral_part: i64 = integral_part.parse()?;
    let integral_part = Rational64::new(integral_part, 1);
    if let Some(decimal_part) = parts_iter.next() {
        let nb_decimals = decimal_part.len();
        Ok(integral_part
            + Rational64::new(
                decimal_part.parse()?,
                10_i64.pow(nb_decimals as u32),
            ))
    } else {
        Ok(integral_part)
    }
}

/// Convert a rational number to a decimal string representation. Rounding
/// is performed to the closest decimal number with the required number of
/// decimals.
fn rational_to_string(rat: Rational64, nb_max_decimals: u8) -> String {
    let rat = rat.reduced();
    if rat.is_integer() {
        format!("{}", rat.to_integer())
    } else {
        let integral_part = rat.trunc();
        let mut fract_part = rat.fract();
        if fract_part < Rational64::new(0, 1) {
            fract_part = -fract_part;
        };
        // The fractional part is a fraction m / n where 0 < m < n.
        // We want to compute a decimal approximation, ie find two integers
        // p and q such that p / 10^q < m / n and the difference is minimal,
        // under the constraint that q <= nb_max_decimals.
        let mut q = nb_max_decimals;
        let mut p = (*fract_part.numer() as f64 * 10.0_f64.powi(q as i32)
            / *fract_part.denom() as f64)
            .round() as i64;
        while p % 10 == 0 {
            q -= 1;
            p = p / 10;
        }
        let integral_part = integral_part.to_integer();
        format!(
            "{integral_part}.{decimal_part:0width$}",
            integral_part = integral_part,
            decimal_part = p,
            width = q as usize,
        )
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

#[cfg(test)]
mod test {
    use super::Rational64;

    #[test]
    fn rational_from_str() {
        assert_eq!(
            super::rational_from_str(&"10").unwrap(),
            Rational64::new(10, 1),
        );
        assert_eq!(
            super::rational_from_str(&"10.5").unwrap(),
            Rational64::new(105, 10),
        );
        assert_eq!(
            super::rational_from_str(&"0.5").unwrap(),
            Rational64::new(5, 10),
        );
        assert_eq!(
            super::rational_from_str(&"3.00523").unwrap(),
            Rational64::new(300523, 100000),
        );
    }

    #[test]
    fn rational_to_string() {
        let nb_max_decimals = 4;
        assert_eq!(
            &super::rational_to_string(Rational64::new(10, 1), nb_max_decimals),
            &"10",
        );
        assert_eq!(
            &super::rational_to_string(
                Rational64::new(105, 10),
                nb_max_decimals
            ),
            &"10.5",
        );
        assert_eq!(
            &super::rational_to_string(
                Rational64::new(105, 100),
                nb_max_decimals
            ),
            &"1.05",
        );
        assert_eq!(
            &super::rational_to_string(
                Rational64::new(-105, 100),
                nb_max_decimals
            ),
            &"-1.05",
        );
        assert_eq!(
            &super::rational_to_string(
                Rational64::new(99, 30),
                nb_max_decimals
            ),
            &"3.3",
        );
        assert_eq!(
            &super::rational_to_string(
                Rational64::new(99, 29),
                nb_max_decimals
            ),
            &"3.4138",
        );
    }
}
