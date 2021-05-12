//! GUI based on the `iced` library.

use iced::{
    Application, Clipboard, Command, Element, Settings, Subscription, Text,
};

use std::path::PathBuf;

mod accounts;
mod file_selector;
mod style;
mod transaction;
use file_selector::FileSelector;

#[cfg(target_arch = "wasm32")]
mod file_input;
#[cfg(target_arch = "wasm32")]
mod url;

#[cfg(target_arch = "wasm32")]
use crate::local_storage;
use accounts::Accounts;

pub fn run(file: Option<PathBuf>) {
    Aaacs::run(Settings {
        flags: AppFlags { file },
        ..Settings::default()
    })
    .expect("Error while running aaacs");
}

enum Aaacs {
    HomePage(FileSelector),
    Editing(Accounts),
    FatalError(String),
}

#[derive(Debug, Clone)]
enum Message {
    HomePage(file_selector::Message),
    Editing(accounts::Message),
    #[cfg(not(target_arch = "wasm32"))]
    Event(iced_native::Event),
}

#[derive(Default)]
pub struct AppFlags {
    file: Option<std::path::PathBuf>,
}

impl Application for Aaacs {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Flags = AppFlags;

    fn new(flags: AppFlags) -> (Self, Command<Self::Message>) {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = flags;
            match local_storage::get_item("latest_state") {
                Some(state) => {
                    if let Some(title) = state.split(":").skip(1).take(1).next()
                    {
                        (
                            Aaacs::Editing(Accounts::new(title.to_string())),
                            Command::none(),
                        )
                    } else {
                        (Aaacs::HomePage(FileSelector::new()), Command::none())
                    }
                }
                None => (Aaacs::HomePage(FileSelector::new()), Command::none()),
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = flags.file {
                let accounts_file = std::fs::File::open(&path);
                match accounts_file {
                    Ok(accounts_file) => {
                        let accounts = Accounts::from_yaml_path_and_reader(
                            path.clone(),
                            accounts_file,
                        );
                        match accounts {
                            Ok(accounts) => {
                                (Aaacs::Editing(accounts), Command::none())
                            }
                            Err(err) => (
                                Aaacs::FatalError(format!(
                                    "Could not parse yaml file {:?}: {:?}",
                                    path, err,
                                )),
                                Command::none(),
                            ),
                        }
                    }
                    Err(err) => match err.kind() {
                        std::io::ErrorKind::NotFound => (
                            Aaacs::Editing(
                                Accounts::from_inexistent_yaml_path(path),
                            ),
                            Command::none(),
                        ),
                        _ => (
                            Aaacs::FatalError(format!(
                                "I/O error opening {:?}: {:?}",
                                path, err,
                            )),
                            Command::none(),
                        ),
                    },
                }
            } else {
                (Aaacs::Editing(Default::default()), Command::none())
            }
        }
    }

    fn title(&self) -> String {
        match self {
            Aaacs::Editing(accounts) => {
                format!("aaacs: {}", accounts.title())
            }
            _ => "aaacs".into(),
        }
    }

    fn update(
        &mut self,
        message: Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        match message {
            #[cfg(target_arch = "wasm32")]
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
            #[cfg(not(target_arch = "wasm32"))]
            Message::Event(event) => match event {
                iced_native::Event::Keyboard(e) => match self {
                    Aaacs::Editing(accounts) => accounts.handle_kb_event(e),
                    _ => (),
                },
                _ => (),
            },
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
                Aaacs::FatalError(_) => Ok(()),
            };
        }
        Command::none()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message::Event)
    }

    fn view(&mut self) -> Element<Message> {
        match self {
            Aaacs::HomePage(selector) => {
                selector.view().map(|msg| Message::HomePage(msg))
            }
            Aaacs::Editing(accounts) => {
                accounts.view().map(|msg| Message::Editing(msg))
            }
            Aaacs::FatalError(error) => {
                Text::new(format!("Fatal error: {}", error))
                    .color([1.0, 0., 0.])
                    .into()
            }
        }
    }
}
