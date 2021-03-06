//! An accounting file being edited

use iced::{button, text_input, Button, Column, Element, Row, Text, TextInput};

use std::io::Read;
use std::path::PathBuf;

use crate::accounts::{ParseError, ParsedAccounts};
use crate::gui_iced::style;
use crate::gui_iced::transaction;
#[cfg(target_arch = "wasm32")]
use crate::local_storage;
use crate::rational::rational_to_string;

#[derive(Default)]
pub struct Accounts {
    #[cfg(target_arch = "wasm32")]
    title: String,
    #[cfg(not(target_arch = "wasm32"))]
    path: PathBuf,
    accounts: ParsedAccounts,
    last_error: Option<ParseError>,
    new_user: String,
    new_user_btn_state: button::State,
    new_user_state: text_input::State,
    transactions: Vec<transaction::Transaction>,
    new_purchase_btn_state: button::State,
    new_transaction: transaction::Transaction,
    #[cfg(target_arch = "wasm32")]
    go_home_btn_state: button::State,
    #[cfg(feature = "debug")]
    latest_message: Option<Message>,
    status: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    NewUserStrChange(String),
    NewTransaction(transaction::Message),
    TransactionChange(usize, transaction::Message),
    AddUser,
    AddPurchase,
    #[cfg(target_arch = "wasm32")]
    GoHome,
}

impl Accounts {
    pub fn title(&self) -> std::borrow::Cow<str> {
        #[cfg(target_arch = "wasm32")]
        {
            std::borrow::Cow::Borrowed(&self.title)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.path.to_string_lossy()
        }
    }

    /// Construct the GUI from a JSON serialization
    #[cfg(target_arch = "wasm32")]
    fn from_json(json: &str, title: String) -> Result<Self, ParseError> {
        let accounts = ParsedAccounts::from_json(json)?;
        let transactions = accounts
            .purchases()
            .iter()
            .map(|purch| transaction::Transaction::new(purch, accounts.users()))
            .collect();
        Ok(Accounts {
            title,
            accounts,
            transactions,
            ..Default::default()
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_inexistent_yaml_path(yaml_path: PathBuf) -> Self {
        Accounts {
            path: yaml_path,
            ..Default::default()
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_yaml_path_and_reader<R: Read>(
        yaml_path: PathBuf,
        yaml_reader: R,
    ) -> Result<Self, ParseError> {
        let accounts = ParsedAccounts::from_yaml_reader(yaml_reader)?;
        let transactions = accounts
            .purchases()
            .iter()
            .map(|purch| transaction::Transaction::new(purch, accounts.users()))
            .collect();
        Ok(Accounts {
            path: yaml_path,
            accounts,
            transactions,
            ..Default::default()
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) -> std::io::Result<()> {
        use std::io;
        let file = std::fs::File::create(&self.path)?;
        serde_yaml::to_writer(file, &self.accounts.as_serializable())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    pub fn new(title: String) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let save_name = format!("aaacs:{}", title);
            if let Some(latest) = local_storage::get_item(&save_name) {
                Self::from_json(&latest, title)
                    .unwrap_or_else(|_| Self { ..Self::default() })
            } else {
                Self {
                    title,
                    ..Self::default()
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = title;
            Self::default()
        }
    }

    pub fn update(&mut self, message: Message) {
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
                self.new_transaction.update(
                    message,
                    self.accounts.users(),
                    None,
                );
                Ok(())
            }
            Message::AddPurchase => {
                self.transactions.push(self.new_transaction.clone());
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
            Message::TransactionChange(purchase_idx, message) => {
                let transaction = self.accounts.purchase_mut(purchase_idx);
                match transaction {
                    Ok((transaction, users)) => {
                        self.transactions[purchase_idx].update(
                            message,
                            users,
                            Some(transaction),
                        );
                        Ok(())
                    }
                    Err(err) => Err(err),
                }
            }
            #[cfg(target_arch = "wasm32")]
            Message::GoHome => Err(ParseError::InvalidState(
                "Accounts should not handle GoHome".to_string(),
            )),
        }
        .err();

        #[cfg(target_arch = "wasm32")]
        {
            // save the current state in local storage
            if self.last_error.is_none() {
                let serialized_accounts: Result<String, ParseError> =
                    serde_json::to_string(&self.accounts.as_serializable())
                        .map_err(|e| e.into());
                let save_name = format!("aaacs:{}", self.title);
                let res = serialized_accounts
                    .map(|accs| local_storage::set_item(&save_name, &accs));
                self.last_error = res.err();
            }
        }
    }

    fn set_status(&mut self, msg: &str) {
        self.status.clear();
        self.status.push_str(msg);
    }

    pub fn view(&mut self) -> Element<Message> {
        let mut column = Column::new().padding(20).spacing(10);
        #[cfg(target_arch = "wasm32")]
        {
            column = column.push(
                Button::new(&mut self.go_home_btn_state, Text::new("Home"))
                    .style(style::Button)
                    .padding(2)
                    .on_press(Message::GoHome),
            );
        }
        let mut users_row = Row::new().spacing(20);
        users_row = users_row
            .push(Text::new(format!("{} users:", self.accounts.users().len())));
        for user in self.accounts.users() {
            users_row = users_row.push(
                Text::new(user.clone())
                    .horizontal_alignment(iced::HorizontalAlignment::Center),
            );
        }
        column = column.push(users_row);
        column = column.push(
            Row::new()
                .spacing(10)
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
                    .style(style::Button)
                    .padding(2)
                    .on_press(Message::AddUser),
                ),
        );
        column = column.push(Text::new(format!(
            "{} transactions:",
            self.accounts.purchases().len()
        )));
        for (tid, transaction) in self.transactions.iter_mut().enumerate() {
            column = column.push(
                transaction
                    .view(self.accounts.users())
                    .map(move |msg| Message::TransactionChange(tid, msg)),
            );
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
                    .style(style::Button)
                    .padding(2)
                    .on_press(Message::AddPurchase),
                );
            }
        }

        column = column.push(Text::new("User balances:"));
        let balances = self.accounts.user_balances();
        let users = self.accounts.users();
        for (user, balance) in users.iter().zip(&balances) {
            column = column.push(Text::new(format!(
                "{} has a balance of: {}",
                user,
                rational_to_string(*balance, 2),
            )));
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
            if let Ok(yaml) =
                serde_yaml::to_string(&self.accounts.as_serializable())
            {
                column = column.push(Text::new("YAML serialization:"));
                for line in yaml.lines() {
                    column = column.push(Text::new(line));
                }
            } else {
                column = column.push(
                    Text::new("Could not serialize YAML").color([1.0, 0., 0.]),
                );
            }
        }
        if self.status.len() > 0 {
            column = column
                .push(iced::widget::Space::with_height(iced::Length::Fill));
            column = column.push(Text::new(&self.status));
        }
        column.into()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn handle_kb_event(&mut self, event: iced_native::keyboard::Event) {
        use iced_native::keyboard::{Event, KeyCode, Modifiers};
        match event {
            Event::KeyPressed {
                key_code: KeyCode::S,
                modifiers:
                    modif
                    @
                    Modifiers {
                        control: _,
                        shift: false,
                        alt: false,
                        logo: _,
                    },
            } => {
                if modif.is_command_pressed() {
                    match self.save() {
                        Err(e) => {
                            self.set_status(&format!("Could not save: {}", e));
                        }
                        Ok(_) => {
                            self.set_status(&"Succesful save");
                        }
                    }
                }
            }
            _ => (),
        }
    }
}
