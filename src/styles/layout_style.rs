use iced::widget::container;
use iced::{Background, Color, Theme};

pub fn header_style(theme: &Theme) -> container::Style {
    container::Style::default().background(Background::Color(Color::BLACK))
}