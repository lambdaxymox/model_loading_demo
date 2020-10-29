#![allow(dead_code)]
use crate::gl;
use crate::gl::types::{
    GLboolean, 
    GLchar, 
    GLenum, 
    GLfloat, 
    GLint, 
    GLubyte, 
    GLuint,
    GLvoid
};
use glfw;
use glfw::{
    Context, 
    Glfw
};
use image::png::{
    PngDecoder,
};
use image::{
    ImageDecoder,
};
use std::error;
use std::ffi::{
    CStr, 
    CString,
};
use std::fs::{
    File,
};
use std::io::{
    Read, 
    BufReader,
    Cursor,
};
use std::sync::mpsc::{
    Receiver
};
use std::ptr;
use std::fmt;
use std::mem;
use std::path::{
    Path
};

use log::{
    info, 
    error
};


// 256 Kilobytes.
const MAX_SHADER_LENGTH: usize = 262144;

const FPS_COUNTER_REFRESH_PERIOD_SECONDS: f64 = 0.5;

// OpenGL extension constants.
const GL_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FE;
const GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FF;


#[inline]
pub fn glubyte_ptr_to_string(cstr: *const GLubyte) -> String {
    unsafe {
        CStr::from_ptr(cstr as *const i8).to_string_lossy().into_owned()
    }
}

#[inline]
pub fn gl_str(st: &str) -> CString {
    CString::new(st).unwrap()
}

/// A record containing a description of the GL capabilities on a local machine.
/// The contents of this record can be used for debugging OpenGL problems on
/// different machines.
struct GLParameters {
    params: Vec<(String, String)>
}

impl fmt::Display for GLParameters {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "GL Context Params:").unwrap();
        for &(ref param, ref value) in self.params.iter() {
            writeln!(f, "{} = {}", param, value).unwrap();
        }
        writeln!(f)
    }
}

/// Print out the GL capabilities on a local machine. This is handy for debugging
/// OpenGL program problems on other people's machines.
fn gl_params() -> GLParameters {
    let params: [GLenum; 12] = [
        gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS,
        gl::MAX_CUBE_MAP_TEXTURE_SIZE,
        gl::MAX_DRAW_BUFFERS,
        gl::MAX_FRAGMENT_UNIFORM_COMPONENTS,
        gl::MAX_TEXTURE_IMAGE_UNITS,
        gl::MAX_TEXTURE_SIZE,
        gl::MAX_VARYING_FLOATS,
        gl::MAX_VERTEX_ATTRIBS,
        gl::MAX_VERTEX_TEXTURE_IMAGE_UNITS,
        gl::MAX_VERTEX_UNIFORM_COMPONENTS,
        gl::MAX_VIEWPORT_DIMS,
        gl::STEREO,
    ];
    let names: [&str; 12] = [
        "GL_MAX_COMBINED_TEXTURE_IMAGE_UNITS",
        "GL_MAX_CUBE_MAP_TEXTURE_SIZE",
        "GL_MAX_DRAW_BUFFERS",
        "GL_MAX_FRAGMENT_UNIFORM_COMPONENTS",
        "GL_MAX_TEXTURE_IMAGE_UNITS",
        "GL_MAX_TEXTURE_SIZE",
        "GL_MAX_VARYING_FLOATS",
        "GL_MAX_VERTEX_ATTRIBS",
        "GL_MAX_VERTEX_TEXTURE_IMAGE_UNITS",
        "GL_MAX_VERTEX_UNIFORM_COMPONENTS",
        "GL_MAX_VIEWPORT_DIMS",
        "GL_STEREO",
    ];
    let mut vec: Vec<(String, String)> = vec![];
    // Integers: this only works if the order is 0-10 integer return types.
    for i in 0..10 {
        let mut v = 0;
        unsafe { 
            gl::GetIntegerv(params[i], &mut v);
        }
        vec.push((format!("{}", names[i]), format!("{}", v)));
    }
    // others
    let mut v: [GLint; 2] = [0; 2];
    unsafe {    
        gl::GetIntegerv(params[10], &mut v[0]);
    }
    vec.push((format!("{}", names[10]), format!("{} {}", v[0], v[1])));
    let mut s = 0;
    unsafe {
        gl::GetBooleanv(params[11], &mut s);
    }
    vec.push((format!("{}", names[11]), format!("{}", s as usize)));

    GLParameters {
        params: vec,
    }
}

