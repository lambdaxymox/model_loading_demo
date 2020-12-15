extern crate glfw;
extern crate cglinalg;
extern crate cgperspective;
extern crate image;
extern crate log;
extern crate file_logger;
extern crate wavefront_obj;
extern crate mini_obj;


mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

mod backend;
mod camera;
mod light;
mod texture;
mod model;


use cglinalg::{
    Angle,
    Degrees,
    Matrix4,
    Radians,
    Vector3,
    Unit,
};
use cgperspective::{
    SimpleCameraMovement,
    CameraMovement,
    CameraAttitudeSpec,
    PerspectiveFovSpec,
    FreeKinematicsSpec,
    Camera
};
use glfw::{
    Action, 
    Context, 
    Key
};
use gl::types::{
    GLfloat,
    GLint,
    GLuint, 
    GLvoid, 
    GLsizeiptr,
};
use log::{
    info
};
use mini_obj::{
    ObjMesh
};

use crate::backend::{
    OpenGLContext,
    ShaderSourceBuilder,
    ShaderSource,
    ShaderHandle,
};
use crate::camera::{
    PerspectiveFovCamera,
};
use crate::light::*;
use crate::model::*;

use std::mem;
use std::ptr;


// Default value for the color buffer.
const CLEAR_COLOR: [f32; 4] = [0.1_f32, 0.1_f32, 0.1_f32, 1.0_f32];
// Default value for the depth buffer.
const CLEAR_DEPTH: [f32; 4] = [1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32];

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;


#[inline]
fn offset_of<S, T>(ptr1: &S, ptr2: &T) -> usize {
    unsafe {
        ptr1 as *const S as usize - ptr2 as *const T as usize
    }
}

fn create_backpack_model() -> Model {
    let buffer = include_bytes!("../assets/backpack.zip");
    let asset = model::load_from_memory(buffer, "backpack.zip", false).unwrap();

    asset
}

fn create_box_mesh() -> ObjMesh {
    let points: Vec<[f32; 3]> = vec![
        [-0.5, -0.5, -0.5], [ 0.5, -0.5, -0.5], [ 0.5,  0.5, -0.5],
        [ 0.5,  0.5, -0.5], [-0.5,  0.5, -0.5], [-0.5, -0.5, -0.5],
        [-0.5, -0.5,  0.5], [ 0.5, -0.5,  0.5], [ 0.5,  0.5,  0.5],  
        [ 0.5,  0.5,  0.5], [-0.5,  0.5,  0.5], [-0.5, -0.5,  0.5],
        [-0.5,  0.5,  0.5], [-0.5,  0.5, -0.5], [-0.5, -0.5, -0.5], 
        [-0.5, -0.5, -0.5], [-0.5, -0.5,  0.5], [-0.5,  0.5,  0.5], 
        [ 0.5,  0.5,  0.5], [ 0.5,  0.5, -0.5], [ 0.5, -0.5, -0.5], 
        [ 0.5, -0.5, -0.5], [ 0.5, -0.5,  0.5], [ 0.5,  0.5,  0.5],
        [-0.5, -0.5, -0.5], [ 0.5, -0.5, -0.5], [ 0.5, -0.5,  0.5],  
        [ 0.5, -0.5,  0.5], [-0.5, -0.5,  0.5], [-0.5, -0.5, -0.5],
        [-0.5,  0.5, -0.5], [ 0.5,  0.5, -0.5], [ 0.5,  0.5,  0.5], 
        [ 0.5,  0.5,  0.5], [-0.5,  0.5,  0.5], [-0.5,  0.5, -0.5],  
    ];
    let tex_coords = vec![
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0],
        [1.0, 1.0], [0.0, 1.0], [0.0, 0.0],
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0],
        [1.0, 1.0], [0.0, 1.0], [0.0, 0.0],
        [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        [0.0, 1.0], [0.0, 0.0], [1.0, 0.0],
        [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        [0.0, 1.0], [0.0, 0.0], [1.0, 0.0],
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0],
        [1.0, 0.0], [0.0, 0.0], [0.0, 1.0],
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0],
        [1.0, 0.0], [0.0, 0.0], [0.0, 1.0]
    ];
    let normals = vec![
        [ 0.0,  0.0, -1.0], [ 0.0,  0.0, -1.0], [ 0.0,  0.0, -1.0],
        [ 0.0,  0.0, -1.0], [ 0.0,  0.0, -1.0], [ 0.0,  0.0, -1.0],
        [ 0.0,  0.0,  1.0], [ 0.0,  0.0,  1.0], [ 0.0,  0.0,  1.0],
        [ 0.0,  0.0,  1.0], [ 0.0,  0.0,  1.0], [ 0.0,  0.0,  1.0],
        [-1.0,  0.0,  0.0], [-1.0,  0.0,  0.0], [-1.0,  0.0,  0.0],
        [-1.0,  0.0,  0.0], [-1.0,  0.0,  0.0], [-1.0,  0.0,  0.0],
        [ 1.0,  0.0,  0.0], [ 1.0,  0.0,  0.0], [ 1.0,  0.0,  0.0],
        [ 1.0,  0.0,  0.0], [ 1.0,  0.0,  0.0], [ 1.0,  0.0,  0.0],
        [ 0.0, -1.0,  0.0], [ 0.0, -1.0,  0.0], [ 0.0, -1.0,  0.0],
        [ 0.0, -1.0,  0.0], [ 0.0, -1.0,  0.0], [ 0.0, -1.0,  0.0],
        [ 0.0,  1.0,  0.0], [ 0.0,  1.0,  0.0], [ 0.0,  1.0,  0.0],
        [ 0.0,  1.0,  0.0], [ 0.0,  1.0,  0.0], [ 0.0,  1.0,  0.0],
    ];

    ObjMesh::new(points, tex_coords, normals)
}

