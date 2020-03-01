use dodrio::bumpalo;
use iced_web::{style, Bus, Element, Widget};

pub struct FileInput {
    pub accept: String,
    pub id: String,
}

impl<Message> Widget<Message> for FileInput {
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        _bus: &Bus<Message>,
        _style_sheet: &mut style::Sheet<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;
        let type_ = bumpalo::format!(in bump, "{}", "file");
        let id = bumpalo::format!(in bump, "{}", self.id);
        let accept = bumpalo::format!(in bump, "{}", self.accept);
        input(bump)
            .attr("type", type_.into_bump_str())
            .attr("id", id.into_bump_str())
            .attr("accept", accept.into_bump_str())
            .finish()
    }
}

impl<'a, Message> From<FileInput> for Element<'a, Message> {
    fn from(input: FileInput) -> Element<'a, Message> {
        Element::new(input)
    }
}