/// Helper function to convert GLSL types to storage sizes
fn type_size(gl_type: GLenum) -> usize {
    match gl_type {
        gl::FLOAT             => 1 * mem::size_of::<GLfloat>(),
        gl::FLOAT_VEC2        => 2 * mem::size_of::<GLfloat>(),
        gl::FLOAT_VEC3        => 3 * mem::size_of::<GLfloat>(),
        gl::FLOAT_VEC4        => 4 * mem::size_of::<GLfloat>(),
        gl::INT               => 1 * mem::size_of::<GLint>(),
        gl::INT_VEC2          => 2 * mem::size_of::<GLint>(),
        gl::INT_VEC3          => 3 * mem::size_of::<GLint>(),
        gl::INT_VEC4          => 4 * mem::size_of::<GLint>(),
        gl::UNSIGNED_INT      => 1 * mem::size_of::<GLuint>(),
        gl::UNSIGNED_INT_VEC2 => 2 * mem::size_of::<GLuint>(),
        gl::UNSIGNED_INT_VEC3 => 3 * mem::size_of::<GLuint>(),
        gl::UNSIGNED_INT_VEC4 => 4 * mem::size_of::<GLuint>(),
        gl::BOOL              => 1 * mem::size_of::<GLboolean>(),
        gl::BOOL_VEC2         => 2 * mem::size_of::<GLboolean>(),
        gl::BOOL_VEC3         => 3 * mem::size_of::<GLboolean>(),
        gl::BOOL_VEC4         => 4 * mem::size_of::<GLboolean>(),
        gl::FLOAT_MAT2        => 4 * mem::size_of::<GLfloat>(),
        gl::FLOAT_MAT2x3      => 6 * mem::size_of::<GLfloat>(),
        gl::FLOAT_MAT2x4      => 8 * mem::size_of::<GLfloat>(),
        gl::FLOAT_MAT3        => 9 * mem::size_of::<GLfloat>(),
        gl::FLOAT_MAT3x2      => 6 * mem::size_of::<GLfloat>(),
        gl::FLOAT_MAT3x4      => 12 * mem::size_of::<GLfloat>(),
        gl::FLOAT_MAT4        => 16 * mem::size_of::<GLfloat>(),
        gl::FLOAT_MAT4x2      => 8 * mem::size_of::<GLfloat>(),
        gl::FLOAT_MAT4x3      => 12 * mem::size_of::<GLfloat>(),
        _ => panic!()
    }
}

/// A record for storing all the OpenGL state needed on the application side
/// of the graphics application in order to manage OpenGL and GLFW.
pub struct OpenGLContext {
    pub glfw: glfw::Glfw,
    pub window: glfw::Window,
    pub events: Receiver<(f64, glfw::WindowEvent)>,
    pub width: u32,
    pub height: u32,
    pub channel_depth: u32,
    pub running_time_seconds: f64,
    pub framerate_time_seconds: f64,
    pub frame_count: u32,
}

impl OpenGLContext {
    /// Updates the timers in a GL context. It returns the elapsed time since the last call to
    /// `update_timers`.
    #[inline]
    pub fn update_timers(&mut self) -> f64 {
        let current_seconds = self.glfw.get_time();
        let elapsed_seconds = current_seconds - self.running_time_seconds;
        self.running_time_seconds = current_seconds;

        elapsed_seconds
    }

    /// Update the framerate and display in the window titlebar.
    #[inline]
    pub fn update_fps_counter(&mut self) {     
        let current_time_seconds = self.glfw.get_time();
        let elapsed_seconds = current_time_seconds - self.framerate_time_seconds;
        if elapsed_seconds > FPS_COUNTER_REFRESH_PERIOD_SECONDS {
            self.framerate_time_seconds = current_time_seconds;
            let fps = self.frame_count as f64 / elapsed_seconds;
            self.window.set_title(&format!("OpenGL DEMO @ {:.2}", fps));
            self.frame_count = 0;
        }

        self.frame_count += 1;
    }
}