fn create_camera(width: u32, height: u32) -> PerspectiveFovCamera<f32> {
    let near = 0.1;
    let far = 100.0;
    let fovy = Degrees(72.0);
    let aspect = width as f32 / height as f32;
    let model_spec = PerspectiveFovSpec::new(
        fovy, 
        aspect, 
        near, 
        far
    );
    let position = Vector3::new(0.0, 0.0, 3.0);
    let forward = Vector3::new(0.0, 0.0, 1.0);
    let right = Vector3::new(1.0, 0.0, 0.0);
    let up  = Vector3::new(0.0, 1.0, 0.0);
    let axis = Vector3::new(0.0, 0.0, -1.0);
    let attitude_spec = CameraAttitudeSpec::new(
        position,
        forward,
        right,
        up,
        axis,
    );
    let movement_speed = 5.0;
    let rotation_speed = Degrees(50.0);
    let kinematics_spec = FreeKinematicsSpec::new(
        movement_speed, 
        rotation_speed
    );

    Camera::new(&model_spec, &attitude_spec, &kinematics_spec)
}

fn create_cube_lights() -> [PointLight<f32>; 4] {
    let position_0 = Vector3::new(0.7, 0.2, 2.0);
    let ambient_0 = Vector3::new(0.2, 0.2, 0.2);
    let diffuse_0 = Vector3::new(0.5, 0.5, 0.5);
    let specular_0 = Vector3::new(1.0, 1.0, 1.0);
    let constant_0 = 1.0;
    let linear_0 = 0.09;
    let quadratic_0 = 0.032;
    let light_0 = PointLight::new(
        position_0,
        constant_0,
        linear_0,
        quadratic_0,
        ambient_0, 
        diffuse_0, 
        specular_0
    );

    let position_1 = Vector3::new(2.3, -3.3, -4.0);
    let ambient_1 = Vector3::new(0.2, 0.2, 0.2);   
    let diffuse_1 = Vector3::new(0.5, 0.5, 0.5);
    let specular_1 = Vector3::new(1.0, 1.0, 1.0);
    let constant_1 = 1.0;
    let linear_1 = 0.09;
    let quadratic_1 = 0.032;
    let light_1 = PointLight::new(
        position_1,
        constant_1,
        linear_1,
        quadratic_1,
        ambient_1,
        diffuse_1,
        specular_1
    );


    let position_2 = Vector3::new(-4.0, 2.0, -12.0);
    let ambient_2 = Vector3::new(0.2, 0.2, 0.2);
    let diffuse_2 = Vector3::new(0.5, 0.5, 0.5);
    let specular_2 = Vector3::new(1.0, 1.0, 1.0);
    let constant_2 = 1.0;
    let linear_2 = 0.09;
    let quadratic_2 = 0.032;
    let light_2 = PointLight::new(
        position_2,
        constant_2,
        linear_2,
        quadratic_2,
        ambient_2, 
        diffuse_2, 
        specular_2
    );

    let position_3 = Vector3::new(0.0, 0.0, -3.0);
    let ambient_3 = Vector3::new(0.05, 0.05, 0.05);
    let diffuse_3 = Vector3::new(0.8, 0.8, 0.8);
    let specular_3 = Vector3::new(1.0, 1.0, 1.0);
    let constant_3 = 1.0;
    let linear_3 = 0.09;
    let quadratic_3 = 0.032;
    let light_3 = PointLight::new(
        position_3,
        constant_3,
        linear_3,
        quadratic_3,
        ambient_3,
        diffuse_3,
        specular_3
    );

    [light_0, light_1, light_2, light_3]
}

