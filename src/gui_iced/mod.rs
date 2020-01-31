//! GUI based on the `iced` library.

use iced::{
    button, text_input, Button, Column, Element, Row, Sandbox, Settings, Text,
    TextInput,
};

use crate::accounts::{ParseError, ParsedAccounts};
use crate::rational::rational_to_string;

mod transaction;

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
    new_transaction: transaction::Transaction,
    #[cfg(feature = "debug")]
    latest_message: Option<Message>,
}

#[derive(Debug, Clone)]
enum Message {
    NewUserStrChange(String),
    NewTransaction(transaction::Message),
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
        #[cfg(feature = "debug")]
        {
            self.latest_message = Some(message.clone());
        }

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
            Message::NewTransaction(message) => {
                self.new_transaction.update(message, self.accounts.users());
                Ok(())
            }
            Message::AddPurchase => {
                let purchase_idx = self.accounts.add_purchase(
                    self.new_transaction.take_descr(),
                    self.new_transaction.take_creditor(),
                    self.new_transaction.take_amount(),
                );
                if let Ok(purchase_idx) = purchase_idx {
                    self.accounts.set_purchase_shares(
                        purchase_idx,
                        self.new_transaction.per_user_shares(),
                    )
                } else {
                    purchase_idx.map(|_| ())
                }
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
            for (user, share) in purchase.benef_to_shares(&self.accounts) {
                row = row.push(Text::new(format!(
                    "User {} has a share of {}",
                    user,
                    rational_to_string(share, 2),
                )));
            }
            column = column.push(row);
        }
        if self.accounts.users().len() > 0 {
            let can_add_transaction = self.new_transaction.is_valid();
            column = column.push(
                self.new_transaction
                    .view(self.accounts.users())
                    .map(|message| Message::NewTransaction(message)),
            );
            if can_add_transaction {
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
        #[cfg(feature = "debug")]
        {
            if let Some(latest_message) = &self.latest_message {
                column = column.push(Text::new(format!(
                    "Latest message: {:?}",
                    latest_message
                )));
            }
        }
        column.into()
    }
}