#[cfg(target_os = "macos")]
fn __init_glfw() -> Glfw {
    // Start a GL context and OS window using the GLFW helper library.
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // We must place the window hints before creating the window because
    // glfw cannot change the properties of a window after it has been created.
    glfw.window_hint(glfw::WindowHint::Resizable(true));
    glfw.window_hint(glfw::WindowHint::Samples(Some(4)));
    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    glfw
}

#[cfg(target_os = "windows")]
fn __init_glfw() -> Glfw {
    // Start a GL context and OS window using the GLFW helper library.
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // We must place the window hints before creating the window because
    // glfw cannot change the properties of a window after it has been created.
    glfw.window_hint(glfw::WindowHint::Resizable(true));
    glfw.window_hint(glfw::WindowHint::Samples(Some(4)));
    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    glfw
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn __init_glfw() -> Glfw {
    // Start a GL context and OS window using the GLFW helper library.
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // We must place the window hints before creating the window because
    // glfw cannot change the properties of a window after it has been created.
    glfw.window_hint(glfw::WindowHint::Resizable(true));
    glfw.window_hint(glfw::WindowHint::Samples(Some(4)));

    glfw
}

/// Initialize a new OpenGL context and start a new GLFW window. 
pub fn start_opengl(width: u32, height: u32) -> Result<OpenGLContext, String> {
    // Start GL context and O/S window using the GLFW helper library.
    info!("Starting GLFW");
    info!("Using GLFW version {}", glfw::get_version_string());

    // Start a GL context and OS window using the GLFW helper library.
    let glfw = __init_glfw();

    info!("Started GLFW successfully");
    let maybe_glfw_window = glfw.create_window(
        width, height, &format!("OpenGL DEMO"), glfw::WindowMode::Windowed
    );
    let (mut window, events) = match maybe_glfw_window {
        Some(tuple) => tuple,
        None => {
            error!("Failed to create GLFW window");
            return Err(String::new());
        }
    };

    window.make_current();
    window.set_key_polling(true);
    window.set_size_polling(true);
    window.set_refresh_polling(true);
    window.set_size_polling(true);
    window.set_sticky_keys(true);

    // Load the OpenGl function pointers.
    gl::load_with(|symbol| { window.get_proc_address(symbol) as *const _ });

    // Get renderer and version information.
    let renderer = glubyte_ptr_to_string(unsafe { gl::GetString(gl::RENDERER) });
    info!("Renderer: {}", renderer);

    let version = glubyte_ptr_to_string(unsafe { gl::GetString(gl::VERSION) });
    info!("OpenGL version supported: {}", version);
    info!("{}", gl_params());

    Ok(OpenGLContext {
        glfw: glfw, 
        window: window, 
        events: events,
        width: width,
        height: height,
        channel_depth: 3,
        running_time_seconds: 0.0,
        framerate_time_seconds: 0.0,
        frame_count: 0,
    })
}


/// Validate that the shader program `shader` can execute with the current OpenGL program state.
/// Use this for information purposes in application development. Return `true` if the program and
/// OpenGL state contain no errors.
pub fn validate_shader_program(shader: GLuint) -> bool {
    let mut params = -1;
    unsafe {
        gl::ValidateProgram(shader);
        gl::GetProgramiv(shader, gl::VALIDATE_STATUS, &mut params);
    }
    if params != gl::TRUE as i32 {
        error!("Program {} GL_VALIDATE_STATUS = GL_FALSE\n", shader);
        error!("{}", program_info_log(shader));
        
        return false;
    }

    info!("Program {} GL_VALIDATE_STATUS = {}\n", shader, params);
    
    true
}

/// A record containing all the relevant compilation log information for a
/// given GLSL shader compiled at run time.
#[derive(Clone, Debug)]
pub struct ShaderLog {
    index: GLuint,
    log: String,
}

impl fmt::Display for ShaderLog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Shader info log for GL index {}:", self.index).unwrap();
        writeln!(f, "{}", self.log)
    }
}

