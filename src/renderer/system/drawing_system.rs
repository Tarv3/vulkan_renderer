use std::sync::Arc;
use vulkano::{command_buffer::pool::standard::StandardCommandPoolBuilder,
              command_buffer::{AutoCommandBufferBuilder},
              device::Queue,
              framebuffer::{RenderPassAbstract, Subpass},
              pipeline::{GraphicsPipelineAbstract, GraphicsPipeline}};

pub struct DrawSystem {
    queue: Arc<Queue>,
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
}

impl DrawSystem {
    pub fn new(queue: Arc<Queue>, pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>) -> Self {
        Self {
            queue,
            pipeline
        }
    }
    pub fn command_buffer_builder(&self) -> AutoCommandBufferBuilder<StandardCommandPoolBuilder> {
        AutoCommandBufferBuilder::secondary_graphics(
            self.queue.device().clone(),
            self.queue.family(),
            self.pipeline.clone().subpass(),
        ).unwrap()
    }
    pub fn new_geometry_draw<R>(queue: Arc<Queue>, subpass: Subpass<R>) -> Self 
    where R: RenderPassAbstract + Send + Sync + 'static
    {
        let vs = vs::Shader::load(queue.device().clone()).expect("Failed to load vertex shader");
        let fs = fs::Shader::load(queue.device().clone()).expect("Failed to load fragment shader");
        let pipeline = Arc::new(GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex>()
        .vertex_shader(vs.main_entry_point(), ())
        .triangle_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs.main_entry_point(), ())
        .depth_stencil_simple_depth()
        .render_pass(subpass)
        .build(queue.device().clone())
        .unwrap()) as Arc<_>;

        Self {
            queue,
            pipeline
        }
    }
}

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    colour: [f32; 3],
    specular: f32,
}
impl_vertex!(Vertex, position, normal, colour, specular);

mod vs {
    #[derive(VulkanoShader)]
    #[allow(dead_code)]
    #[ty = "vertex"]
    #[src = "
#version 450
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec3 colour;
layout(location = 3) in float specular;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec3 v_colour;
layout(location = 2) out float v_specular;
void main() {
    v_colour = colour;
    v_normal = normal;
    v_specular = specular;
}
"]
    struct Dummy;
}

mod fs {
    #[derive(VulkanoShader)]
    #[allow(dead_code)]
    #[ty = "fragment"]
    #[src = "
#version 450
layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec3 v_colour;
layout(location = 2) in float v_specular;


layout(location = 0) out vec3 f_colour;
layout(location = 1) out float f_specular;
layout(location = 2) out vec3 f_normals;
void main() {
    f_colour = v_colour;
    f_specular = v_specular;
    f_normals = v_normal; 
}
"]
    struct Dummy;
}