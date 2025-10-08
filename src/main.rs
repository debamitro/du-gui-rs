mod app;

use app::AppState;
use iced::Theme;

fn main() -> iced::Result {
    iced::application("FindBigFolders", AppState::update, AppState::view)
        .subscription(AppState::subscription)
        .theme(theme)
        .run_with(AppState::new)
}

fn theme(state: &AppState) -> Theme {
    Theme::CatppuccinFrappe
}