fn create_directional_light() -> DirLight<f32> {
    let direction = Vector3::new(-0.2, -1.0, -0.3);
    let ambient = Vector3::new(0.05, 0.05, 0.05);
    let diffuse = Vector3::new(0.4, 0.4, 0.4);
    let specular = Vector3::new(0.5, 0.5, 0.5);

    DirLight::new(direction, ambient, diffuse, specular)
}

fn create_mesh_shader_source() -> ShaderSource<'static, 'static, 'static> {
    let vertex_name = "multiple_lights.vert.glsl";
    let vertex_source = include_str!("../shaders/multiple_lights.vert.glsl");
    let fragment_name = "multiple_lights.frag.glsl";
    let fragment_source = include_str!("../shaders/multiple_lights.frag.glsl");
    
    ShaderSourceBuilder::new(
        vertex_name,
        vertex_source,
        fragment_name,
        fragment_source)
    .build()
}

fn create_cube_light_shader_source() -> ShaderSource<'static, 'static, 'static> {
    let vertex_name = "lighting_cube.vert.glsl";
    let vertex_source = include_str!("../shaders/lighting_cube.vert.glsl");
    let fragment_name = "lighting_cube.frag.glsl";
    let fragment_source = include_str!("../shaders/lighting_cube.frag.glsl");

    ShaderSourceBuilder::new(
        vertex_name,
        vertex_source,
        fragment_name,
        fragment_source)
    .build()
}

fn send_to_gpu_mesh(mesh: &Mesh) -> (GLuint, GLuint, GLuint) {
    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
    };
    debug_assert!(vao > 0);
    
    let mut vbo = 0; 
    unsafe {
        gl::GenBuffers(1, &mut vbo);
    };
    debug_assert!(vbo > 0);
    
    // Element Buffer Object.
    let mut ebo = 0;
    unsafe {
        gl::GenBuffers(1, &mut ebo);
    }
    debug_assert!(ebo > 0);

    unsafe {
        let null_vertex = mem::zeroed::<Vertex>();
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, 
            (mesh.vertices.len() * mem::size_of::<Vertex>()) as GLsizeiptr, 
            mesh.vertices.as_ptr() as *const GLvoid, 
            gl::STATIC_DRAW
        );
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (mesh.vertex_indices.len() * mem::size_of::<u32>()) as GLsizeiptr,
            mesh.vertex_indices.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW
        );
        gl::VertexAttribPointer(
            0, 
            3, 
            gl::FLOAT, 
            gl::FALSE, 
            mem::size_of::<Vertex>() as GLint, 
            offset_of(&null_vertex, &null_vertex.position) as *const GLvoid
        );
        gl::VertexAttribPointer(
            1, 
            3, 
            gl::FLOAT, 
            gl::FALSE, 
            mem::size_of::<Vertex>() as GLint,
            offset_of(&null_vertex, &null_vertex.normal) as *const GLvoid
        );
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as GLint,
            offset_of(&null_vertex, &null_vertex.tex_coords) as *const GLvoid
        );
        gl::VertexAttribPointer(
            3,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as GLint,
            offset_of(&null_vertex, &null_vertex.tangent) as *const GLvoid
        );
        gl::VertexAttribPointer(
            4,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as GLint,
            offset_of(&null_vertex, &null_vertex.bitangent) as *const GLvoid
        );

        gl::EnableVertexAttribArray(0);
        gl::EnableVertexAttribArray(1);
        gl::EnableVertexAttribArray(2);
        gl::EnableVertexAttribArray(3);
        gl::EnableVertexAttribArray(4);
    }

    (vao, vbo, ebo)
}

fn send_to_gpu_shaders(_context: &mut OpenGLContext, source: &ShaderSource) -> ShaderHandle {
    backend::compile(source).unwrap()
}

