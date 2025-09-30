use iced::widget::{button, column, row, text};
use iced::{Alignment, Application, Command, Element, Theme};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum Message {
    ShowAbout,
    BackToMain,
    CurrentUser,
    AllUsers,
    Scanned(Vec<FileEntry>),
}

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub file: String,
    pub size: String,
}

#[derive(Default)]
pub struct AppState {
    mode: Mode,
    entries: Vec<FileEntry>,
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
        let entries = vec![
            FileEntry {
                file: "example.txt".to_string(),
                size: "1 KB".to_string(),
            },
            FileEntry {
                file: "another.file".to_string(),
                size: "2 MB".to_string(),
            },
        ];
        (Self { mode: Mode::Main, entries }, Command::none())
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
            Message::CurrentUser => {
                return Command::perform(scan_home(), Message::Scanned);
            }
            Message::AllUsers => {
                // TODO: implement scanning all users
            }
            Message::Scanned(entries) => {
                self.entries = entries;
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        match self.mode {
            Mode::Main => {
                let mut table_column = column![]
                    .push(row![text("File").width(200), text("Size").width(100)]);
                for entry in &self.entries {
                    table_column = table_column.push(
                        row![text(&entry.file).width(200), text(&entry.size).width(100)]
                    );
                }
                column![
                    text("Welcome to du-gui-rs").size(50),
                    button("About").on_press(Message::ShowAbout),
                    button("Current User").on_press(Message::CurrentUser),
                    button("All Users").on_press(Message::AllUsers),
                    table_column,
                ]
                .padding(20)
                .align_items(Alignment::Center)
                .into()
            }
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

fn calculate_dir_size(path: &Path) -> u64 {
    use std::fs;

    let mut size = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    size += metadata.len();
                }
            } else if entry_path.is_dir() {
                size += calculate_dir_size(&entry_path);
            }
        }
    }
    size
}

fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size_f = size as f64;
    let mut unit_index = 0;
    while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_index += 1;
    }
    format!("{:.1} {}", size_f, UNITS[unit_index])
}

fn parse_size(size_str: &str) -> u64 {
    let parts: Vec<&str> = size_str.split_whitespace().collect();
    if parts.len() == 2 {
        if let Ok(num) = parts[0].parse::<f64>() {
            match parts[1] {
                "B" => (num) as u64,
                "KB" => (num * 1024.0) as u64,
                "MB" => (num * 1024.0 * 1024.0) as u64,
                "GB" => (num * 1024.0 * 1024.0 * 1024.0) as u64,
                "TB" => (num * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64,
                _ => 0,
            }
        } else {
            0
        }
    } else {
        0
    }
}

async fn scan_home() -> Vec<FileEntry> {
    tokio::task::spawn_blocking(|| {
        use std::fs;

        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let mut entries = Vec::new();

        if let Ok(dir_entries) = fs::read_dir(&home) {
            for entry in dir_entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let size = calculate_dir_size(&path);
                    let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let size_str = format_size(size);
                    entries.push(FileEntry { file: file_name, size: size_str });
                }
            }
        }

        entries.sort_by(|a, b| {
            // Simple sort by size, parse the string roughly
            let a_size = parse_size(&a.size);
            let b_size = parse_size(&b.size);
            b_size.cmp(&a_size)
        });
        entries
    }).await.unwrap_or_default()
}
