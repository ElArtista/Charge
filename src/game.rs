use assets::*;
use gl;
use glutin::{
    dpi::*, Api, ContextBuilder, ElementState, Event, EventsLoop, GlContext, GlProfile, GlRequest,
    GlWindow, VirtualKeyCode, WindowBuilder, WindowEvent,
};
use graphics::*;
use math::*;
use std::path::Path;
use std::time::Instant;

const WND_DIMENSIONS: (f32, f32) = (1280.0, 720.0);

const VERTEX_SHADER: &str = "\
#version 300 es
in vec3 vpos;
in vec3 vnrm;
in vec2 vuv0;

out vec2 texcoord;
out vec3 normal;
out vec3 ws_pos;

uniform mat4 model;
uniform mat4 mvp;
uniform mat3 nmm;

void main()
{
    texcoord = vuv0;
    ws_pos = (model * vec4(vpos, 1.0)).xyz;
    normal = nmm * vnrm;
    gl_Position = mvp * vec4(vpos, 1.0);
}
";

const FRAGMENT_SHADER: &str = "\
#version 300 es
precision mediump float;
out vec4 fcolor;

in vec2 texcoord;
in vec3 normal;
in vec3 ws_pos;

uniform sampler2D tex;
uniform vec3 light_pos;

const vec3 light_color = vec3(1.0);

void main()
{
    vec3 base_color = texture(tex, texcoord).rgb;
    vec3 N = normalize(normal);
    vec3 L = normalize(light_pos - ws_pos);
    float kD = max(dot(N, L), 0.0);
    vec3 color = kD * base_color * light_color;
    fcolor = vec4(color, 1.0);
}
";

struct Timer {
    start: Instant,
}

impl Timer {
    fn new() -> Self {
        Timer {
            start: Instant::now(),
        }
    }

    fn elapsed_msec(&self) -> f32 {
        let now = Instant::now();
        let dur = now.duration_since(self.start);
        let elapsed = dur.as_secs() as f64 * 1000.0 + dur.subsec_nanos() as f64 / 1.0e6;
        elapsed as f32
    }
}

pub struct Game {
    events_loop: EventsLoop,
    window: GlWindow,
    shdr: Shader,
    mesh: Mesh,
    tex: Texture,
    timer: Timer,
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
            .with_gl(GlRequest::Specific(Api::OpenGlEs, (3, 0)));

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
        let (vdata, num_verts, indcs) = Self::load_flattened_model("spot/spot.obj").unwrap();

        // Load sample mesh
        let mesh = Mesh::from_data(
            &vdata,
            num_verts,
            Some(&indcs),
            vattr_flag(Vattr::Position) | vattr_flag(Vattr::Normal) | vattr_flag(Vattr::UV0),
        );

        // Load sample image
        let img_data = load(Path::new("spot/spot.png")).unwrap();
        let img = Image::from_buf(img_data).unwrap();

        // Load sample texture
        let tex = Texture::from_image(&img);

        Game {
            events_loop: events_loop,
            window: gl_window,
            shdr: shdr,
            mesh: mesh,
            tex: tex,
            timer: Timer::new(),
        }
    }

    fn load_flattened_model(fpath: &str) -> Result<(Vec<f32>, usize, Vec<u32>), String> {
        let mut mdl_data = try!(load(Path::new(fpath)));
        let mut model = try!(Model::from_buf(&mut mdl_data));
        let (mut vpos, mut vnrm, mut vuv0, mut indc) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let mut nvrt = 0;
        for s in model.shapes.iter_mut() {
            vpos.append(&mut s.positions);
            vnrm.append(&mut s.normals);
            vuv0.append(&mut s.texcoords);
            indc.append(&mut s.indices);
            nvrt += vpos.len() / 3;
        }
        let mut vdata = Vec::new();
        vdata.append(&mut vpos);
        vdata.append(&mut vnrm);
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
            gl::Enable(gl::CULL_FACE);
        }

        let wnd_sz = self.window.get_inner_size().unwrap();
        let proj = perspective(
            wnd_sz.width as f32 / wnd_sz.height as f32,
            60.0_f32.to_radians(),
            0.1,
            100.0,
        );
        let view = look_at(
            &vec3(0.0, 0.0, -3.0),
            &vec3(0.0, 0.0, 0.0),
            &vec3(0.0, 1.0, 0.0),
        );
        let modl = rotate_y(&identity(), 26.0_f32.to_radians());
        let nmm = mat4_to_mat3(&inverse_transpose(modl)); // mat3(transpose(inverse(model)))
        let mvp = proj * view * modl;
        let mdl = modl;

        self.shdr.activate();
        self.shdr.set_uniform("model", mdl.as_ref());
        self.shdr.set_uniform("nmm", nmm.as_ref());
        self.shdr.set_uniform("mvp", mvp.as_ref());
        self.shdr.set_uniform("tex", 0);

        // Make time varying movable light
        let time = self.timer.elapsed_msec() / 1000.0;
        let light_pos: Vec3 = vec3(time.sin(), 0.0, time.cos()) * 10.0;
        self.shdr.set_uniform("light_pos", light_pos.as_ref());

        self.tex.bind(0);
        self.mesh.draw();
        self.window.swap_buffers().unwrap();
    }

    pub fn perf(&mut self, ms: f32, ut: f32, rt: f32) {
        let title = format!(
            "[Fps: {:.2} / Msec: {:.2} (CPU: {:.2} | GPU: {:.2})]",
            1000.0 / ms,
            ms,
            ut,
            rt
        );
        self.window.set_title(title.as_str());
    }
}