/// Query the shader information log generated during shader compilation from
/// OpenGL.
pub fn shader_info_log(shader_index: GLuint) -> ShaderLog {
    let mut actual_length = 0;
    unsafe {
        gl::GetShaderiv(shader_index, gl::INFO_LOG_LENGTH, &mut actual_length);
    }
    let mut raw_log = vec![0 as i8; actual_length as usize];
    unsafe {
        gl::GetShaderInfoLog(shader_index, raw_log.len() as i32, &mut actual_length, &mut raw_log[0]);
    }
    
    let mut log = String::new();
    for i in 0..actual_length as usize {
        log.push(raw_log[i] as u8 as char);
    }

    ShaderLog { index: shader_index, log: log }
}

/// A record containing all the relevant compilation log information for a
/// given GLSL shader program compiled at run time.
#[derive(Clone, Debug)]
pub struct ProgramLog {
    index: GLuint,
    log: String,
}

impl fmt::Display for ProgramLog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Program info log for GL index {}:", self.index).unwrap();
        writeln!(f, "{}", self.log)
    }
}

impl ProgramLog {
    fn new(index: GLuint, log: String) -> ProgramLog {
        ProgramLog {
            index: index,
            log: log,
        }
    }
}

/// Query the shader program information log generated during shader compilation from OpenGL.
pub fn program_info_log(shader: GLuint) -> ProgramLog {
    let mut actual_length = 0;
    unsafe {
        gl::GetProgramiv(shader, gl::INFO_LOG_LENGTH, &mut actual_length);
    }
    let mut raw_log = vec![0 as i8; actual_length as usize];
    unsafe {
        gl::GetProgramInfoLog(shader, raw_log.len() as i32, &mut actual_length, &mut raw_log[0]);
    }

    let mut log = String::new();
    for i in 0..actual_length as usize {
        log.push(raw_log[i] as u8 as char);
    }

    ProgramLog { index: shader, log: log }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    ShaderNotFound,
    CouldNotParseShader,
    CouldNotCompileShader,
    CouldNotLinkShader,
    ShaderValidationFailed,
}

#[derive(Clone, Debug)]
pub struct ShaderCompilationError {
    kind: ErrorKind,
    shader: Option<u32>,
    shader_name: String,
    log: ProgramLog,
}

impl ShaderCompilationError {
    #[inline]
    fn new(kind: ErrorKind, shader: Option<u32>, shader_name: String, log: ProgramLog) -> Self {
        Self {
            kind: kind,
            shader: shader,
            shader_name: shader_name,
            log: log,
        }
    }
}

impl fmt::Display for ShaderCompilationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::ShaderNotFound => {
                write!(f, 
                    "Could not open the shader file `{}` for reading.", 
                    self.shader_name
                )
            }
            ErrorKind::CouldNotParseShader => {
                write!(f, 
                    "The shader file `{}` exists, but there was an error in reading it.", 
                    self.shader_name
                )
            }
            ErrorKind::CouldNotCompileShader => {
                write!(f, 
                    "The shader program `{}` could not be compiled.\nLOG GENERATED BY OPENGL:\n{}\n", 
                    self.shader_name, self.log
                )
            }
            ErrorKind::CouldNotLinkShader => {
                write!(f, 
                    "The shader program `{}` could not be linked.\nLOG GENERATED BY OPENGL:\n{}\n",
                    self.shader_name, self.log
                )
            }
            ErrorKind::ShaderValidationFailed => {
                write!(f, 
                    "Shader validation failed for the shader program `{}`.\nLOG GENERATED BY OPENGL:\n{}\n",
                    self.shader_name, self.log
                )
            }
        }
    }
}

impl error::Error for ShaderCompilationError {}


/// Load the shader source file(s).
pub fn parse_shader<P: AsRef<Path>, R: Read>(
    reader: &mut R, file_name: P, shader_str: &mut [u8]) -> Result<usize, ShaderCompilationError> {

    shader_str[0] = 0;
    let bytes_read = match reader.read(shader_str) {
        Ok(val) => val,
        Err(_) => {
            let kind = ErrorKind::CouldNotParseShader;
            let shader_name = file_name.as_ref().display().to_string();
            let log = ProgramLog::new(0, String::from(""));
            return Err(ShaderCompilationError::new(kind, None, shader_name, log));
        }
    };

    // Append \0 character to end of the shader string to mark the end of a C string.
    shader_str[bytes_read] = 0;

    Ok(bytes_read)
}

