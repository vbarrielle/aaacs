//! Widget describing a transaction being edited

use iced::{text_input, Column, Element, Radio, Row, TextInput};

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
}

#[derive(Debug, Clone)]
pub enum Message {
    DescrStrChange(String),
    UserSelected(usize),
    AmountStrChange(String),
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
}

mod shares {

    use iced::{text_input, Element, TextInput};
    use crate::rational::rational_from_str;

    #[derive(Default)]
    pub struct Share {
        value: String,
        state: text_input::State,
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        StrChange(String),
    }

    impl Share {
        pub fn update(&mut self, message: Message, users: &[String]) {
            match message {
                Message::StrChange(share) => {
                    if share.len() == 0
                        || rational_from_str(&share).is_ok()
                    {
                        self.value = share;
                    }
                }
            }
        }

        pub fn view(&mut self, users: &[String]) -> Element<Message> {
            TextInput::new(
                &mut self.state,
                "Share",
                &self.value,
                Message::StrChange,
            ).into()
        }
    }
}
