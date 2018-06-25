use renderer::system::gbuffer::GBuffer;
use std::sync::Arc;
use vulkano::{command_buffer::{AutoCommandBufferBuilder, CommandBuffer, DynamicState},
              device::Queue,
              format::Format,
              framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
              image::{AttachmentImage, ImageAccess, ImageUsage, ImageViewAccess},
              pipeline::viewport::Viewport,
              sync::GpuFuture};

pub struct RenderSystem {
    queue: Arc<Queue>,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    gbuffer: GBuffer,
}

impl RenderSystem {
    pub fn new(
        queue: Arc<Queue>,
        render_pass: Arc<RenderPassAbstract + Send + Sync>,
        gbuffer: GBuffer,
    ) -> Self {
        Self {
            queue,
            render_pass,
            gbuffer,
        }
    }
    pub fn get_subpass(
        &self,
        index: u32,
    ) -> Option<Subpass<Arc<RenderPassAbstract + Send + Sync>>> {
        if index < self.render_pass.num_subpasses() as u32 {
            return Some(Subpass::from(self.render_pass.clone(), index).unwrap());
        }
        None
    }
    pub fn frame<F, I>(&mut self, before_future: F, final_image: I) -> Frame
    where
        F: GpuFuture + 'static,
        I: ImageAccess + ImageViewAccess + Clone + Send + Sync + 'static,
    {
        let img_dims = ImageAccess::dimensions(&final_image).width_height();
        if self.gbuffer.dims() != img_dims {
            self.gbuffer.rebuild_with_dims(self.queue.clone(), img_dims);
        }
        let framebuffer = Arc::new(
            Framebuffer::start(self.render_pass.clone())
                .add(final_image.clone())
                .unwrap()
                .add(self.gbuffer.diffuse.clone())
                .unwrap()
                .add(self.gbuffer.specular.clone())
                .unwrap()
                .add(self.gbuffer.normal.clone())
                .unwrap()
                .add(self.gbuffer.depth.clone())
                .unwrap()
                .build()
                .unwrap(),
        );
        let command_buffer = Some(
            AutoCommandBufferBuilder::primary_one_time_submit(
                self.queue.device().clone(),
                self.queue.family(),
            ).unwrap()
                .begin_render_pass(
                    framebuffer.clone(),
                    true,
                    vec![
                        [0.0, 0.0, 0.0, 0.0].into(),
                        0.0f32.into(),
                        [0.0, 0.0, 0.0, 0.0].into(),
                        [0.0, 0.0, 0.0, 0.0].into(),
                        1.0f32.into(),
                    ],
                )
                .unwrap(),
        );
        Frame {
            render_system: self,
            before_main_cb_future: Some(Box::new(before_future)),
            framebuffer,
            number_of_stages: self.render_pass.num_subpasses() as u8,
            stage: 0,
            command_buffer,
        }
    }
}

pub fn deffered_lighting_render_pass(
    queue: Arc<Queue>,
    final_output_format: Format,
) -> Arc<RenderPassAbstract + Send + Sync> {
    let render_pass = Arc::new(
        ordered_passes_renderpass!(queue.device().clone(),
            attachments: {
                // The image that will contain the final rendering (in this example the swapchain
                // image, but it could be another image).
                final_color: {
                    load: Clear,
                    store: Store,
                    format: final_output_format,
                    samples: 1,
                },
                // Will be bound to `self.diffuse_buffer`.
                diffuse: {
                    load: Clear,
                    store: DontCare,
                    format: Format::A2B10G10R10UnormPack32,
                    samples: 1,
                },
                specular: {
                    load: Clear,
                    store: DontCare,
                    format: Format::R16Unorm,
                    samples: 1,
                },
                // Will be bound to `self.normals_buffer`.
                normals: {
                    load: Clear,
                    store: DontCare,
                    format: Format::R16G16B16A16Sfloat,
                    samples: 1,
                },
                // Will be bound to `self.depth_buffer`.
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16Unorm,
                    samples: 1,
                }
            },
            passes: [
                // Write to the diffuse, normals and depth attachments.
                {
                    color: [diffuse, specular, normals],
                    depth_stencil: {depth},
                    input: []
                },
                // Apply lighting by reading these three attachments and writing to `final_color`.
                {
                    color: [final_color],
                    depth_stencil: {},
                    input: [diffuse, specular, normals, depth]
                }
            ]
        ).unwrap(),
    );
    return render_pass;
}

// Want to expose the command buffer at each stage
pub struct Frame<'a> {
    render_system: &'a RenderSystem,
    number_of_stages: u8,
    stage: u8,
    before_main_cb_future: Option<Box<GpuFuture>>,
    framebuffer: Arc<FramebufferAbstract + Send + Sync>,
    command_buffer: Option<AutoCommandBufferBuilder>,
}

// returns the RenderPass for the next subpass and the index of the current subpass
impl<'a> Frame<'a> {
    pub fn next_pass<'f>(&'f mut self) -> (Option<RenderPass<'f, 'a>>, u8) {
        match {
            let current = self.stage;
            self.stage += 1;
            current
        } {
            n if n == 0 => (Some(RenderPass::SubPass(Pass { frame: self })), n),
            n if n < self.number_of_stages => {
                self.command_buffer = Some(
                    self.command_buffer
                        .take()
                        .unwrap()
                        .next_subpass(true)
                        .unwrap(),
                );
                (Some(RenderPass::SubPass(Pass { frame: self })), n)
            }
            n if n == self.number_of_stages => {
                let command_buffer = self.command_buffer
                    .take()
                    .unwrap()
                    .end_render_pass()
                    .unwrap()
                    .build()
                    .unwrap();
                let after_main_cb = self.before_main_cb_future
                    .take()
                    .unwrap()
                    .then_execute(self.render_system.queue.clone(), command_buffer)
                    .unwrap();
                return (Some(RenderPass::Finished(Box::new(after_main_cb))), n);
            }
            n => (None, n),
        }
    }
}

pub enum RenderPass<'f, 's: 'f> {
    SubPass(Pass<'f, 's>),
    Finished(Box<GpuFuture>),
}

pub struct Pass<'f, 's: 'f> {
    frame: &'f mut Frame<'s>,
}

impl<'f, 's: 'f> Pass<'f, 's> {
    #[inline]
    pub fn execute<C>(&mut self, command_buffer: C)
    where
        C: CommandBuffer + Send + Sync + 'static,
    {
        unsafe {
            self.frame.command_buffer = Some(
                self.frame
                    .command_buffer
                    .take()
                    .unwrap()
                    .execute_commands(command_buffer)
                    .unwrap(),
            );
        }
    }
    pub fn viewport_dimensions(&self) -> [u32; 2] {
        let dims = self.frame.framebuffer.dimensions();
        [dims[0], dims[1]]
    }
    pub fn dynamic_state(&self) -> DynamicState {
        let dimensions = self.viewport_dimensions();
        DynamicState {
            viewports: Some(vec![Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0 .. 1.0
            }]),
            .. DynamicState::none()
        }
    }
}