/// Create a shader from source files.
pub fn compile_shader<P: AsRef<Path>, R: Read>(
    reader: &mut R, file_name: P, kind: GLenum) -> Result<GLuint, ShaderCompilationError> {

    let disp = file_name.as_ref().display();
    info!("Creating shader from {}.\n", disp);

    let mut shader_string = vec![0; MAX_SHADER_LENGTH];
    let bytes_read = match parse_shader(reader, &file_name, &mut shader_string) {
        Ok(val) => val,
        Err(e) => {
            error!("{}", e);
            return Err(e);
        }
    };

    if bytes_read >= (MAX_SHADER_LENGTH - 1) {
        info!(
            "WARNING: The shader was truncated because the shader code 
            was longer than MAX_SHADER_LENGTH {} bytes.", MAX_SHADER_LENGTH
        );
    }

    let shader = unsafe { 
        gl::CreateShader(kind)
    };
    let pointer = shader_string.as_ptr() as *const GLchar;
    unsafe {
        gl::ShaderSource(shader, 1, &pointer, ptr::null());
        gl::CompileShader(shader);
    }

    // Check for shader compile errors.
    let mut params = -1;
    unsafe {
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut params);
    }

    if params != gl::TRUE as i32 {
        let error_kind = ErrorKind::CouldNotCompileShader;
        let shader_index = Some(shader);
        let shader_name = file_name.as_ref().display().to_string();
        let shader_log = shader_info_log(shader);
        let log = ProgramLog::new(shader_log.index, shader_log.log);
        error!("ERROR: GL shader index {} did not compile\n{}", shader, log);
        
        return Err(ShaderCompilationError::new(error_kind, shader_index, shader_name, log));
    }
    info!("Shader compiled with index {}.\n", shader);
    
    Ok(shader)
}

/// Compile and link a shader program.
pub fn link_shader(
    vertex_shader: GLuint, fragment_shader: GLuint) -> Result<GLuint, ShaderCompilationError> {

    let program = unsafe { gl::CreateProgram() };
    info!("Created program {}. Attaching shaders {} and {}.\n",
        program, vertex_shader, fragment_shader
    );

    unsafe {
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        // Link the shader program. If binding input attributes, do that before linking.
        gl::LinkProgram(program);
    }

    let mut params = -1;
    unsafe {
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut params);
    }
    if params != gl::TRUE as i32 {
        let kind = ErrorKind::CouldNotLinkShader;
        let shader_index = Some(program);
        let shader_name = String::from("");
        let log = program_info_log(program);
        error!("ERROR: could not link shader program GL index {}\n", program);
        error!("{}", log);
        return Err(ShaderCompilationError::new(kind, shader_index, shader_name, log));
    }

    unsafe {
        // Delete shaders here to free memory.
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
    }

    Ok(program)
}

/// Compile and link a shader program directly from the files.
pub fn compile_from_files<P: AsRef<Path>, Q: AsRef<Path>>(
    context: &OpenGLContext,
    vert_file_name: P, frag_file_name: Q) -> Result<GLuint, ShaderCompilationError> {

    let mut vert_reader = BufReader::new(match File::open(&vert_file_name) {
        Ok(val) => val,
        Err(_) => {
            let kind = ErrorKind::ShaderNotFound;
            let shader = None;
            let shader_name = vert_file_name.as_ref().display().to_string();
            let log = ProgramLog::new(0, String::from(""));
            return Err(ShaderCompilationError::new(kind, shader, shader_name, log));
        }
    });
    let mut frag_reader = BufReader::new(match File::open(&frag_file_name) {
        Ok(val) => val,
        Err(_) => {
            let kind = ErrorKind::ShaderNotFound;
            let shader = None;
            let shader_name = frag_file_name.as_ref().display().to_string();
            let log = ProgramLog::new(0, String::from(""));
            return Err(ShaderCompilationError::new(kind, shader, shader_name, log));
        }
    });

    let vertex_shader = compile_shader(
        &mut vert_reader, vert_file_name, gl::VERTEX_SHADER
    )?;
    let fragment_shader = compile_shader(
        &mut frag_reader, frag_file_name, gl::FRAGMENT_SHADER
    )?;
    let program = link_shader(vertex_shader, fragment_shader)?;

    Ok(program)
}

