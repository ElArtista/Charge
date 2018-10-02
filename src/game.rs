use assets::*;
use gl;
use glutin::{
    dpi::*, ContextBuilder, ElementState, Event, EventsLoop, GlContext, GlProfile, GlRequest,
    GlWindow, VirtualKeyCode, WindowBuilder, WindowEvent,
};
use graphics::*;
use math::*;
use std::path::Path;

const WND_DIMENSIONS: (f32, f32) = (1280.0, 720.0);

const VERTEX_SHADER: &str = "\
#version 330 core
in vec3 vpos;
in vec2 vuv0;

out vec2 texcoord;
uniform mat4 mvp;

void main()
{
    texcoord = vuv0;
    gl_Position = mvp * vec4(vpos, 1.0);
}
";

const FRAGMENT_SHADER: &str = "\
#version 330 core
out vec4 color;
in vec2 texcoord;

uniform sampler2D tex;

void main()
{
    color = vec4(texture(tex, texcoord).rgb, 1.0);
}
";

pub struct Game {
    events_loop: EventsLoop,
    window: GlWindow,
    shdr: Shader,
    mesh: Mesh,
    tex: Texture,
}

impl Game {
    pub fn new() -> Game {
        // Event pump
        let events_loop = EventsLoop::new();

        // Plain window
        let window = WindowBuilder::new()
            .with_dimensions(LogicalSize::new(
                WND_DIMENSIONS.0 as f64,
                WND_DIMENSIONS.1 as f64,
            )).with_resizable(false);

        // Accelerated 3D context
        let context = ContextBuilder::new()
            .with_multisampling(4)
            .with_gl_profile(GlProfile::Compatibility)
            .with_gl_debug_flag(true)
            .with_gl(GlRequest::GlThenGles {
                opengl_version: (3, 3),
                opengles_version: (2, 0),
            });

        // Window with accelerated 3D context
        let gl_window = GlWindow::new(window, context, &events_loop).unwrap();

        // Make context current before calling gl function loader
        unsafe { gl_window.make_current().unwrap() };

        // Load OpenGL function pointers
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

        // Load sample shader
        let shdr = Shader::new(
            VERTEX_SHADER,
            None,
            FRAGMENT_SHADER,
            Some(&["vpos", "vnrm", "vuv0"]),
        );

        // Load sample 3D model
        let (vdata, num_verts, indcs) = Self::load_flattened_model("assets/spot/spot.obj").unwrap();

        // Load sample mesh
        let mesh = Mesh::from_data(
            &vdata,
            num_verts,
            Some(&indcs),
            vattr_flag(Vattr::Position) | vattr_flag(Vattr::UV0),
        );

        // Load sample image
        let img = Image::from_file(Path::new("assets/spot/spot.png")).unwrap();

        // Load sample texture
        let tex = Texture::from_image(&img);

        Game {
            events_loop: events_loop,
            window: gl_window,
            shdr: shdr,
            mesh: mesh,
            tex: tex,
        }
    }

    fn load_flattened_model(fpath: &str) -> Result<(Vec<f32>, usize, Vec<u32>), String> {
        let mut model = try!(Model::from_file(Path::new(fpath)));
        let mut vpos = Vec::new();
        let mut vuv0 = Vec::new();
        let mut indc = Vec::new();
        let mut nvrt = 0;
        for s in model.shapes.iter_mut() {
            vpos.append(&mut s.positions);
            vuv0.append(&mut s.texcoords);
            indc.append(&mut s.indices);
            nvrt += vpos.len() / 3;
        }
        let mut vdata = Vec::new();
        vdata.append(&mut vpos);
        vdata.append(&mut vuv0);
        Ok((vdata, nvrt, indc))
    }

    pub fn update(&mut self, _dt: f32) -> bool {
        let mut exit_flag = false;
        let wnd = &mut self.window;
        self.events_loop.poll_events(|event| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => exit_flag = true,
                WindowEvent::Resized(logical_size) => {
                    let dpi_factor = wnd.get_hidpi_factor();
                    wnd.resize(logical_size.to_physical(dpi_factor));
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Released {
                        if let Some(key) = input.virtual_keycode {
                            match key {
                                VirtualKeyCode::Escape => exit_flag = true,
                                _ => (),
                            }
                        }
                    }
                }
                _ => (),
            },
            _ => (),
        });
        exit_flag
    }

    pub fn render(&self, _interpolation: f32) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::DEPTH_TEST);
        }
        let proj = perspective(Deg(60.0), WND_DIMENSIONS.0 / WND_DIMENSIONS.1, 0.1, 100.0);
        let view = Matrix4::look_at(
            Point3::new(0.0, 0.0, -3.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );
        let modl = Matrix4::from_angle_y(Deg(26.0));
        let mvp = conv::array4x4(proj * view * modl);
        self.shdr.activate();
        self.shdr.set_uniform("mvp", &mvp);
        self.shdr.set_uniform("tex", 0);
        self.tex.bind(0);
        self.mesh.draw();
        self.window.swap_buffers().unwrap();
    }

    pub fn perf(&self, ms: f32, fps: f32) {
        let title = format!("[Msec: {:.2} / Fps: {:.2}]", ms, fps);
        self.window.set_title(title.as_str());
    }
}
