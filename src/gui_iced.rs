//! GUI based on the `iced` library.

use iced::{
    button, text_input, Button, Column, Element, Radio, Row, Sandbox, Settings,
    Text, TextInput,
};

use crate::accounts::{ParseError, ParsedAccounts};
use crate::rational::{rational_from_str, rational_to_string};

pub fn run() {
    Accounts::run(Settings::default());
}

#[derive(Default)]
struct Accounts {
    accounts: ParsedAccounts,
    last_error: Option<ParseError>,
    new_user: String,
    new_user_btn_state: button::State,
    new_user_state: text_input::State,
    new_purchase_btn_state: button::State,
    new_transaction_descr: String,
    new_transaction_descr_state: text_input::State,
    new_transaction_uid: usize,
    new_transaction_creditor: String,
    new_transaction_amount: String,
    new_transaction_amount_state: text_input::State,
}

#[derive(Debug, Clone)]
enum Message {
    NewUserStrChange(String),
    NewDescrStrChange(String),
    UserSelected(usize),
    NewAmountStrChange(String),
    AddUser,
    AddPurchase,
}

impl Sandbox for Accounts {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("aacs")
    }

    fn update(&mut self, message: Message) {
        self.last_error = match message {
            Message::AddUser => {
                let mut new_user = String::new();
                std::mem::swap(&mut self.new_user, &mut new_user);
                self.accounts.add_user(new_user)
            }
            Message::NewUserStrChange(new_user) => {
                self.new_user = new_user;
                Ok(())
            }
            Message::UserSelected(uid) => {
                self.new_transaction_creditor =
                    self.accounts.users()[uid].to_string();
                self.new_transaction_uid = uid;
                Ok(())
            }
            Message::NewDescrStrChange(new_descr) => {
                self.new_transaction_descr = new_descr;
                Ok(())
            }
            Message::NewAmountStrChange(new_amount) => {
                if new_amount.len() == 0
                    || rational_from_str(&new_amount).is_ok()
                {
                    self.new_transaction_amount = new_amount;
                }
                Ok(())
            }
            Message::AddPurchase => {
                let mut new_transaction_descr = String::new();
                let mut new_transaction_creditor = String::new();
                let mut new_transaction_amount = String::new();
                std::mem::swap(
                    &mut new_transaction_descr,
                    &mut self.new_transaction_descr,
                );
                std::mem::swap(
                    &mut new_transaction_creditor,
                    &mut self.new_transaction_creditor,
                );
                std::mem::swap(
                    &mut new_transaction_amount,
                    &mut self.new_transaction_amount,
                );
                self.accounts
                    .add_purchase(
                        new_transaction_descr,
                        new_transaction_creditor,
                        // No Panic: validity of rational enforced by GUI
                        rational_from_str(&new_transaction_amount).unwrap(),
                    )
                    .map(|_| ())
            }
        }
        .err();
    }

    fn view(&mut self) -> Element<Message> {
        let mut column = Column::new().padding(20);
        column = column
            .push(Text::new(format!("{} users:", self.accounts.users().len())));
        for user in self.accounts.users() {
            column = column.push(Text::new(user.clone()));
        }
        column = column.push(
            Row::new()
                .push(
                    TextInput::new(
                        &mut self.new_user_state,
                        "New user",
                        &self.new_user,
                        Message::NewUserStrChange,
                    )
                    .on_submit(Message::AddUser),
                )
                .push(
                    Button::new(
                        &mut self.new_user_btn_state,
                        Text::new("Add user"),
                    )
                    .on_press(Message::AddUser),
                ),
        );
        column = column.push(Text::new(format!(
            "{} transactions:",
            self.accounts.purchases().len()
        )));
        for purchase in self.accounts.purchases() {
            let mut row = Row::new();
            // TODO make editable (if purchase selected?)
            row = row.push(Text::new(format!("{}: ", purchase.descr())));
            row = row.push(Text::new(format!(
                "{} €",
                rational_to_string(purchase.amount(), 2)
            )));
            row = row.push(Text::new(format!(
                ", paid by {}",
                purchase.who_paid(&self.accounts)
            )));
            column = column.push(row);
        }
        if self.accounts.users().len() > 0 {
            let mut user_choice = Column::new();
            self.new_transaction_creditor =
                self.accounts.users()[self.new_transaction_uid].to_string();
            for (uid, user) in self.accounts.users().iter().enumerate() {
                user_choice = user_choice.push(Radio::new(
                    uid,
                    user,
                    Some(self.new_transaction_uid),
                    Message::UserSelected,
                ));
            }
            let row = Row::new()
                .push(TextInput::new(
                    &mut self.new_transaction_descr_state,
                    "Transaction name",
                    &self.new_transaction_descr,
                    Message::NewDescrStrChange,
                ))
                .push(user_choice)
                .push(TextInput::new(
                    &mut self.new_transaction_amount_state,
                    "Amount",
                    &self.new_transaction_amount,
                    Message::NewAmountStrChange,
                ));
            column = column.push(row);
            if self.new_transaction_descr.len() > 0
                && self.new_transaction_creditor.len() > 0
                && self.new_transaction_amount.len() > 0
            {
                // FIXME this button is shown conditionally in part
                // because there could be a bug where pressing this button
                // before filling the new transaction info could prevent
                // messages from being sent. I do think having this button
                // conditionned on its ability to be clicked is a good idea
                // though.
                column = column.push(
                    Button::new(
                        &mut self.new_purchase_btn_state,
                        Text::new("Add transaction"),
                    )
                    .on_press(Message::AddPurchase),
                );
            }
        }
        if let Some(last_error) = &self.last_error {
            column = column.push(
                Text::new(format!("Error: {}", last_error))
                    .color([1.0, 0., 0.]),
            );
        }
        column.into()
    }
}
