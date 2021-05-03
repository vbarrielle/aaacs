use iced::{button, Color};

pub struct Button;

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        button::Style {
            background: Color::from_rgb(0., 0.8, 0.8).into(),
            border_radius: 5.0,
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            background: Color::from_rgb(0., 1.0, 1.0).into(),
            text_color: Color::WHITE,
            border_radius: 5.0,
            ..button::Style::default()
        }
    }
}
