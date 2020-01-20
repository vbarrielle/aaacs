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
}

#[derive(Debug, Clone)]
enum Message {
    NewUserStrChange(String),
    AddUser,
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
            }
        }.err();
    }

    fn view(&mut self) -> Element<Message> {
        let mut column = Column::new().padding(20);
        for user in self.accounts.users() {
            column = column.push(Text::new(user.clone()));
        }
        column
            .push(
                TextInput::new(
                    &mut self.new_user_state,
                    "New user",
                    &self.new_user,
                    Message::NewUserStrChange,
                ).on_submit(Message::AddUser),
            )
            .into()
    }
}
