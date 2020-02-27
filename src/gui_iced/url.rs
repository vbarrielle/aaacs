
use iced_web::{Widget, style, Bus};
use dodrio::bumpalo;

struct UrlA {
    text: String,
    href: String,
    download: String,
}

impl<Message> Widget<Message> for UrlA {
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        _bus: &Bus<Message>,
        style_sheet: &mut style::Sheet<'b>,
    ) -> dodrio::Node<'b>
    {
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
