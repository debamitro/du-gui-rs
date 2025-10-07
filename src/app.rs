use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Application, Command, Element, Length, Renderer, Theme};
use iced_table::table;
use iced::futures;
use iced::subscription::{self, Subscription};
use std::path::{Path, PathBuf};
use futures::channel::mpsc;
use std::collections::HashMap;
use iced::futures::SinkExt;
use std::process;

#[derive(Debug, Clone)]
pub enum Message {
    ShowAbout,
    BackToMain,
    SearchReady(mpsc::Sender<Message>, mpsc::Sender<Message>),
    CurrentUser,
    AllUsers,
    Scanned(FileEntry),
    Done
    Stop,
    SyncHeader(scrollable::AbsoluteOffset),
    OpenFolder(String),
}

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub file: String,
    pub size: u64,
}

pub struct AppState {
    mode: Mode,
    entries: Vec<FileEntry>,
    entries_visible: u64,
    sort_cutoff: u64,
    search_tx: Option<mpsc::Sender<Message>>,
    stop_tx: Option<mpsc::Sender<Message>>,
    columns: Vec<FileColumn>,
    header: scrollable::Id,
    body: scrollable::Id,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mode: Mode::default(),
            entries: Vec::new(),
            entries_visible: 20,
            sort_cutoff: 1000,
            search_tx: None,
            stop_tx: None,
            columns: vec![
                FileColumn::new(FileColumnKind::File),
                FileColumn::new(FileColumnKind::Size),
            ],
            header: scrollable::Id::unique(),
            body: scrollable::Id::unique(),
        }
    }
}

#[derive(Default)]
pub enum Mode {
    #[default]
    Main,
    About,
}

struct FileColumn {
    kind: FileColumnKind,
    width: f32,
}

impl FileColumn {
    fn new(kind: FileColumnKind) -> Self {
        let width = match kind {
            FileColumnKind::File => 500.0,
            FileColumnKind::Size => 100.0,
        };

        Self { kind, width }
    }
}

enum FileColumnKind {
    File,
    Size,
}

impl<'a> table::Column<'a, Message, Theme, Renderer> for FileColumn {
    type Row = FileEntry;

    fn header(&'a self, _col_index: usize) -> Element<'a, Message> {
        let content = match self.kind {
            FileColumnKind::File => "Folder",
            FileColumnKind::Size => "Size",
        };

        container(text(content)).center_y().into()
    }

    fn cell(&'a self, _col_index: usize, _row_index: usize, row: &'a FileEntry) -> Element<'a, Message> {
        let content: Element<_> = match self.kind {
            FileColumnKind::File => button(text(&row.file)).on_press(Message::OpenFolder(row.file.clone())).into(),
            FileColumnKind::Size => text(format_size(row.size)).into(),
        };

        container(content).width(Length::Fill).center_y().into()
    }

    fn width(&self) -> f32 {
        self.width
    }

    fn resize_offset(&self) -> Option<f32> {
        None
    }
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
        String::from("Disk Usage")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::ShowAbout => {
                self.mode = Mode::About;
            }
            Message::BackToMain => {
                self.mode = Mode::Main;
            }
            Message::SearchReady(tx1, tx2) => {
                self.search_tx = Some(tx1);
                self.stop_tx = Some(tx2);
            }
            Message::CurrentUser =>  {
                self.entries.clear();
                if let Some(tx) = &mut self.search_tx {
                    let _ = tx.try_send(Message::CurrentUser);
                }
            }
            Message::AllUsers => {
                self.entries.clear();
                if let Some(tx) = &mut self.search_tx {
                    let _ = tx.try_send(Message::AllUsers);
                }
            }
            Message::Scanned(entry) => {
                self.entries.push(entry);
                if self.entries.len() % self.sort_cutoff == 0 {
                    self.entries.sort_by(|a, b| b.size.cmp(&a.size));
                }
            }
            Message::Stop => {
                if let Some(tx) = &mut self.stop_tx {
                    let _ = tx.try_send(Message::Stop);
                }
            }
            Message::SyncHeader(_) => {
                // Handle header sync if needed
            }
            Message::OpenFolder(path) => {
                let _ = std::process::Command::new("open").arg(&path).spawn();
            }
            Message::Done => {
                self.entries.sort_by(|a, b| b.size.cmp(&a.size));
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
                    let (stop_tx, mut stop_rx) = mpsc::channel(1);
                    let _ = output.send(Message::SearchReady(cmd_tx, stop_tx)).await;

                    loop {
                        let msg = cmd_rx.try_next();
                        if let Ok(Some(Message::CurrentUser)) = msg {
                            if let Some(dir) = dirs::home_dir() {
                                scan_dirs(&dir, &mut output, &mut stop_rx).await;
                                let _ = output.send(Message::Done).await;
                            }
                        }
                        else if let Ok(Some(Message::AllUsers)) = msg {
                            if let Some(dir) = dirs::home_dir() {
                                if let Some(parent_dir) = dir.parent() {
                                    scan_dirs(&parent_dir, &mut output, &mut stop_rx).await;
                                    let _ = output.send(Message::Done).await;
                                }
                            }
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
                let file_table = table(
                    self.header.clone(),
                    self.body.clone(),
                    &self.columns,
                    &self.entries[..self.entries.len().min(20)],
                    Message::SyncHeader,
                );
                column![
                    text("Disk Usage").size(50),
                    button("About").on_press(Message::ShowAbout),
                    row![
                        button("Current User").on_press(Message::CurrentUser),
                        button("All Users").on_press(Message::AllUsers),
                        button("Stop").on_press(Message::Stop),
                    ]
                    .spacing(5)
                    .padding([0, 10, 0, 0]),
                    file_table,
                ]
                .padding(20)
                .spacing(5)
                .width(Length::Fill)
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

async fn calculate_dir_size(path: &Path, tx: &mut mpsc::Sender<Message>, stop_rx: &mut mpsc::Receiver<Message>) {
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
                        if p.is_symlink() {
                            continue;
                        }

                        if p.is_file() {
                            if let Ok(meta) = entry.metadata() {
                                size += meta.len();
                            }
                        } else if p.is_dir() {
                            if let Some(s) = sizes.get(&p) {
                                size += *s;
                            }
                            sizes.remove(&p);
                        }
                    }
                }
                sizes.insert(item.path.clone(), size);
                let _ = tx.send(Message::Scanned(FileEntry { file: item.path.to_str().unwrap_or_default().to_string(), size: size })).await;
                if let Ok(Some(Message::Stop)) = stop_rx.try_next() {
                    return;
                }
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

async fn scan_dirs(start_dir: &Path, tx: &mut mpsc::Sender<Message>, stop_rx: &mut mpsc::Receiver<Message>) {
    use std::fs;

    if let Ok(dir_entries) = fs::read_dir(&start_dir) {
        for entry in dir_entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                calculate_dir_size(&path, tx, stop_rx).await;
            }
        }
    }
}
