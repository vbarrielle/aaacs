//! GUI based on the `iced` library.

use iced::{Element, Sandbox, Settings};

mod accounts;
mod file_selector;
mod transaction;
use file_selector::FileSelector;

use accounts::Accounts;

pub fn run() {
    Aaacs::run(Settings::default());
}

enum Aaacs {
    HomePage(FileSelector),
    Editing(Accounts),
}

#[derive(Debug, Clone)]
enum Message {
    HomePage(file_selector::Message),
    Editing(accounts::Message),
}

impl Sandbox for Aaacs {
    type Message = Message;

    fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Aaacs::HomePage(Default::default())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Aaacs::Editing(Default::default())
        }
    }

    fn title(&self) -> String {
        String::from("aacs")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Editing(msg) => {
                if let Aaacs::Editing(accounts) = self {
                    accounts.update(msg);
                }
            }
            Message::HomePage(msg) => {
                if let Aaacs::HomePage(selector) = self {
                    if let Some(title) = selector.update(msg) {
                        *self =
                            Aaacs::Editing(Accounts::new(title.to_string()));
                    }
                }
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        match self {
            Aaacs::HomePage(selector) => {
                selector.view().map(|msg| Message::HomePage(msg))
            }
            Aaacs::Editing(accounts) => {
                accounts.view().map(|msg| Message::Editing(msg))
            }
        }
    }
}
