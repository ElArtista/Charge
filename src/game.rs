use gl;
use glutin::{
    dpi::*, ContextBuilder, ElementState, Event, EventsLoop, GlContext, GlProfile, GlRequest,
    GlWindow, VirtualKeyCode, WindowBuilder, WindowEvent,
};
use graphics::*;

const WND_DIMENSIONS: (f64, f64) = (1280.0, 720.0);

const VERTEX_SHADER: &str = "\
#version 330 core
in vec3 position;

void main()
{
    gl_Position = vec4(position, 1.0);
}
";

const FRAGMENT_SHADER: &str = "\
#version 330 core
out vec4 color;

void main()
{
    color = vec4(1.0f, 0.0f, 0.0f, 1.0f);
}
";

const VERTICES: &[f32] = &[
    // Positions
    -0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0,
];

pub struct Game {
    events_loop: EventsLoop,
    window: GlWindow,
    shdr: Shader,
    mesh: Mesh,
}

impl Game {
    pub fn new() -> Game {
        // Event pump
        let events_loop = EventsLoop::new();

        // Plain window
        let window = WindowBuilder::new()
            .with_dimensions(LogicalSize::new(WND_DIMENSIONS.0, WND_DIMENSIONS.1))
            .with_resizable(false);

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
        let shdr = Shader::from_sources(VERTEX_SHADER, None, FRAGMENT_SHADER);
        shdr.setup_attrib_indexes(&["position"]);

        // Load sample mesh
        let mesh = Mesh::from_data(VERTICES, None, vattr_flag(Vattr::Position));

        Game {
            events_loop: events_loop,
            window: gl_window,
            shdr: shdr,
            mesh: mesh,
        }
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
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        self.shdr.activate();
        self.mesh.draw();
        self.window.swap_buffers().unwrap();
    }

    pub fn perf(&self, ms: f32, fps: f32) {
        let title = format!("[Msec: {:.2} / Fps: {:.2}]", ms, fps);
        self.window.set_title(title.as_str());
    }
}
