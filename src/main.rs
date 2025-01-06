use std::{f32::consts::PI, ffi::CString, fmt::Display, mem::size_of, time::Instant};

use gl::types::{GLuint, GLvoid};
use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};

pub mod compiler;

use compiler::ShaderCompiler;

struct Position {
    x: f32,
    y: f32,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pos({}, {})", self.x, self.y)
    }
}

struct OpenGLApp {
    glfw: Glfw,
    window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>,
    gl_loaded: bool,
    vert_shader_name: GLuint,
    frag_shader_name: GLuint,
    program_name: GLuint,
    array_buffer_name: GLuint,
    element_buffer_name: GLuint,
    vertex_array_name: GLuint,
    vertex_count: i32,
    element_count: i32,
    u_time_location: i32,
    u_resolution: i32,
    u_mouse: i32,
    mouse_pos: Position,
}

impl OpenGLApp {
    fn new(width: u32, height: u32, title: &str) -> OpenGLApp {
        let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to init glfw.");
        let (mut window, events) = glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window");

        window.make_current();
        window.set_key_polling(true);
        window.set_size_polling(true);
        window.set_cursor_pos_polling(true);
        // window.set_framebuffer_size_polling(true);
        window.set_cursor_mode(glfw::CursorMode::Normal);

        OpenGLApp {
            glfw,
            window,
            events,
            gl_loaded: false,
            vert_shader_name: 0,
            frag_shader_name: 0,
            program_name: 0,
            array_buffer_name: 0,
            element_buffer_name: 0,
            vertex_array_name: 0,
            vertex_count: 4,
            element_count: 6,
            u_time_location: 0,
            u_resolution: 0,
            u_mouse: 0,
            mouse_pos: Position { x: 0.0, y: 0.0 },
        }
    }

    fn init_gl(&mut self) {
        if !self.gl_loaded {
            gl::load_with(|symbol| self.window.get_proc_address(symbol) as *const _);
            self.gl_loaded = true;
        }
    }

    fn begin(&mut self) -> bool {
        let mut validated = true;
        print_gl_string("Vendor", gl::VENDOR);
        print_gl_string("Renderer", gl::RENDERER);
        print_gl_string("Version", gl::VERSION);
        // print_gl_string("Extensions", gl::EXTENSIONS);

        if validated {
            validated = self.init_program();
        }

        if validated {
            validated = self.init_buffer();
        }

        if validated {
            validated = self.init_vertex_array();
        }

        validated
    }

    fn init_program(&mut self) -> bool {
        let mut validated = true;
        if validated {
            let mut compiler = ShaderCompiler::new();
            self.vert_shader_name = compiler.create(gl::VERTEX_SHADER, "./src/shaders/vert.glsl");
            self.frag_shader_name = compiler.create(gl::FRAGMENT_SHADER, "./src/shaders/frag.glsl");

            self.program_name = unsafe { gl::CreateProgram() };
            unsafe {
                gl::AttachShader(self.program_name, self.vert_shader_name);
                gl::AttachShader(self.program_name, self.frag_shader_name);

                gl::BindAttribLocation(self.program_name, 0, "Position".as_ptr() as *const i8);
                gl::LinkProgram(self.program_name);

                let u_time_name = CString::new("u_time").unwrap();
                self.u_time_location =
                    gl::GetUniformLocation(self.program_name, u_time_name.as_ptr());

                let u_resolution_name = CString::new("u_resolution").unwrap();
                self.u_resolution =
                    gl::GetUniformLocation(self.program_name, u_resolution_name.as_ptr());

                let u_mouse_name = CString::new("u_mouse").unwrap();
                self.u_mouse = gl::GetUniformLocation(self.program_name, u_mouse_name.as_ptr());
            }
            validated = validated && compiler.check();
            validated = validated && compiler.check_program(self.program_name);
        }

        validated && check_gl_error("init_program")
    }

