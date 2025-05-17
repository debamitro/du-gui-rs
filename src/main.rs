use gpui::*;

mod about;
mod app;
use about::AboutWindow;
use app::AppState;

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.bind_keys(vec![KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(quit);
        cx.on_action(about);
        cx.set_menus(vec![Menu {
            name: "set_menus".into(),
            items: vec![
                MenuItem::action("About", About),
                MenuItem::action("Quit", Quit),
            ],
        }]);
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_cx| AppState {}))
            .unwrap();
    });
}

actions!(set_menus, [Quit, About]);

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

fn about(_: &About, cx: &mut App) {
    cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| AboutWindow {}))
        .unwrap();
}