/// Initialize the logger.
fn init_logger(log_file: &str) {
    file_logger::init(log_file).expect("Failed to initialize logger.");
}

/// Create and OpenGL context.
fn init_gl(width: u32, height: u32) -> backend::OpenGLContext {
    let context = match backend::start_opengl(width, height) {
        Ok(val) => val,
        Err(e) => {
            panic!("Failed to Initialize OpenGL context. Got error: {}", e);
        }
    };

    context
}

/// The GLFW frame buffer size callback function. This is normally set using 
/// the GLFW `glfwSetFramebufferSizeCallback` function, but instead we explicitly
/// handle window resizing in our state updates on the application side. Run this function 
/// whenever the size of the viewport changes.
fn framebuffer_size_callback(context: &mut OpenGLContext, width: u32, height: u32) {
    context.width = width;
    context.height = height;
    unsafe {
        gl::Viewport(0, 0, width as i32, height as i32);
    }
}

fn process_input(context: &mut OpenGLContext) -> CameraMovement {
    match context.window.get_key(Key::Escape) {
        Action::Press | Action::Repeat => {
            context.window.set_should_close(true);
        }
        _ => {}
    }

    let mut movement = CameraMovement::new();
    match context.window.get_key(Key::A) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::MoveLeft;
        }
        _ => {}
        }
    match context.window.get_key(Key::D) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::MoveRight;
        }
        _ => {}
    }
    match context.window.get_key(Key::Q) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::MoveUp;
        }
        _ => {}
    }
    match context.window.get_key(Key::E) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::MoveDown;
        }
        _ => {}
    }
    match context.window.get_key(Key::W) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::MoveForward;
        }
        _ => {}
    }
    match context.window.get_key(Key::S) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::MoveBackward;
        }
        _ => {}
    }
    match context.window.get_key(Key::Left) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::YawLeft;
        }
        _ => {}
    }
    match context.window.get_key(Key::Right) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::YawRight;
        }
        _ => {}
    }
    match context.window.get_key(Key::Up) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::PitchUp;
        }
        _ => {}
    }
    match context.window.get_key(Key::Down) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::PitchDown;
        }
        _ => {}
    }
    match context.window.get_key(Key::Z) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::RollCounterClockwise;
        }
        _ => {}
    }
    match context.window.get_key(Key::C) {
        Action::Press | Action::Repeat => {
            movement += SimpleCameraMovement::RollClockwise;
        }
        _ => {}
    }

    movement
}

fn main() {
    let model = create_backpack_model();
    let light_mesh = create_box_mesh();
    init_logger("opengl_demo.log");
    info!("BEGIN LOG");
    info!("Model name: \"{}\"", model.name);
    info!("Number of meshes loaded: {}", model.meshes.len());
    info!("Number of textures loaded: {}", model.textures_loaded.len());
    let mut camera = create_camera(SCREEN_WIDTH, SCREEN_HEIGHT);
    let cube_lights= create_cube_lights();
    let dir_light = create_directional_light();
    let mut context = init_gl(SCREEN_WIDTH, SCREEN_HEIGHT);

    //  Load the model data for the cube light shader..
    let mesh_shader_source = create_mesh_shader_source();
    let mesh_shader = send_to_gpu_shaders(&mut context, &mesh_shader_source);

    // Load the lighting cube model.
    let light_shader_source = create_cube_light_shader_source();
    let light_shader = send_to_gpu_shaders(&mut context, &light_shader_source);

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
        gl::ClearBufferfv(gl::COLOR, 0, &CLEAR_COLOR[0] as *const GLfloat);
        gl::ClearBufferfv(gl::DEPTH, 0, &CLEAR_DEPTH[0] as *const GLfloat);
        gl::Viewport(0, 0, context.width as GLint, context.height as GLint);
    }

    while !context.window.should_close() {
        let elapsed_seconds = context.update_timers();
        context.update_fps_counter();
        context.glfw.poll_events();
        let (width, height) = context.window.get_framebuffer_size();
        if (width != context.width as i32) && (height != context.height as i32) {
            camera.update_viewport(width as usize, height as usize);
            framebuffer_size_callback(&mut context, width as u32, height as u32);
        }

        let delta_movement = process_input(&mut context);
        camera.update_movement(delta_movement, elapsed_seconds as f32);

        context.window.swap_buffers();
    }

    info!("END LOG");
}

