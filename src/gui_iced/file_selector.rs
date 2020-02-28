//! A file selector for local storage

use iced::{button, text_input, Button, Column, Element, Text, TextInput};

#[cfg(target_arch = "wasm32")]
use crate::local_storage;

#[derive(Default)]
pub struct FileSelector {
    new_accounts: String,
    new_accounts_state: text_input::State,
    new_accounts_btn_state: button::State,
    existing: Vec<existing_accounts::ExistingAccounts>,
}

#[derive(Debug, Clone)]
pub enum Message {
    NewAccountsStrChange(String),
    OpenAccounts(usize),
    CreateAccounts,
}

impl FileSelector {
    pub fn new() -> Self {
        let mut existing = Vec::with_capacity(16);
        #[cfg(target_arch = "wasm32")]
        {
            for (title, json) in local_storage::saved_accounts() {
                existing.push(existing_accounts::ExistingAccounts::new(
                    title, &json,
                ));
            }
        }
        Self {
            existing,
            ..Default::default()
        }
    }

    pub fn update(&mut self, message: Message) -> Option<&str> {
        match message {
            Message::CreateAccounts => Some(&self.new_accounts),
            Message::NewAccountsStrChange(title) => {
                self.new_accounts = title;
                None
            }
            Message::OpenAccounts(i) => Some(&self.existing[i].title()),
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
        column = column.push(Text::new("Latest accounts:"));
        for (idx, existing) in self.existing.iter_mut().enumerate() {
            column = column
                .push(existing.view().map(move |_| Message::OpenAccounts(idx)));
        }
        column.into()
    }
}

mod existing_accounts {

    #[cfg(target_arch = "wasm32")]
    use crate::gui_iced::url::UrlA;
    use iced::{button, Button, Element, Row, Text};

    #[derive(Default)]
    pub struct ExistingAccounts {
        title: String,
        json_b64: String,
        open_button_state: button::State,
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        Selected,
    }

    impl ExistingAccounts {
        pub fn new(title: String, json: &str) -> Self {
            Self {
                title,
                json_b64: data_encoding::BASE64.encode(json.as_bytes()),
                ..Default::default()
            }
        }

        pub fn title(&self) -> &str {
            &self.title
        }

        pub fn view(&mut self) -> Element<Message> {
            let mut row =
                Row::new().spacing(10).push(Text::new(&self.title)).push(
                    Button::new(&mut self.open_button_state, Text::new("Open"))
                        .background(iced::Background::Color(
                            [0., 0.8, 0.8].into(),
                        ))
                        .border_radius(5)
                        .padding(2)
                        .on_press(Message::Selected),
                );
            #[cfg(target_arch = "wasm32")]
            {
                row = row.push(UrlA {
                    text: "Download json".to_string(),
                    href: format!("data:text/plain;base64,{}", self.json_b64),
                    download: format!("{}.json", self.title),
                });
            }
            row.into()
        }
    }
}