    fn init_buffer(&mut self) -> bool {
        let array_data = [
            glm::vec2(1.0, 1.0),
            glm::vec2(-1.0, 1.0),
            glm::vec2(-1.0, -1.0),
            glm::vec2(1.0, -1.0),
        ]
        .as_ptr() as *const f32 as *const GLvoid;

        let element_data = [0, 1, 2, 0, 2, 3].as_ptr() as *const f32 as *const GLvoid;

        unsafe {
            gl::GenBuffers(1, &mut self.array_buffer_name);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.array_buffer_name);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertex_count as usize * size_of::<glm::Vector2<f32>>()) as isize,
                array_data,
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            gl::GenBuffers(1, &mut self.element_buffer_name);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.element_buffer_name);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.element_count as usize * size_of::<u32>()) as isize,
                element_data,
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
        check_gl_error("init_buffer")
    }

    fn init_vertex_array(&mut self) -> bool {
        unsafe {
            gl::GenVertexArrays(1, &mut self.vertex_array_name);
            gl::BindVertexArray(self.vertex_array_name);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.array_buffer_name);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.element_buffer_name);
            gl::EnableVertexArrayAttrib(self.vertex_array_name, 0);
            gl::BindVertexArray(0);
        }
        check_gl_error("init_vertex_array")
    }

    fn run(&mut self) {
        if self.window.window_ptr().is_null() {
            panic!("Window not initialized");
        }

        let mut result = true;

        if result {
            result = self.begin();
        }
        let start_time = Instant::now();

        while result {
            self.glfw.poll_events();

            self.handle_events();

            let elapsed_time = start_time.elapsed().as_secs_f32() * PI / 5.0;
            let fixed_time = elapsed_time.rem_euclid(2.0 * PI);

            result = self.render(fixed_time);
            result = result && check_gl_error("render");

            if self.window.should_close() {
                break;
            }

            self.window.swap_buffers();
        }
    }

    #[allow(unused_variables)]
    fn render(&self, u_time: f32) -> bool {
        let buffers = gl::BACK;
        let (width, height) = self.get_window_size();

        unsafe {
            gl::DrawBuffer(buffers);

            gl::Viewport(0, 0, width, height);

            gl::ClearColor(0.2, 0.3, 0.3, 0.1);
            gl::ClearDepthf(1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::UseProgram(self.program_name);
            gl::Uniform2f(self.u_resolution, width as f32, height as f32);
            gl::Uniform2f(self.u_mouse, self.mouse_pos.x, self.mouse_pos.y);
            gl::Uniform1f(self.u_time_location, u_time);

            gl::BindVertexArray(self.vertex_array_name);

            gl::DrawElements(
                gl::TRIANGLES,
                self.element_count,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
        true
    }

    fn get_window_size(&self) -> (i32, i32) {
        self.window.get_framebuffer_size()
    }

    fn handle_events(&mut self) {
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                WindowEvent::Size(width, height) => unsafe {
                    gl::Viewport(0, 0, width, height);
                },
                WindowEvent::CursorPos(x, y) => {
                    let (_, height) = self.get_window_size();
                    self.mouse_pos.x = x as f32;
                    self.mouse_pos.y = height as f32 - y as f32;
                }
                _ => {}
            }
        }
    }
}

fn check_gl_error(title: &str) -> bool {
    let error = unsafe { gl::GetError() };

    if error == gl::NO_ERROR {
        return true;
    }

    let error_string = match error {
        gl::INVALID_ENUM => "GL_INVALID_ENUM",
        gl::INVALID_VALUE => "GL_INVALID_VALUE",
        gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
        gl::INVALID_FRAMEBUFFER_OPERATION => "INVALID_FRAMEBUFFER_OPERATION",
        gl::OUT_OF_MEMORY => "OUT_OF_MEMORY",
        _ => "UNKNOWN",
    };

    println!("OpenGL Error{}: {}", error_string, title);

    error == gl::NO_ERROR
}

fn print_gl_string(name_str: &str, name: gl::types::GLenum) {
    if gl::GetString::is_loaded() {
        let string_raw = unsafe { gl::GetString(name) };
        let length = {
            let mut len = 0;
            while unsafe { *string_raw.add(len) } != 0 {
                len += 1;
            }
            len
        };
        let s = std::str::from_utf8(unsafe { std::slice::from_raw_parts(string_raw, length) });
        println!("{}: {}", name_str, s.unwrap());
    }
}

fn main() {
    let mut app = OpenGLApp::new(800, 600, "Shader test");
    app.init_gl();
    println!();
    app.run();
}
