use std::sync::Arc;
use vulkano::{command_buffer::{AutoCommandBufferBuilder, CommandBuffer},
              device::Queue,
              format::Format,
              framebuffer::{Framebuffer, FramebufferAbstract,
                            RenderPassAbstract, Subpass},
              image::{AttachmentImage, ImageAccess, ImageUsage, ImageViewAccess},
              sync::GpuFuture};

pub struct GBuffer {
    pub diffuse: Arc<AttachmentImage>,
    pub specular: Arc<AttachmentImage>,
    pub normal: Arc<AttachmentImage>,
    pub depth: Arc<AttachmentImage>,
    pub builder: GBufferBuilder,
}

impl GBuffer {
    #[inline]
    pub fn dims(&self) -> [u32; 2] {
        ImageAccess::dimensions(&self.diffuse).width_height()
    }
    pub fn rebuild_with_dims(&mut self, queue: Arc<Queue>, dims: [u32; 2]) {
        *self = self.builder.build_with_dims(queue, dims);
    }
}
#[derive(Copy, Clone)]
pub struct GBufferBuilder {
    diffuse_usage: (ImageUsage, Format),
    specular_usage: (ImageUsage, Format),
    normals_usage: (ImageUsage, Format),
    depth_usage: (ImageUsage, Format),
}

impl GBufferBuilder {
    pub fn build_no_dims(&self, queue: Arc<Queue>) -> GBuffer {
        self.build_with_dims(queue, [1, 1])
    }
    #[inline]
    pub fn build_with_dims(&self, queue: Arc<Queue>, dimensions: [u32; 2]) -> GBuffer {
        GBuffer {
            diffuse: AttachmentImage::with_usage(
                queue.device().clone(),
                dimensions,
                self.diffuse_usage.1,
                self.diffuse_usage.0,
            ).unwrap(),
            specular: AttachmentImage::with_usage(
                queue.device().clone(),
                dimensions,
                self.specular_usage.1,
                self.specular_usage.0,
            ).unwrap(),
            normal: AttachmentImage::with_usage(
                queue.device().clone(),
                dimensions,
                self.normals_usage.1,
                self.normals_usage.0,
            ).unwrap(),
            depth: AttachmentImage::with_usage(
                queue.device().clone(),
                dimensions,
                self.depth_usage.1,
                self.depth_usage.0,
            ).unwrap(),
            builder: *self,
        }
    }
    pub fn new_default() -> Self {
        let atch_usage = ImageUsage {
            transient_attachment: true,
            input_attachment: true,
            ..ImageUsage::none()
        };
        Self {
            diffuse_usage: (atch_usage, Format::A2B10G10R10UnormPack32),
            specular_usage: (atch_usage, Format::R16Unorm),
            normals_usage: (atch_usage, Format::R16G16B16A16Sfloat),
            depth_usage: (atch_usage, Format::D16Unorm),
        }
    }
    pub fn set_diffuse_usage(&mut self, atch_usage: ImageUsage, format: Format) {
        self.diffuse_usage = (atch_usage, format);
    }
    pub fn set_specular_usage(&mut self, atch_usage: ImageUsage, format: Format) {
        self.specular_usage = (atch_usage, format);
    }
    pub fn set_normals_usage(&mut self, atch_usage: ImageUsage, format: Format) {
        self.normals_usage = (atch_usage, format);
    }
    pub fn set_depth_usage(&mut self, atch_usage: ImageUsage, format: Format) {
        self.depth_usage = (atch_usage, format);
    }
}