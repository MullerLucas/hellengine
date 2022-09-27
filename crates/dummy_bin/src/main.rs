fn main() {

    let win = hell_winit::WinitWindow::new("hell-app", 800, 600).expect("failed to create window");
    // TODO: error handling
    let app = hell_app::HellApp::new(&win).unwrap();

    win.main_loop(app);
}