/// Compile and link a shader program directly from any readable sources.
pub fn compile_from_reader<R1: Read, P1: AsRef<Path>, R2: Read, P2: AsRef<Path>>(
    vert_reader: &mut R1, vert_file_name: P1,
    frag_reader: &mut R2, frag_file_name: P2) -> Result<GLuint, ShaderCompilationError> {

    let vertex_shader = compile_shader(
        vert_reader, vert_file_name, gl::VERTEX_SHADER
    )?;
    let fragment_shader = compile_shader(
        frag_reader, frag_file_name, gl::FRAGMENT_SHADER
    )?;

    let program = link_shader(vertex_shader, fragment_shader)?;

    Ok(program)
}

pub fn compile(shader_source: &ShaderSource) -> Result<ShaderHandle, ShaderCompilationError> {
    let mut vert_reader = Cursor::new(shader_source.vertex_source);
    let mut frag_reader = Cursor::new(shader_source.fragment_source);
    let result = compile_from_reader(
        &mut vert_reader, shader_source.vertex_name,
        &mut frag_reader, shader_source.fragment_name
    );
    let shader = match result {
        Ok(value) => value,
        Err(e) => {
            panic!("Could not compile shaders. Got error: {}", e);
        }
    };
    debug_assert!(shader > 0);

    Ok(ShaderHandle::new(shader))
}

use cglinalg::{
    Vector2,
    Vector3,
    Vector4,
    Matrix2,
    Matrix3,
    Matrix4,
};

#[derive(Copy, Clone, Debug)]
pub struct ShaderSource<'a, 'b, 'c> {
    vertex_name: &'a str,
    vertex_source: &'a str,
    fragment_name: &'b str,
    fragment_source: &'b str,
    geometry_name: Option<&'c str>,
    geometry_source: Option< &'c str>,
}

pub struct ShaderSourceBuilder<'a, 'b, 'c> {
    vertex_name: &'a str,
    vertex_source: &'a str,
    fragment_name: &'b str,
    fragment_source: &'b str,
    geometry_name: Option<&'c str>,
    geometry_source: Option< &'c str>,
}

