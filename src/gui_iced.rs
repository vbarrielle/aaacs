//! GUI based on the `iced` library.


use iced::{
    button, Button, Column, Element, Sandbox, Settings, Text, TextInput,
    text_input,
};

use crate::accounts::{ParsedAccounts, ParseError};

pub fn run() {
    Accounts::run(Settings::default());
}

#[derive(Default)]
struct Accounts {
    accounts: ParsedAccounts,
    last_error: Option<ParseError>,
    new_user: String,
    new_user_state: text_input::State,
    new_purchase_btn_state: button::State,
}

#[derive(Debug, Clone)]
enum Message {
    NewUserStrChange(String),
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
                self.accounts.add_user(self.new_user.clone())
            },
            Message::NewUserStrChange(new_user) => {
                self.new_user = new_user;
                Ok(())
            },
            Message::AddPurchase => {
                self.accounts.add_purchase(
                    "Transaction".to_string(),
                    self.accounts.users()[0].to_string(),
                    0.into(),
                ).map(|_| ())
            },
        }.err();
    }

    fn view(&mut self) -> Element<Message> {
        let mut column = Column::new().padding(20);
        for user in self.accounts.users() {
            column = column.push(Text::new(user.clone()));
        }
        column = column
            .push(
                TextInput::new(
                    &mut self.new_user_state,
                    "New user",
                    &self.new_user,
                    Message::NewUserStrChange,
                ).on_submit(Message::AddUser),
            );
        for purchase in self.accounts.purchases() {
            column = column.push(
                Text::new(purchase.descr())
            );
        }
        if self.accounts.users().len() > 0 {
            column = column
                .push(
                    Button::new(
                        &mut self.new_purchase_btn_state,
                        Text::new("Add transaction"),
                    ).on_press(Message::AddPurchase)
                );
        }
        column.into()
    }
}
