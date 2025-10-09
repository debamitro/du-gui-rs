use iced::widget::button;
use iced::Theme;
use iced::Border;

pub fn action_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::primary(theme, status);
    style.border = Border::default().rounded(15);
    style
}

pub fn stop_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::danger(theme, status);
    style.border = Border::default().rounded(35);
    style
}