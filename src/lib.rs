#![allow(unused_imports)]

use core::convert::{TryFrom, TryInto};
use ogl33::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    // Array Buffers holds arrays of vertex data for drawing.
    Array = GL_ARRAY_BUFFER as isize,
    // Element Array Buffers hold indexes of what vertexes to use for drawing.
    ElementArray = GL_ELEMENT_ARRAY_BUFFER as isize,
}


pub enum ShaderType {
    // Vertex shaders determine the position of geometry within the screen.
    Vertex = GL_VERTEX_SHADER as isize,
    // Fragment shaders determine the color output of geometry.
    //
    // Also other values, but mostly color.
    Fragment = GL_FRAGMENT_SHADER as isize,
}

pub struct Shader(pub GLuint);

pub struct VertexArray(pub GLuint);

pub struct VertexBuffer(pub GLuint);

pub struct ShaderProgram(pub GLuint, Vec<u32>);

impl VertexArray {
    // Creates new vertex array object
    pub fn new() -> Option<Self> {
        let mut vertex_array_object = 0;
        unsafe {
            // glGenVertexArrays() fills the vertex_array_object with names of the new VAOs
            // THe first input is the length of the vao (or number of VAOs)
            glGenVertexArrays(1, &mut vertex_array_object);
        }

        // The vertex array object should never be 0 after glGenVertexArrays() has been called
        // Check that this has not happened
        if vertex_array_object != 0 {
            Some(Self(vertex_array_object))
        } else {
            None
        }
    }

    // Binds the vao to make it the active VAO. Which means that all GL functions will now do whatever they do to the currently active VAO
    pub fn bind(&self) {
        unsafe { glBindVertexArray(self.0) }
    }

    pub fn clear_binding() {
        unsafe { glBindVertexArray(0) }
    }
}

impl VertexBuffer {
    // Creates new vertex buffer object
    pub fn new() -> Option<Self> {
        let mut vertex_buffer_object = 0;
        unsafe {
            // Fills the vertec_buffer_object
            // Give it a length and a buffer and it fills it
            // Can be used for all buffers
            glGenBuffers(1, &mut vertex_buffer_object);
        }
        // Should still not be 0
        if vertex_buffer_object != 0 {
            Some(Self(vertex_buffer_object))
        } else {
            None
        }
    }

    // Bind this vertex buffer for the given type
    // Binds the buffer to the binding target
    // In this case we bind the VBO to the GL_ARRAY_BUFFER
    pub fn bind(&self, ty: BufferType) {
        unsafe { glBindBuffer(ty as GLenum, self.0) }
    }

    pub fn clear_bind(ty: BufferType) {
        unsafe { glBindBuffer(ty as GLenum, 0) }
    }
}

impl Shader {
    // Makes a new shader.
    //
    // Prefer the [`Shader::from_source`](Shader::from_source) method.
    //
    // Possibly skip the direct creation of the shader object and use
    // [`ShaderProgram::from_vert_frag`](ShaderProgram::from_vert_frag).
    pub fn new(ty: ShaderType) -> Option<Self> {
        let shader = unsafe { glCreateShader(ty as GLenum) };
        if shader != 0 {
            Some(Self(shader))
        } else {
            None
        }
    }

    // Assigns a source string to the shader.
    //
    // Replaces any previously assigned source.
    pub fn set_source(&self, src: &str) {
        unsafe {
            glShaderSource(
                self.0,
                1,
                &(src.as_bytes().as_ptr().cast()),
                &(src.len().try_into().unwrap()),
            );
        }
    }

    // Compiles the shader based on the current source.
    pub fn compile(&self) {
        unsafe { glCompileShader(self.0) };
    }

    pub fn compile_success(&self) -> bool {
        let mut compiled = 0;
        unsafe { glGetShaderiv(self.0, GL_COMPILE_STATUS, &mut compiled) };
        compiled == i32::from(GL_TRUE)
    }

    pub fn info_log(&self) -> String {
        let mut needed_len = 0;
        unsafe { glGetShaderiv(self.0, GL_INFO_LOG_LENGTH, &mut needed_len) };
        let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;
        unsafe {
            glGetShaderInfoLog(
                self.0,
                v.capacity().try_into().unwrap(),
                &mut len_written,
                v.as_mut_ptr().cast(),
            );
            v.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&v).into_owned()
    }

    // Marks a shader for deletion.
    //
    // Note: This _does not_ immediately delete the shader. It only marks it for
    // deletion. If the shader has been previously attached to a program then the
    // shader will stay allocated until it's unattached from that program.
    pub fn delete(self) {
        unsafe { glDeleteShader(self.0) };
    }

    /// Takes a shader type and source string and produces either the compiled
    /// shader or an error message.
    ///
    /// Prefer [`ShaderProgram::from_vert_frag`](ShaderProgram::from_vert_frag),
    /// it makes a complete program from the vertex and fragment sources all at
    /// once.
    pub fn from_source(ty: ShaderType, source: &str) -> Result<Self, String> {
        let id = Self::new(ty).ok_or_else(|| "Could not allocate new shader".to_string())?;
        id.set_source(source);
        id.compile();
        if id.compile_success() {
            Ok(id)
        } else {
            let out = id.info_log();
            id.delete();
            Err(out)
        }
    }
}

impl ShaderProgram {
    // Allocates a new program object.
    //
    // Prefer [`ShaderProgram::from_vert_frag`](ShaderProgram::from_vert_frag),
    // it makes a complete program from the vertex and fragment sources all at
    // once.
    pub fn new() -> Option<Self> {
        let prog = unsafe { glCreateProgram() };
        if prog != 0 {
            Some(Self(prog,vec![]))
        } else {
            None
        }
    }

