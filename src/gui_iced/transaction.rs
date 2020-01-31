//! Widget describing a transaction being edited

use iced::{text_input, Column, Element, Radio, Row, Text, TextInput};

use num_rational::Rational64;

use crate::rational::rational_from_str;

#[derive(Default)]
pub struct Transaction {
    descr: String,
    descr_state: text_input::State,
    uid: usize,
    creditor: String,
    amount: String,
    amount_state: text_input::State,
    shares: Vec<shares::Share>,
}

#[derive(Debug, Clone)]
pub enum Message {
    DescrStrChange(String),
    UserSelected(usize),
    AmountStrChange(String),
    ShareChanged(usize, shares::Message),
}

impl Transaction {
    pub fn update(&mut self, message: Message, users: &[String]) {
        match message {
            Message::DescrStrChange(new_descr) => {
                self.descr = new_descr;
            }
            Message::AmountStrChange(new_amount) => {
                if new_amount.len() == 0
                    || rational_from_str(&new_amount).is_ok()
                {
                    self.amount = new_amount;
                }
            }
            Message::UserSelected(uid) => {
                self.creditor = users[uid].to_string();
                self.uid = uid;
            }
            Message::ShareChanged(uid, message) => {
                self.shares[uid].update(message);
            }
        }
    }

    pub fn view(&mut self, users: &[String]) -> Element<Message> {
        let mut user_choice = Column::new();
        self.creditor = users[self.uid].to_string();
        for (uid, user) in users.iter().enumerate() {
            user_choice = user_choice.push(Radio::new(
                uid,
                user,
                Some(self.uid),
                Message::UserSelected,
            ));
        }
        if self.shares.len() != users.len() {
            // number of users has changed, probably because a new user
            // has been added. If that is the case, we are probably the
            // new transaction widget, and the safest thing to do is to
            // reset the shares to an empty value.
            self.shares.clear();
            self.shares.resize(users.len(), Default::default());
        }
        let mut shares_col = Column::new();
        for (uid, share) in self.shares.iter_mut().enumerate() {
            shares_col = shares_col.push(
                Row::new().push(Text::new(&users[uid])).push(
                    share
                        .view()
                        .map(move |msg| Message::ShareChanged(uid, msg)),
                ),
            );
        }
        Row::new()
            .push(TextInput::new(
                &mut self.descr_state,
                "Transaction name",
                &self.descr,
                Message::DescrStrChange,
            ))
            .push(user_choice)
            .push(TextInput::new(
                &mut self.amount_state,
                "Amount",
                &self.amount,
                Message::AmountStrChange,
            ))
            .push(shares_col)
            .into()
    }

    pub fn take_creditor(&mut self) -> String {
        let mut creditor = String::new();
        std::mem::swap(&mut creditor, &mut self.creditor);
        creditor
    }

    pub fn take_amount(&mut self) -> Rational64 {
        // No Panic: validity of rational enforced by GUI
        let rat = rational_from_str(&self.amount).unwrap();
        self.amount.clear();
        rat
    }

    pub fn take_descr(&mut self) -> String {
        let mut descr = String::new();
        std::mem::swap(&mut descr, &mut self.descr);
        descr
    }

    /// Checks wether this transaction contains all the information
    /// to be added to the accounts
    pub fn is_valid(&self) -> bool {
        self.descr.len() > 0 && self.creditor.len() > 0 && self.amount.len() > 0
    }

    /// Get an iterator yielding the user ids of users with set shares
    /// and the corresponding share value.
    pub fn per_user_shares(
        &self,
    ) -> impl Iterator<Item = (usize, Rational64)> + '_ {
        self.shares
            .iter()
            .enumerate()
            .filter_map(|(uid, share)| share.value().map(|val| (uid, val)))
    }
}

mod shares {

    use crate::rational::rational_from_str;
    use iced::{text_input, Element, TextInput};
    use num_rational::Rational64;

    #[derive(Default, Clone)]
    pub struct Share {
        value: String,
        value_parsed: Option<Rational64>,
        state: text_input::State,
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        StrChange(String),
    }

    impl Share {
        pub fn update(&mut self, message: Message) {
            match message {
                Message::StrChange(share) => {
                    if share.len() == 0 {
                        self.value = share;
                        self.value_parsed = None;
                    } else if let Ok(value_parsed) = rational_from_str(&share) {
                        self.value = share;
                        self.value_parsed = Some(value_parsed);
                    }
                }
            }
        }

        pub fn view(&mut self) -> Element<Message> {
            TextInput::new(
                &mut self.state,
                "Share",
                &self.value,
                Message::StrChange,
            )
            .into()
        }

        pub fn value(&self) -> Option<Rational64> {
            self.value_parsed
        }
    }
}
