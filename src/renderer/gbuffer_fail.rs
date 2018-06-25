use std::ops::{Index, IndexMut};
use std::sync::Arc;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::framebuffer::{AttachmentsList, FramebufferBuilder, RenderPassAbstract};
use vulkano::image::{AttachmentImage, ImageAccess, ImageUsage};

pub trait GBuffer: Index<usize, Output = Arc<AttachmentImage>> + IndexMut<usize> {
    fn buffer_usage(&self, index: usize) -> ImageUsage;
    fn buffer_format(&self, index: usize) -> Format;
    fn queue(&self) -> Arc<Queue>;
    fn dims(&self) -> [u32; 2];
    fn number_of_buffers(&self) -> usize;
    fn rebuild_with_dims(&mut self, dims: [u32; 2]) {
        for i in 0..self.number_of_buffers() {
            self[i] = AttachmentImage::with_usage(
                self.queue().device().clone(),
                dims,
                self.buffer_format(i),
                self.buffer_usage(i),
            ).unwrap();
        }
    }
}

pub struct RenderBuffer {
    pub buffer: Arc<AttachmentImage>,
    pub usage: ImageUsage,
}

impl RenderBuffer {
    pub fn new(
        queue: Arc<Queue>,
        dimensions: [u32; 2],
        format: Format,
        image_usage: ImageUsage,
    ) -> Self {
        RenderBuffer {
            buffer: AttachmentImage::with_usage(
                queue.device().clone(),
                dimensions,
                format,
                image_usage,
            ).unwrap(),
            usage: image_usage,
        }
    }
    pub fn rebuild_with_dims(&mut self, queue: Arc<Queue>, dimensions: [u32; 2]) {
        *self = RenderBuffer::new(queue, dimensions, self.buffer.format(), self.usage);
    }
    pub fn format(&self) -> Format {
        self.buffer.format()
    }
    pub fn dimensions(&self) -> [u32; 2]{
         ImageAccess::dimensions(&self.buffer).width_height()
    }
}

pub struct GBufferContainer {
    queue: Arc<Queue>,
    buffers: Vec<RenderBuffer>,
}

impl Index<usize> for GBufferContainer {
    type Output =  Arc<AttachmentImage>;    
    fn index(&self, index: usize) -> &Arc<AttachmentImage> {
        &self.buffers[index].buffer
    }

} 
impl IndexMut<usize> for GBufferContainer {
    fn index_mut(&mut self, index: usize) -> &mut Arc<AttachmentImage> {
        &mut self.buffers[index].buffer
    }

} 
impl GBuffer for GBufferContainer {
    fn buffer_usage(&self, index: usize) -> ImageUsage {
        self.buffers[index].usage
    }
    fn buffer_format(&self, index: usize) -> Format {
        self.buffers[index].format()
    }
    fn queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }
    fn dims(&self) -> [u32; 2] {
        self.buffers[0].dimensions()
    }
    fn number_of_buffers(&self) -> usize {
        self.buffers.len()
    }
}

