use hell_renderer::vulkan::vulkan_core::VulkanCore;

fn main() {
    let win = hell_winit::WinitWindow::new("hell_app", 800, 600)
        .expect("failed to create window");


    let _vulkan_core = VulkanCore::new(win.create_surface_info());

    win.main_loop(update);
}

fn update(delta_time: f32) {
    // println!("update: {}", delta_time);
}
