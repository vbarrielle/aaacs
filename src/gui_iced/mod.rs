//! GUI based on the `iced` library.

use iced::{Element, Sandbox, Settings};

mod accounts;
mod transaction;

use accounts::Accounts;

pub fn run() {
    Aaacs::run(Settings::default());
}

//#[derive(Default)]
struct Aaacs {
    sheet: Accounts,
}

#[derive(Debug, Clone)]
enum Message {
    Sheet(accounts::Message),
}

impl Sandbox for Aaacs {
    type Message = Message;

    fn new() -> Self {
        Self {
            sheet: Accounts::new(),
        }
    }

    fn title(&self) -> String {
        String::from("aacs")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Sheet(msg) => {
                self.sheet.update(msg);
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        self.sheet.view().map(|msg| Message::Sheet(msg))
    }
}
