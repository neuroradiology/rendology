pub mod machine;
pub mod object;
pub mod camera;

use nalgebra as na;
use glium::{self, program, uniform};
use num_traits::ToPrimitive;

pub use object::{Object, Instance, InstanceParams};
pub use camera::Camera;

use object::ObjectBuffers;

pub struct Resources {
    object_buffers: Vec<ObjectBuffers>,
    program: glium::Program,
}

#[derive(Debug)]
pub enum CreationError {
    ObjectCreationError(object::CreationError),
    ProgramChooserCreationError(glium::program::ProgramChooserCreationError),
}

impl From<object::CreationError> for CreationError {
    fn from(err: object::CreationError) -> CreationError {
        CreationError::ObjectCreationError(err)
    }
}

impl From<glium::program::ProgramChooserCreationError> for CreationError {
    fn from(err: glium::program::ProgramChooserCreationError) -> CreationError {
        CreationError::ProgramChooserCreationError(err)
    }
}

impl Resources {
    pub fn create<F: glium::backend::Facade>(
        facade: &F,
    ) -> Result<Resources, CreationError> {
        // Unfortunately, it doesn't seem easy to use enum_map here,
        // since we need to check for errors in creating buffers
        let mut object_buffers = Vec::new();

        for i in 0 .. Object::NumTypes as u32 {
            // Safe to unwrap here, since we iterate within the range
            let object: Object = num_traits::FromPrimitive::from_u32(i).unwrap();

            object_buffers.push(object.create_buffers(facade)?);
        }

        let program = program!(facade,
            140 => {
                vertex: "
                    #version 140

                    uniform mat4 mat_model;
                    uniform mat4 mat_view;
                    uniform mat4 mat_projection;

                    uniform vec4 color;

                    in vec3 position;
                    in vec3 normal;
                    out vec3 v_normal;
                    out vec4 v_color;

                    void main() {
                        gl_Position = mat_projection
                            * mat_view
                            * mat_model
                            * vec4(position, 1.0);

                        v_normal = normal;
                        v_color = color;

                    }
                ",

                fragment: "
                    #version 140

                    uniform float M_PI = 3.1415926535;

                    uniform float t;

                    in vec3 v_normal;
                    in vec4 v_color;
                    out vec4 f_color;

                    void main() {

                        vec3 lightdirA = vec3(sin(t/6.0), cos(t/6.0), 0.0); 
                        vec3 lightdirB = vec3(sin(t/6.0 + M_PI/2.0), cos(t/6.0 + M_PI/2.0), 0.0); 
                        float ambient = 0.2;
                        float diffuseA = clamp(dot(lightdirA, v_normal), 0.0, 1.0);
                        float diffuseB = clamp(dot(lightdirB, v_normal), 0.0, 1.0);
                        f_color = (ambient + diffuseA + diffuseB) * v_color;
                    }
                "
            },
        )?;

        Ok(Resources {
            object_buffers,
            program
        })
    }

    fn get_object_buffers(&self, object: Object) -> &ObjectBuffers {
        // Safe to unwrap array access here, since we have initialized buffers
        // for all objects
        &self.object_buffers[object.to_usize().unwrap()]
    }
}

pub struct Context {
    pub camera: camera::Camera,
    pub elapsed_time_secs: f32,
}

#[derive(Default)]
pub struct RenderList {
    instances: Vec<Instance>,
}

impl RenderList {
    pub fn new() -> RenderList {
        Default::default()
    }

    pub fn add_instance(&mut self, instance: &Instance) {
        self.instances.push(instance.clone());
    }

    pub fn add(&mut self, object: Object, params: &InstanceParams) {
        self.add_instance(&Instance { object, params: params.clone() });
    }

    pub fn render<S: glium::Surface>(
        &self,
        resources: &Resources,
        context: &Context,
        target: &mut S,
    ) -> Result<(), glium::DrawError> {
        // TODO: Could sort by object here to reduce state switching for large
        // numbers of objects.

        let mat_projection: [[f32; 4]; 4] = context.camera.projection().into();
        let mat_view: [[f32; 4]; 4] = context.camera.view().into();

        let params = glium::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. Default::default()
        };

        //let params = Default::default();

        for instance in &self.instances {
            let buffers = resources.get_object_buffers(instance.object);

            let mat_model: [[f32; 4]; 4] = instance.params.transform.into();
            let color: [f32; 4] = instance.params.color.into();
            let uniforms = uniform! {
                mat_model: mat_model, 
                mat_view: mat_view,
                mat_projection: mat_projection,
                color: color,
                t: context.elapsed_time_secs,
            };

            match &buffers.index_buffer {
                object::IndexBuffer::IndexBuffer(buffer) => {
                    target.draw(
                        &buffers.vertex_buffer,
                        buffer,
                        &resources.program,
                        &uniforms,
                        &params,
                    )?;
                }
                object::IndexBuffer::NoIndices(buffer) => {
                    target.draw(
                        &buffers.vertex_buffer,
                        buffer,
                        &resources.program,
                        &uniforms,
                        &params,
                    )?;
                }
            }
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        self.instances.clear();
    }
}
