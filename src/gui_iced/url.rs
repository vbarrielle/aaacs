use dodrio::bumpalo;
use iced_web::{Bus, Css, Element, Widget};

pub struct UrlA {
    pub text: String,
    pub href: String,
    pub download: String,
}

impl<Message> Widget<Message> for UrlA {
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        _bus: &Bus<Message>,
        _style_sheet: &mut Css<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;
        let download = bumpalo::format!(in bump, "{}", self.download);
        let href = bumpalo::format!(in bump, "{}", self.href);
        let text_ = bumpalo::format!(in bump, "{}", self.text);
        a(bump)
            .attr("download", download.into_bump_str())
            .attr("href", href.into_bump_str())
            .children([text(text_.into_bump_str())])
            .finish()
    }
}

impl<'a, Message> From<UrlA> for Element<'a, Message> {
    fn from(url: UrlA) -> Element<'a, Message> {
        Element::new(url)
    }
}
