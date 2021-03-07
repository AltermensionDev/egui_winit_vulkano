use std::sync::Arc;

use egui::CtxRef;
use vulkano::{
    command_buffer::AutoCommandBuffer,
    device::Queue,
    framebuffer::{RenderPassAbstract, Subpass},
    image::ImageViewAccess,
};
use winit::{dpi::PhysicalSize, event::Event, window::Window};

use crate::{context::EguiContext, renderer::EguiVulkanoRenderer, utils::texture_from_file_bytes};

pub struct Gui {
    context: EguiContext,
    renderer: EguiVulkanoRenderer,
}

impl Gui {
    /// Creates new Egui to Vulkano integration by setting the necessary parameters
    /// This is to be called once we have access to vulkano_win's winit window surface
    /// and after render pass has been created
    /// - `size`: Size of the window as [PhysicalSize<u32>]
    /// - `scale_factor`: pointes per pixel, = `window.scale_factor()`
    /// - `gfx_queue`: Vulkano's [`Queue`]
    /// - `subpass`: Vulkano's subpass created from render pass, see examples
    /// - Render pass must have depth attachment and at least one color attachment
    pub fn new<R>(
        size: PhysicalSize<u32>,
        scale_factor: f64,
        gfx_queue: Arc<Queue>,
        subpass: Subpass<R>,
    ) -> Gui
    where
        R: RenderPassAbstract + Send + Sync + 'static,
    {
        assert!(subpass.has_depth());
        assert!(subpass.num_color_attachments() >= 1);
        // ToDo: Validate what ever is useful
        let context = EguiContext::new(size, scale_factor);
        let renderer = EguiVulkanoRenderer::new(gfx_queue.clone(), subpass);
        Gui { context, renderer }
    }

    /// Updates context state by winit event. Integration must have been initialized
    pub fn update<T>(&mut self, winit_event: &Event<T>) {
        self.context.handle_event(winit_event)
    }

    /// Sets Egui integration's UI layout. This must be called before draw
    /// Begins Egui frame
    pub fn immediate_ui(&mut self, layout_function: impl FnOnce(CtxRef)) {
        self.context.begin_frame();
        // Render Egui
        layout_function(self.context());
    }

    /// Renders ui & Updates cursor icon
    /// Finishes Egui frame
    pub fn draw(&mut self, window: &Window, framebuffer_dimensions: [u32; 2]) -> AutoCommandBuffer {
        // Get outputs of `immediate_ui`
        let (output, clipped_meshes) = self.context.end_frame();
        // Update cursor icon
        self.context.update_cursor_icon(window, output.cursor_icon);
        // Draw egui meshes
        let cb = self.renderer.draw(&mut self.context, clipped_meshes, framebuffer_dimensions);
        cb
    }

    /// Registers a user image from Vulkano image view to be used by egui
    pub fn register_user_image_view(
        &mut self,
        image: Arc<dyn ImageViewAccess + Send + Sync>,
    ) -> egui::TextureId {
        self.renderer.register_user_image(image)
    }

    /// Registers a user image to be used by egui
    /// - `image_file_bytes`: e.g. include_bytes!("./assets/tree.png")
    pub fn register_user_image(&mut self, image_file_bytes: &[u8]) -> egui::TextureId {
        let image = texture_from_file_bytes(self.renderer.queue(), image_file_bytes)
            .expect("Failed to create image");
        self.renderer.register_user_image(image)
    }

    /// Unregisters a user image
    pub fn unregister_user_image(&mut self, texture_id: egui::TextureId) {
        self.renderer.unregister_user_image(texture_id);
    }

    /// Access egui's context (which can be used to e.g. set fonts, visuals etc)
    pub fn context(&self) -> egui::CtxRef {
        self.context.context()
    }
}