impl<'a, 'b, 'c> ShaderSourceBuilder<'a, 'b, 'c> {
    pub fn new(
        vertex_name: &'a str, 
        vertex_source: &'a str, 
        fragment_name: &'b str, 
        fragment_source: &'b str) -> Self 
    {
        Self {
            vertex_name: vertex_name,
            vertex_source: vertex_source,
            fragment_name: fragment_name,
            fragment_source: fragment_source,
            geometry_name: None,
            geometry_source: None,
        }
    }

    pub fn with_geometry_shader(&mut self, geometry_name: &'c str, geometry_source: &'c str) {
        self.geometry_name = Some(geometry_name);
        self.geometry_source = Some(geometry_source);
    }

    pub fn build(self) -> ShaderSource<'a, 'b, 'c> {
        ShaderSource {
            vertex_name: self.vertex_name,
            vertex_source: self.vertex_source,
            fragment_name: self.fragment_name,
            fragment_source: self.fragment_source,
            geometry_name: self.geometry_name,
            geometry_source: self.geometry_source,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ShaderHandle {
    pub id: u32,
}

impl ShaderHandle {
    #[inline]
    const fn new(id: u32) -> ShaderHandle {
        ShaderHandle {
            id: id,
        }
    }

    #[inline]
    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    #[inline]
    pub fn get_attrib_location(&self, name: &str) -> u32 {
        let location = unsafe {
            gl::GetAttribLocation(self.id, gl_str(name).as_ptr())
        };
        debug_assert!(location > -1);

        location as u32
    }

    #[inline]
    pub fn get_uniform_location(&self, name: &str) -> i32 {
        let location = unsafe {
            gl::GetUniformLocation(self.id, gl_str(name).as_ptr())
        };
        debug_assert!(location > -1);

        location
    }

    #[inline]
    pub fn set_bool(&self, name: &str, value: bool) {
        let location = self.get_uniform_location(name);
        unsafe {
            gl::Uniform1i(location, value as i32);
        }
    }

    #[inline]
    pub fn set_int(&self, name: &str, value: i32) {
        let location = self.get_uniform_location(name);
        unsafe {
            gl::Uniform1i(location, value);
        }
    }

    #[inline]
    pub fn set_float(&self, name: &str, value: f32) {
        let location = self.get_uniform_location(name);
        unsafe {
            gl::Uniform1f(location, value);
        }
    }

    #[inline]
    pub fn set_vec2(&self, name: &str, value: &Vector2<f32>) {
        let location = self.get_uniform_location(name);
        unsafe {
            gl::Uniform2fv(location, 1, value.as_ptr());
        }
    }

    #[inline]
    pub fn set_vec3(&self, name: &str, value: &Vector3<f32>) {
        let location = self.get_uniform_location(name);
        unsafe {
            gl::Uniform3fv(location, 1, value.as_ptr());
        }
    }

    #[inline]
    pub fn set_vec4(&self, name: &str, value: &Vector4<f32>) {
        let location = self.get_uniform_location(name);
        unsafe {
            gl::Uniform4fv(location, 1, value.as_ptr());
        }
    }

    #[inline]
    pub fn set_mat2(&self, name: &str, value: &Matrix2<f32>) {
        let location = self.get_uniform_location(name);
        unsafe {
            gl::UniformMatrix2fv(location, 1, gl::FALSE, value.as_ptr());
        }
    }

    #[inline]
    pub fn set_mat3(&self, name: &str, value: &Matrix3<f32>) {
        let location = self.get_uniform_location(name);
        unsafe {
            gl::UniformMatrix3fv(location, 1, gl::FALSE, value.as_ptr());
        }
    }

    #[inline]
    pub fn set_mat4(&self, name: &str, value: &Matrix4<f32>) {
        let location = self.get_uniform_location(name);
        unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, value.as_ptr());
        }
    }
}

pub fn load_image(buffer: &[u8]) -> TextureImage2D {
    let cursor = Cursor::new(buffer);
    let image_decoder = PngDecoder::new(cursor).unwrap();
    let (width, height) = image_decoder.dimensions();
    let total_bytes = image_decoder.total_bytes();
    let bytes_per_pixel = image_decoder.color_type().bytes_per_pixel() as u32;
    let mut image_data = vec![0 as u8; total_bytes as usize];
    image_decoder.read_image(&mut image_data).unwrap();

    assert_eq!(total_bytes, (width * height * bytes_per_pixel) as u64);

    TextureImage2D::new(width, height, bytes_per_pixel, image_data)
}

/// Load texture image into the GPU.
pub fn send_to_gpu_texture(texture_image: &TextureImage2D, wrapping_mode: GLuint) -> Result<GLuint, String> {
    let mut tex = 0;
    unsafe {
        gl::GenTextures(1, &mut tex);
    }
    debug_assert!(tex > 0);
    unsafe {
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexImage2D(
            gl::TEXTURE_2D, 0, gl::RGBA as i32, texture_image.width as i32, texture_image.height as i32, 0,
            gl::RGBA, gl::UNSIGNED_BYTE,
            texture_image.as_ptr() as *const GLvoid
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrapping_mode as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrapping_mode as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as GLint);
    }

    let mut max_aniso = 0.0;
    unsafe {
        gl::GetFloatv(GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT, &mut max_aniso);
        gl::TexParameterf(gl::TEXTURE_2D, GL_TEXTURE_MAX_ANISOTROPY_EXT, max_aniso);
    }

    Ok(tex)
}


#[derive(Clone)]
pub struct TextureImage2D {
    pub width: u32,
    pub height: u32,
    pub bytes_per_pixel: u32,
    data: Vec<u8>,
}

impl TextureImage2D {
    pub fn new(width: u32, height: u32, bytes_per_pixel: u32, data: Vec<u8>) -> Self {
        Self {
            width: width,
            height: height,
            bytes_per_pixel: bytes_per_pixel,
            data: data,
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

