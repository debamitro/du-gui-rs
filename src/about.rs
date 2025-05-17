use gpui::*;

pub struct AboutWindow {}

impl Render for AboutWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let about_text =
            "This application tries to find out which files are taking space on your hard disk.";
        div()
            .flex()
            .flex_col()
            .bg(rgb(0x2e7d32))
            .size_full()
            .justify_center()
            .items_center()
            .text_xl()
            .text_color(rgb(0xffffff))
            .children([
                div().child("About du-gui-rs"),
                div().child("Version 1.0.0"),
                div().child(about_text),
            ])
    }
}
