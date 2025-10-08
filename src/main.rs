mod app;

use app::AppState;

fn main() -> iced::Result {
    iced::application("FindBigFolders", AppState::update, AppState::view)
        .subscription(AppState::subscription)
        .run_with(AppState::new)
}
