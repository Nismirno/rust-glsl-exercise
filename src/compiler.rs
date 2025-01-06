use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    str::FromStr,
};

use gl::types::{GLchar, GLenum, GLuint};

#[derive(Default)]
pub struct ShaderCompiler {
    shader_files: HashMap<GLuint, String>,
    shader_names: HashMap<String, GLuint>,
    pending_checks: HashMap<String, GLuint>,
}

impl ShaderCompiler {
    pub fn new() -> ShaderCompiler {
        ShaderCompiler {
            shader_names: HashMap::new(),
            shader_files: HashMap::new(),
            pending_checks: HashMap::new(),
        }
    }
    pub fn create(&mut self, shader_type: GLenum, file_name: &str) -> GLuint {
        let shader_source = std::fs::read_to_string(file_name)
            .unwrap_or_else(|_| format!("File {} not found", file_name));

        let shader_source_cstr = CString::new(shader_source).unwrap();
        let shader_source_ptr = shader_source_cstr.as_ptr();

        let name = unsafe { gl::CreateShader(shader_type) };
        unsafe {
            gl::ShaderSource(name, 1, &shader_source_ptr, std::ptr::null());
            gl::CompileShader(name);
        }

        let file_name = String::from_str(file_name).unwrap();
        self.shader_files.insert(name, file_name.clone());
        self.shader_names.insert(file_name.clone(), name);
        self.pending_checks.insert(file_name.clone(), name);

        name
    }

    pub fn check(&self) -> bool {
        let mut success = true;

        for (_, name) in self.pending_checks.iter() {
            let mut result = 0;
            unsafe {
                gl::GetShaderiv(*name, gl::COMPILE_STATUS, &mut result as *mut i32);
                println!(
                    "Got {} shader compile status {}",
                    self.shader_files.get(name).unwrap(),
                    result
                );
            }

            if result == 1 {
                continue;
            }

            let mut info_log_length: i32 = 0;
            unsafe {
                gl::GetShaderiv(*name, gl::INFO_LOG_LENGTH, &mut info_log_length as *mut i32);
                println!(
                    "Checked {} shader info log length {}",
                    self.shader_files.get(name).unwrap(),
                    info_log_length
                );
            }

            if info_log_length > 0 {
                let mut buffer: Vec<GLchar> = vec![0; info_log_length as usize];
                let mut buffer_len = 0;
                unsafe {
                    gl::GetShaderInfoLog(
                        *name,
                        info_log_length,
                        &mut buffer_len as *mut i32,
                        buffer.as_mut_ptr(),
                    );
                    println!(
                        "Got {} shader info log",
                        self.shader_files.get(name).unwrap()
                    );
                    let info_log = CStr::from_ptr(buffer.as_ptr())
                        .to_string_lossy()
                        .to_string();
                    println!("{}", info_log);
                }
            }

            success = success && result == 1;
        }

        success
    }

    pub fn check_program(&self, program_name: GLuint) -> bool {
        if program_name == 0 {
            return false;
        }

        let mut result = 0;

        unsafe {
            gl::GetProgramiv(program_name, gl::LINK_STATUS, &mut result as *mut i32);
        }

        if result == 1 {
            return true;
        }

        let mut info_log_length = 0;
        unsafe {
            gl::GetProgramiv(
                program_name,
                gl::INFO_LOG_LENGTH,
                &mut info_log_length as *mut i32,
            );
        }

        if info_log_length > 0 {
            let buffer: *mut i8 = std::ptr::null_mut();
            unsafe {
                gl::GetProgramInfoLog(program_name, info_log_length, std::ptr::null_mut(), buffer);
            }
            let info_log = unsafe {
                std::str::from_utf8(std::slice::from_raw_parts(
                    buffer as *const u8,
                    info_log_length as usize,
                ))
                .unwrap()
            };
            println!("{}", info_log);
        }

        result == 1
    }
}
