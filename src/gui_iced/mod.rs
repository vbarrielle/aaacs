//! GUI based on the `iced` library.

use iced::{Element, Sandbox, Settings};

mod accounts;
mod file_selector;
mod transaction;
use file_selector::FileSelector;

#[cfg(target_arch = "wasm32")]
mod file_input;
#[cfg(target_arch = "wasm32")]
mod url;

#[cfg(target_arch = "wasm32")]
use crate::local_storage;
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
            match local_storage::get_item("latest_state") {
                Some(state) => {
                    if let Some(title) = state.split(":").skip(1).take(1).next()
                    {
                        Aaacs::Editing(Accounts::new(title.to_string()))
                    } else {
                        Aaacs::HomePage(FileSelector::new())
                    }
                }
                None => Aaacs::HomePage(FileSelector::new()),
            }
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
            Message::Editing(accounts::Message::GoHome) => {
                *self = Aaacs::HomePage(FileSelector::new());
            }
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
        #[cfg(target_arch = "wasm32")]
        {
            // save the state
            // FIXME warn if saving impossible
            let save_res = match self {
                Aaacs::HomePage(_) => {
                    local_storage::set_item("latest_state", "homepage")
                }
                Aaacs::Editing(accounts) => local_storage::set_item(
                    "latest_state",
                    &format!("editing:{}", accounts.title()),
                ),
            };
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
