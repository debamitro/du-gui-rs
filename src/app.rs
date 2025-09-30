use iced::widget::{button, column, text};
use iced::{Alignment, Application, Command, Element, Theme};

#[derive(Debug, Clone)]
pub enum Message {
    ShowAbout,
    BackToMain,
}

#[derive(Default)]
pub struct AppState {
    mode: Mode,
}

#[derive(Default)]
pub enum Mode {
    #[default]
    Main,
    About,
}

impl Application for AppState {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("du-gui-rs")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::ShowAbout => {
                self.mode = Mode::About;
            }
            Message::BackToMain => {
                self.mode = Mode::Main;
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        match self.mode {
            Mode::Main => column![
                text("Welcome to du-gui-rs").size(50),
                button("About").on_press(Message::ShowAbout),
            ]
            .padding(20)
            .align_items(Alignment::Center)
            .into(),
            Mode::About => column![
                text("About du-gui-rs").size(50),
                text("Version 1.0.0").size(30),
                text("This application tries to find out which files are taking space on your hard disk.").size(20),
                button("Back").on_press(Message::BackToMain),
            ]
            .padding(20)
            .align_items(Alignment::Center)
            .into(),
        }
    }
}
