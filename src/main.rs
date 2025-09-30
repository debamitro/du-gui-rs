mod app;

use app::AppState;
use iced::Application;

fn main() -> iced::Result {
    AppState::run(iced::Settings::default())
}