    // Attaches a shader object to this program object.
    pub fn attach_shader(&self, shader: &Shader) {
        unsafe { glAttachShader(self.0, shader.0) };
    }

    // Links the various attached, compiled shader objects into a usable program.
    pub fn link_program(&self) {
        unsafe { glLinkProgram(self.0) };
    }

    pub fn get_shader(&self, shader: &str) -> Option<u32>{
        let index = self.1.iter().position(|&x| x == shader.hash()).unwrap_or_else(|| usize::MAX);
        if index != usize::MAX {
            Some(self.1[(index + 1) as usize])
        }else{
            None
        }
    }

    pub fn link_success(&self) -> bool {
        let mut success = 0;
        unsafe { glGetProgramiv(self.0, GL_LINK_STATUS, &mut success) };
        success == i32::from(GL_TRUE)
    }

    // Gets the log data for this program.
    //
    // This is usually used to check the message when a program failed to link.
    pub fn info_log(&self) -> String {
        let mut needed_len = 0;
        unsafe { glGetShaderiv(self.0, GL_INFO_LOG_LENGTH, &mut needed_len) };
        let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;
        unsafe {
            glGetShaderInfoLog(
                self.0,
                v.capacity().try_into().unwrap(),
                &mut len_written,
                v.as_mut_ptr().cast(),
            );
            v.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&v).into_owned()
    }

    // Marks the program for deletion.
    //
    // Note: This _does not_ immediately delete the program. If the program is
    // currently in use it won't be deleted until it's not the active program.
    // When a program is finally deleted and attached shaders are unattached.
    pub fn delete(self) {
        unsafe { glDeleteProgram(self.0) };
    }

    pub fn use_program(&self) {
        unsafe { glUseProgram(self.0) };
    }


    // Takes a vertex shader source string and a fragment shader source string
    // and either gets you a working program object or gets you an error message.
    //
    // This is the preferred way to create a simple shader program in the common
    // case. It's just less error prone than doing all the steps yourself.
    pub fn from_vert_frag(vert: &str, frag: &str) -> Result<Self, String> {
        let mut p = Self::new().ok_or_else(|| "Could not allocate a program".to_string())?;
        let v = Shader::from_source(ShaderType::Vertex, vert)
            .map_err(|e| format!("Vertex Compile Error: {}", e))?;
        let f = Shader::from_source(ShaderType::Fragment, frag)
            .map_err(|e| format!("Fragment Compile Error: {}", e))?;

        p.attach_shader(&v);
        //TODO:
        // Gör hash funktionen mer effektiv
        // Just nu hashar den hela shadern (alltså koden)
        // Kan lösas genom att endast namnge shadern i en kommentar
        // Och göra en smart hashShader funktion
        p.1.push(vert.hash());
        p.1.push(v.0);
        p.attach_shader(&f);
        p.1.push(frag.hash());
        p.1.push(f.0);

        p.link_program();
        v.delete();
        f.delete();
        if p.link_success() {
            Ok(p)
        } else {
            let out = format!("Program Link Error: {}", p.info_log());
            p.delete();
            Err(out)
        }
    }
}

// Sends data
pub fn buffer_data(ty: BufferType, data: &[u8], usage: GLenum) {
    unsafe {
        glBufferData(
            // Specifies the binding target we want to buffer to
            ty as GLenum,
            // The number of bytes we want to buffer(send)
            data.len().try_into().unwrap(),
            // The pointer to the start of the data
            data.as_ptr().cast(),
            // The usage hint
            // Some tasks are easier for the GPU other for the CPU
            // If we hint how the data will be used then GL will be able to make a smart choice
            // In this case we want GL_STATIC_DRAW "since we'll just be sending the data once, and then GL will draw with it many times."
            usage,
        );
    }
}

pub fn buffer_sub_data(ty:BufferType, data: &[u8], offset: usize){
    unsafe{
        glBufferSubData(
            ty as GLenum,
            offset.try_into().unwrap(),
            data.len().try_into().unwrap(),
            data.as_ptr().cast(),
        );
    }
}

pub fn clear_color(r: f32, g: f32, b: f32, a: f32) {
    unsafe { glClearColor(r, g, b, a) }
}

pub trait Hashable {
    fn hash(&self) -> u32;
    fn hashShader(&self) -> u32;
}

impl Hashable for str {
    //sdbm hash-encoding for a given string
    fn hash(&self) -> u32 {
        let mut hash: u32 = 0;

        //self.chars().for_each(|c| print!("{}", c));

        for _c in self.encode_utf16() {
            let mut c = _c.to_string();
            hash = u32::from(_c)
                .wrapping_add(hash << 6)
                .wrapping_add(hash << 16)
                .wrapping_sub(hash);
        }

        hash
    }

    fn hashShader(&self) -> u32{

        let mut hash: u32 = 0;
        for _c in self.encode_utf16() {
            if _c == 35{
                break;
            }
            hash = u32::from(_c)
                .wrapping_add(hash << 6)
                .wrapping_add(hash << 16)
                .wrapping_sub(hash);
        }
        hash
    }

}


/// WIREFRAME
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolygonMode{
    ///Just show the points
    Point = GL_POINT as isize,
    /// Just show the lines
    Line = GL_LINE as isize,
    /// Fill the polygons
    Fill = GL_FILL as isize,
}

pub fn polygon_mode(mode: PolygonMode){
    unsafe{
        glPolygonMode(GL_FRONT_AND_BACK, mode as GLenum)
    };
}

