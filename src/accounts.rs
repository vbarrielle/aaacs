//! Implementations of the internal representation of accounts
use std::error::Error;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use num_rational::Rational64;

use crate::rational::ParseRationalError;
use crate::rational::{rational_from_str, rational_to_string};

#[derive(Debug, Clone)]
pub enum ParseError {
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

#[derive(Debug)]
struct ParsedPurchase {
    descr: String,
    who_paid: usize,
    amount: Rational64,
    benef_to_shares: Vec<Rational64>,
}

#[derive(Debug)]
pub struct ParsedAccounts {
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

