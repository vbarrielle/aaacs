//! Implementations of the internal representation of accounts
use std::collections::HashMap;
use std::error::Error;

use num_rational::Rational64;
use serde::{Deserialize, Serialize};

use crate::rational::ParseRationalError;
use crate::rational::{rational_from_str, rational_to_string};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    EmptyUser,
    EmptyDescr,
    UnknownUser(String),
    RationalParsingFailed(ParseRationalError),
    UserAlreadyPresent(String),
    UserHasData(String),
    InvalidPurchase(usize),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EmptyUser => {
                write!(f, "Cannot add the empty string as a new user")
            }
            ParseError::EmptyDescr => {
                write!(f, "Cannot add the empty string as a description")
            }
            ParseError::UnknownUser(user) => {
                write!(f, "Unknown user: {}", user)
            }
            ParseError::RationalParsingFailed(error) => {
                write!(f, "Could not parse rational: {}.", error)
            }
            ParseError::UserAlreadyPresent(user) => {
                write!(f, "Cannot insert user {} twice.", user)
            }
            ParseError::UserHasData(user) => write!(
                f,
                "Cannot remove user {}, he has paid a transaction or shares.",
                user,
            ),
            ParseError::InvalidPurchase(index) => {
                write!(f, "Transaction {} does not exist.", index,)
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
pub struct SerializedAccounts {
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

#[derive(Debug, PartialEq, Clone)]
pub struct ParsedPurchase {
    descr: String,
    who_paid: usize,
    amount: Rational64,
    benef_to_shares: Vec<Rational64>,
}

impl ParsedPurchase {
    pub fn descr(&self) -> &str {
        &self.descr
    }

    pub fn who_paid<'a>(&self, accounts: &'a ParsedAccounts) -> &'a str {
        &accounts.users[self.who_paid]
    }

    pub fn amount(&self) -> Rational64 {
        self.amount
    }

    pub fn benef_to_shares<'a>(
        &'a self,
        accounts: &'a ParsedAccounts,
    ) -> impl Iterator<Item = (&'a str, Rational64)> {
        self.benef_to_shares
            .iter()
            .enumerate()
            .map(move |(uid, share)| (&accounts.users[uid][..], *share))
    }

    pub fn shares(&self) -> &[Rational64] {
        &self.benef_to_shares[..]
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ParsedAccounts {
    users: Vec<String>,
    purchases: Vec<ParsedPurchase>,
}

impl ParsedAccounts {
    /// Return the existing users
    pub fn users(&self) -> &[String] {
        &self.users[..]
    }

    pub fn purchases(&self) -> &[ParsedPurchase] {
        &self.purchases[..]
    }

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
                balances[user_id] -= purchase.amount * share / total_shares;
            }
            balances[purchase.who_paid] += purchase.amount;
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

    /// Add a new user to the accounts. Its shares in all existing transactions
    /// will be zero.
    pub fn add_user(&mut self, user: String) -> Result<(), ParseError> {
        if user.len() == 0 {
            return Err(ParseError::EmptyUser);
        }
        let loc = self.users.binary_search(&user);
        let index = match loc {
            Ok(_) => return Err(ParseError::UserAlreadyPresent(user)),
            Err(index) => index,
        };
        self.users.insert(index, user);
        let zero = Rational64::new(0, 1);
        for purchase in self.purchases.iter_mut() {
            purchase.benef_to_shares.insert(index, zero);
            if purchase.who_paid >= index {
                purchase.who_paid += 1;
            }
        }
        Ok(())
    }

    pub fn remove_user(&mut self, user: String) -> Result<(), ParseError> {
        let loc = self.users.binary_search(&user);
        let index = match loc {
            Err(_) => return Err(ParseError::UnknownUser(user)),
            Ok(index) => index,
        };
        for purchase in self.purchases.iter() {
            if purchase.who_paid == index
                || purchase.benef_to_shares[index] > 0.into()
            {
                return Err(ParseError::UserHasData(user));
            }
        }
        self.users.remove(index);
        for purchase in self.purchases.iter_mut() {
            purchase.benef_to_shares.remove(index);
            if purchase.who_paid >= index {
                purchase.who_paid -= 1;
            }
        }
        Ok(())
    }

    /// Add a purchase to the accounts, with empty shares. The purchase shares
    /// shall be filled-in later.
    ///
    /// On success, returns the index of the added purchase.
    pub fn add_purchase(
        &mut self,
        descr: String,
        who_paid: String,
        amount: Rational64,
    ) -> Result<usize, ParseError> {
        if who_paid.len() == 0 {
            return Err(ParseError::EmptyUser);
        }
        if descr.len() == 0 {
            return Err(ParseError::EmptyDescr);
        }
        let loc = self.users.binary_search(&who_paid);
        let who_paid = match loc {
            Err(_) => return Err(ParseError::UnknownUser(who_paid)),
            Ok(index) => index,
        };
        self.purchases.push(ParsedPurchase {
            descr,
            who_paid,
            amount,
            benef_to_shares: vec![0.into(); self.users.len()],
        });
        Ok(self.purchases.len() - 1)
    }

    /// Modify the shares for the requested user in a selected purchase
    pub fn set_purchase_user_share(
        &mut self,
        purchase_idx: usize,
        user: String,
        share: Rational64,
    ) -> Result<(), ParseError> {
        let loc = self.users.binary_search(&user);
        let user = match loc {
            Err(_) => return Err(ParseError::UnknownUser(user)),
            Ok(index) => index,
        };
        if purchase_idx >= self.purchases.len() {
            return Err(ParseError::InvalidPurchase(purchase_idx));
        }
        self.purchases[purchase_idx].benef_to_shares[user] = share;
        Ok(())
    }

    // Change the user who paid for a purchase
    pub fn change_purchase_creditor(
        &mut self,
        purchase_idx: usize,
        who_paid: String,
    ) -> Result<(), ParseError> {
        let loc = self.users.binary_search(&who_paid);
        let who_paid = match loc {
            Err(_) => return Err(ParseError::UnknownUser(who_paid)),
            Ok(index) => index,
        };
        if purchase_idx >= self.purchases.len() {
            return Err(ParseError::InvalidPurchase(purchase_idx));
        }
        self.purchases[purchase_idx].who_paid = who_paid;
        Ok(())
    }

    // Change the user who paid for a purchase
    pub fn change_purchase_amount(
        &mut self,
        purchase_idx: usize,
        amount: Rational64,
    ) -> Result<(), ParseError> {
        if purchase_idx >= self.purchases.len() {
            return Err(ParseError::InvalidPurchase(purchase_idx));
        }
        self.purchases[purchase_idx].amount = amount;
        Ok(())
    }

    // Remove a purchase
    pub fn remove_purchase(
        &mut self,
        purchase_idx: usize,
    ) -> Result<(), ParseError> {
        if purchase_idx >= self.purchases.len() {
            return Err(ParseError::InvalidPurchase(purchase_idx));
        }
        self.purchases.remove(purchase_idx);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{ParseError, ParsedAccounts, ParsedPurchase};

    #[test]
    fn add_remove_user() {
        let mut accounts = ParsedAccounts {
            users: vec![
                "Eska".to_string(),
                "Shuba".to_string(),
                "Simon".to_string(),
            ],
            purchases: vec![
                ParsedPurchase {
                    descr: "jambon".to_string(),
                    who_paid: 0,
                    amount: 15.into(),
                    benef_to_shares: vec![1.into(), 2.into(), 1.into()],
                },
                ParsedPurchase {
                    descr: "vin".to_string(),
                    who_paid: 2,
                    amount: 10.into(),
                    benef_to_shares: vec![0.into(), 2.into(), 1.into()],
                },
            ],
        };
        let orig = accounts.clone();
        assert_eq!(
            accounts.add_user("Eska".to_string()),
            Err(ParseError::UserAlreadyPresent("Eska".to_string()))
        );

        assert!(accounts.add_user("PlappMachine".to_string()).is_ok());
        let expected = ParsedAccounts {
            users: vec![
                "Eska".to_string(),
                "PlappMachine".to_string(),
                "Shuba".to_string(),
                "Simon".to_string(),
            ],
            purchases: vec![
                ParsedPurchase {
                    descr: "jambon".to_string(),
                    who_paid: 0,
                    amount: 15.into(),
                    benef_to_shares: vec![
                        1.into(),
                        0.into(),
                        2.into(),
                        1.into(),
                    ],
                },
                ParsedPurchase {
                    descr: "vin".to_string(),
                    who_paid: 3,
                    amount: 10.into(),
                    benef_to_shares: vec![
                        0.into(),
                        0.into(),
                        2.into(),
                        1.into(),
                    ],
                },
            ],
        };
        assert_eq!(accounts, expected);

        assert_eq!(
            accounts.remove_user("Eska".to_string()),
            Err(ParseError::UserHasData("Eska".to_string())),
        );
        assert!(accounts.remove_user("PlappMachine".to_string()).is_ok());
        assert_eq!(accounts, orig);
    }

    #[test]
    fn add_purchase() {
        let mut accounts = ParsedAccounts {
            users: vec![
                "Eska".to_string(),
                "Shuba".to_string(),
                "Simon".to_string(),
            ],
            purchases: vec![
                ParsedPurchase {
                    descr: "jambon".to_string(),
                    who_paid: 0,
                    amount: 15.into(),
                    benef_to_shares: vec![1.into(), 2.into(), 1.into()],
                },
                ParsedPurchase {
                    descr: "vin".to_string(),
                    who_paid: 2,
                    amount: 10.into(),
                    benef_to_shares: vec![0.into(), 2.into(), 1.into()],
                },
            ],
        };
        let purchase_idx = accounts
            .add_purchase("fromage".to_string(), "Shuba".to_string(), 23.into())
            .unwrap();
        accounts
            .set_purchase_user_share(
                purchase_idx,
                "Eska".to_string(),
                42.into(),
            )
            .unwrap();
        accounts
            .change_purchase_creditor(purchase_idx, "Simon".to_string())
            .unwrap();
        accounts
            .change_purchase_amount(purchase_idx, 20.into())
            .unwrap();
        accounts.remove_purchase(0).unwrap();
        let expected = ParsedAccounts {
            users: vec![
                "Eska".to_string(),
                "Shuba".to_string(),
                "Simon".to_string(),
            ],
            purchases: vec![
                ParsedPurchase {
                    descr: "vin".to_string(),
                    who_paid: 2,
                    amount: 10.into(),
                    benef_to_shares: vec![0.into(), 2.into(), 1.into()],
                },
                ParsedPurchase {
                    descr: "fromage".to_string(),
                    who_paid: 2,
                    amount: 20.into(),
                    benef_to_shares: vec![42.into(), 0.into(), 0.into()],
                },
            ],
        };
        assert_eq!(accounts, expected);
    }
}
