fn main() {
    let win = hell_winit::WinitWindow::new("hell-app", 800, 600)
        .expect("failed to create window");

    win.main_loop();
}
