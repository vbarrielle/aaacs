//! A file selector for local storage

use iced::{button, text_input, Button, Column, Element, Row, Text, TextInput};

#[cfg(target_arch = "wasm32")]
use crate::local_storage;

#[derive(Default)]
pub struct FileSelector {
    new_accounts: String,
    new_accounts_state: text_input::State,
    new_accounts_btn_state: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    NewAccountsStrChange(String),
    CreateAccounts,
}

impl FileSelector {
    pub fn update(&mut self, message: Message) -> Option<&str> {
        match message {
            Message::CreateAccounts => Some(&self.new_accounts),
            Message::NewAccountsStrChange(title) => {
                self.new_accounts = title;
                None
            }
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let mut column = Column::new().padding(20).spacing(10).max_width(800);
        column = column.push(
            TextInput::new(
                &mut self.new_accounts_state,
                "Title",
                &self.new_accounts,
                Message::NewAccountsStrChange,
            )
            .on_submit(Message::CreateAccounts),
        );
        column = column.push(
            Button::new(
                &mut self.new_accounts_btn_state,
                Text::new("Create accounts"),
            )
            .background(iced::Background::Color([0., 0.8, 0.8].into()))
            .border_radius(5)
            .padding(2)
            .on_press(Message::CreateAccounts),
        );
        column.into()
    }
}
