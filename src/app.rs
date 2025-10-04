use iced::widget::{button, column, row, text};
use iced::{Alignment, Application, Command, Element, Theme};
use iced::futures;
use iced::subscription::{self, Subscription};
use std::path::{Path, PathBuf};
use futures::channel::mpsc;
use std::collections::HashMap;
use iced::futures::SinkExt;

#[derive(Debug, Clone)]
pub enum Message {
    ShowAbout,
    BackToMain,
    SearchReady(mpsc::Sender<Message>),
    CurrentUser,
    AllUsers,
    Scanned(FileEntry),
    Stop,
}

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub file: String,
    pub size: u64,
}

#[derive(Default)]
pub struct AppState {
    mode: Mode,
    entries: Vec<FileEntry>,
    search_tx: Option<mpsc::Sender<Message>>,
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
        ];
        (Self { mode: Mode::Main, entries, search_tx: None }, Command::none())
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
            Message::SearchReady(tx) => {
                self.search_tx = Some(tx);
            }
            Message::CurrentUser =>  {
                self.entries.clear();
                if let Some(tx) = &mut self.search_tx {
                    println!("Sending CurrentUser");
                    let _ = tx.try_send(Message::CurrentUser);
                    println!("Sent CurrentUser");
                }
            }
            Message::AllUsers => {
                // TODO: implement scanning all users
            }
            Message::Scanned(entry) => {
                println!("Scanned: {} {}", entry.file, entry.size);
                self.entries.push(entry);
                self.entries.sort_by(|a, b| b.size.cmp(&a.size));
            }
            Message::Stop => {
            }
        }
        Command::none()
    }
    fn subscription(&self) -> Subscription<Self::Message> {
            subscription::channel(
                std::any::TypeId::of::<Message>(),
                100,
                |mut output| async move {
                    let (cmd_tx, mut cmd_rx) = mpsc::channel(10);
                    println!("Sending SearchReady");
                    let _ = output.send(Message::SearchReady(cmd_tx)).await;
                    println!("Sent SearchReady");

                    loop {
                        println!("Waiting for message");
                        let msg = cmd_rx.try_next();
                        println!("Received message: {:?}", msg);
                        if let Ok(Some(Message::CurrentUser)) = msg {
                            scan_home(&mut output).await;
                            let _ = output.send(Message::Stop).await;
                        }
                        else if let Err(_) = msg {
                            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                        }
                    }
                }
            )
    }
    fn view(&self) -> Element<Self::Message> {
        match self.mode {
            Mode::Main => {
                let mut table_column = column![]
                    .push(row![text("File").width(200), text("Size").width(100)]);
                for entry in &self.entries {
                    table_column = table_column.push(
                        row![text(&entry.file).width(200), text(format_size(entry.size)).width(100)]
                    );
                }
                column![
                    text("Welcome to du-gui-rs").size(50),
                    button("About").on_press(Message::ShowAbout),
                    button("Current User").on_press(Message::CurrentUser),
                    button("All Users").on_press(Message::AllUsers),
                    button("Stop").on_press(Message::Stop),
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

async fn calculate_dir_size(path: &Path, tx: &mut mpsc::Sender<Message>) {
    use std::fs;

    #[derive(Clone)]
    enum State {
        Visiting,
        Visited,
    }

    #[derive(Clone)]
    struct Item {
        path: PathBuf,
        state: State,
    }

    let mut stack = vec![Item { path: path.to_path_buf(), state: State::Visiting }];
    let mut sizes: HashMap<PathBuf, u64> = HashMap::new();

    while let Some(mut item) = stack.pop() {
        match item.state {
            State::Visiting => {
                if item.path.is_file() {
                    let size = item.path.metadata().unwrap().len();
                    sizes.insert(item.path.clone(), size);
                } else {
                    item.state = State::Visited;
                    stack.push(item.clone());
                    if let Ok(entries) = fs::read_dir(&item.path) {
                        for entry in entries.flatten() {
                            let p = entry.path();
                            stack.push(Item { path: p, state: State::Visiting });
                        }
                    }
                }
            }
            State::Visited => {
                let mut size = 0;
                if let Ok(entries) = fs::read_dir(&item.path) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_file() {
                            if let Ok(meta) = entry.metadata() {
                                size += meta.len();
                            }
                        } else if p.is_dir() {
                            if let Some(s) = sizes.get(&p) {
                                size += *s;
                            }
                        }
                    }
                }
                sizes.insert(item.path.clone(), size);
                println!("Sending Scanned for {} {}", item.path.to_str().unwrap_or_default(), format_size(size));
                let _ = tx.send(Message::Scanned(FileEntry { file: item.path.to_str().unwrap_or_default().to_string(), size: size })).await;
                println!("Sent Scanned");
            }
        }
    }
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

async fn scan_home(tx: &mut mpsc::Sender<Message>) {
    use std::fs;

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

    if let Ok(dir_entries) = fs::read_dir(&home) {
        for entry in dir_entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                calculate_dir_size(&path, tx).await;
            }
        }
    }
}
