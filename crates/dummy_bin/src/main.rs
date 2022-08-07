fn main() {

    let win = hell_winit::WinitWindow::new("hell-app", 800, 600).expect("failed to create window");
    let app = hell_app::HellApp::new(&win);

    win.main_loop(app);
}
