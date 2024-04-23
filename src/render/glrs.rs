use std::{
    ffi::{c_void, CString},
    fmt::Display,
    ops::Range,
};

use gl::types::{GLchar, GLenum, GLint, GLuint, GLvoid};

use crate::util::{RectI, Vec2I};

#[derive(Debug, Clone, Copy)]
pub enum GLShaderType {
    Fragment,
    Vertex,
}
impl GLShaderType {
    fn gl_value(&self) -> GLuint {
        match self {
            Self::Fragment => gl::FRAGMENT_SHADER,
            Self::Vertex => gl::VERTEX_SHADER,
        }
    }
    fn name(&self) -> &str {
        match self {
            Self::Fragment => "Fragment",
            Self::Vertex => "Vertex",
        }
    }
}

#[derive(Debug)]
pub enum GLError {
    ShaderCompilation { typ: GLShaderType, message: String },
    ShaderLinking { message: String },
}
impl Display for GLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ !!!!!!! ] ")?;
        match self {
            GLError::ShaderCompilation { typ, message } => {
                write!(f, "Shader/{} compile failed:\n{}", typ.name(), message)
            }
            GLError::ShaderLinking { message } => write!(f, "Shader link failed:\n{}", message),
        }
    }
}
impl std::error::Error for GLError {}

pub struct GLShader {
    sources: Vec<CString>,
    typ: GLShaderType,
    ref_id: u32,
}
impl GLShader {
    pub fn load(typ: GLShaderType, source: &str) -> Result<Self, GLError> {
        let mut shader = Self::new(typ);
        shader.source(source);
        shader.compile()?;
        Ok(shader)
    }
    pub fn new(typ: GLShaderType) -> Self {
        Self {
            sources: vec![],
            ref_id: unsafe { gl::CreateShader(typ.gl_value()) },
            typ,
        }
    }
    pub fn source(&mut self, source: &str) {
        unsafe {
            self.sources.push(CString::new(source).unwrap());
            gl::ShaderSource(
                self.ref_id,
                1,
                &self.sources.last().unwrap().as_ptr(),
                std::ptr::null(),
            );
        }
    }
    pub fn compile(&mut self) -> Result<(), GLError> {
        unsafe {
            gl::CompileShader(self.ref_id);

            let mut success: GLint = 0;
            gl::GetShaderiv(self.ref_id, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut len: GLint = 0;
                gl::GetShaderiv(self.ref_id, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer: Vec<u8> = Vec::with_capacity(len as usize);
                gl::GetShaderInfoLog(
                    self.ref_id,
                    len,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                buffer.set_len((len as usize).saturating_sub(1));
                Err(GLError::ShaderCompilation {
                    typ: self.typ,
                    message: std::str::from_utf8(&buffer).unwrap().to_string(),
                })
            } else {
                Ok(())
            }
        }
    }
}
impl Drop for GLShader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.ref_id) }
    }
}

struct GLShaderProgram {
    ref_id: u32,
}
impl GLShaderProgram {
    fn new() -> Self {
        Self {
            ref_id: unsafe { gl::CreateProgram() },
        }
    }
}
impl Drop for GLShaderProgram {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.ref_id) }
    }
}
pub struct GLShaderProgramBuilder {
    obj: GLShaderProgram,
}
impl GLShaderProgramBuilder {
    pub fn new() -> Self {
        Self {
            obj: GLShaderProgram::new(),
        }
    }
    pub fn attatch_shader(&self, shader: &GLShader) {
        unsafe { gl::AttachShader(self.obj.ref_id, shader.ref_id) }
    }
    pub fn link(self) -> Result<GLShaderProgramLinked, GLError> {
        unsafe {
            gl::LinkProgram(self.obj.ref_id);

            let mut success: GLint = 0;
            gl::GetProgramiv(self.obj.ref_id, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut len: GLint = 0;
                gl::GetProgramiv(self.obj.ref_id, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer: Vec<u8> = Vec::with_capacity(len as usize);
                gl::GetProgramInfoLog(
                    self.obj.ref_id,
                    len,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                buffer.set_len((len as usize).saturating_sub(1));
                Err(GLError::ShaderLinking {
                    message: std::str::from_utf8(&buffer).unwrap().to_string(),
                })
            } else {
                Ok(GLShaderProgramLinked { obj: self.obj })
            }
        }
    }
}
pub struct GLShaderProgramLinked {
    obj: GLShaderProgram,
}
impl GLShaderProgramLinked {
    pub fn use_for_draw(&self) {
        unsafe { gl::UseProgram(self.obj.ref_id) }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rgba<T> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}
impl Rgba<f32> {
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub fn gl_clear_color(&self) {
        let Rgba { r, g, b, a } = *self;
        unsafe {
            gl::ClearColor(r, g, b, a);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}

pub struct GLTexture2d<const W: usize, const H: usize> {
    ref_id: GLuint,
}
impl<const W: usize, const H: usize> GLTexture2d<W, H> {
    pub fn new() -> Self {
        let data: Box<[[Rgba<u8>; W]; H]> = vec![
            [Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            }; W];
            H
        ]
        .try_into()
        .unwrap();

        let mut ref_id = 0;

        unsafe {
            gl::GenTextures(1, &mut ref_id);
            gl::BindTexture(gl::TEXTURE_2D, ref_id);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        }

        let this = Self { ref_id };
        unsafe {
            this.gl_tex_image_2d(data.as_ptr() as *const GLvoid);
        }
        this
    }
    unsafe fn gl_tex_image_2d(&self, pixels: *const GLvoid) {
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as GLint,
            W as GLint,
            H as GLint,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            pixels,
        );
    }
    pub fn bind(&self, slot: GLTextureSlot, location: GLint) {
        unsafe {
            gl::ActiveTexture(slot.gl_enum());
            gl::BindTexture(gl::TEXTURE_2D, self.ref_id);
            gl::Uniform1i(location, slot.gl_int());
        }
    }
    pub fn update_partial<const WS: usize, const HS: usize>(
        &self,
        x0: usize,
        y0: usize,
        data: [[Rgba<u8>; WS]; HS],
    ) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.ref_id);
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                x0 as GLint,
                y0 as GLint,
                WS as GLint,
                HS as GLint,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const GLvoid,
            )
        }
    }
}
impl<const W: usize, const H: usize> Drop for GLTexture2d<W, H> {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.ref_id) }
    }
}
pub enum GLTextureSlot {
    Tex0,
}
impl GLTextureSlot {
    fn gl_enum(&self) -> GLenum {
        match self {
            Self::Tex0 => gl::TEXTURE0,
        }
    }
    fn gl_int(&self) -> GLint {
        match self {
            Self::Tex0 => 0,
        }
    }
}

pub type Triangle = [[f32; 2]; 3];

/// A vertex object containing `N` 2D [`f32`] triangles
pub struct TriPosVO<const N: usize> {
    tris: [Triangle; N],
    vbo: gl::types::GLuint,
    vao: gl::types::GLuint,
}
impl<const N: usize> TriPosVO<N> {
    pub fn new(tris: [Triangle; N]) -> Self {
        let mut self_ = Self {
            vbo: 0,
            vao: 0,
            tris,
        };
        // Create and bind the vertex buffer
        unsafe {
            gl::GenBuffers(1, &mut self_.vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, self_.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (N * (3 * 2) * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
                &self_.tris[0][0][0] as *const f32 as *const c_void,
                gl::STATIC_DRAW,
            );
        }

        // Create and bind the vertex array object
        unsafe {
            gl::GenVertexArrays(1, &mut self_.vao);
            gl::BindVertexArray(self_.vao);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                (2 * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(0);
        }

        self_
    }
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
        }
    }
}
impl<const N: usize> Drop for TriPosVO<N> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

/// A vertex object containing `N` `S` dimensional [`f32`] vectors
pub struct F32VO<const N: usize, const S: usize> {
    vbo: gl::types::GLuint,
    vao: gl::types::GLuint,
    pub data: [[f32; S]; N],
}
impl<const L: usize, const S: usize> F32VO<L, S> {
    pub fn new(data: [[f32; S]; L]) -> Self {
        let mut self_ = Self {
            vbo: 0,
            vao: 0,
            data,
        };
        // Create and bind the vertex buffer
        unsafe {
            gl::GenBuffers(1, &mut self_.vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, self_.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (L * S * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
                &data[0] as *const f32 as *const c_void,
                gl::STATIC_DRAW,
            );
        }

        // Create and bind the vertex array object
        unsafe {
            gl::GenVertexArrays(1, &mut self_.vao);
            gl::BindVertexArray(self_.vao);
            gl::VertexAttribPointer(
                0,
                S as gl::types::GLint,
                gl::FLOAT,
                gl::FALSE,
                (S * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(0);
        }

        self_
    }
    pub fn update(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (L * S * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
                &self.data[0] as *const f32 as *const c_void,
                gl::STATIC_DRAW,
            );
        }
    }
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::EnableVertexAttribArray(0);
        }
    }
}
impl<const L: usize, const S: usize> Drop for F32VO<L, S> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

#[macro_export]
macro_rules! glrs_renderable {
    (
        $vis:vis $struct_name:ident (
            $vo_ty:ty
        ) {
            // shaders (vert: $sh_vert:expr, frag: $sh_frag:expr $(,)?) ;
            // vo ( $new_vo:expr );
            // fn $new_fn:ident ( $( $newarg:ident : $newarg_ty:ty ),* $(,)? );
            // $( $field:ident : $field_ty:ty ),* $(,)?

            $shaders:ident (vert: $sh_vert:expr, frag: $sh_frag:expr $(,)?) ;
            $vo:ident ( $new_vo:expr );
            fn new ( $( $newarg:ident : $newarg_ty:ty ),* $(,)? ) $new_fn:block ;
            $( $field_vis:vis $field:ident : $field_ty:ty ),* $(,)?
        }
    ) => {
        $vis struct $struct_name {
            vo: $vo_ty,
            shaders: $crate::render::glrs::GLShaderProgramLinked,
            $( $field_vis $field: $field_ty, )*
        }
        impl $struct_name {
            pub fn new(
                $( $newarg : $newarg_ty, )*
            ) -> Self {
                use $crate::render::glrs;
                let $shaders = {
                    let builder = glrs::GLShaderProgramBuilder::new();
                    let vert = glrs::GLShader::load(
                        glrs::GLShaderType::Vertex,
                        include_str!($sh_vert),
                    )
                    .unwrap();
                    let frag = glrs::GLShader::load(
                        glrs::GLShaderType::Fragment,
                        include_str!($sh_frag),
                    )
                    .unwrap();

                    builder.attatch_shader(&vert);
                    builder.attatch_shader(&frag);

                    builder.link().unwrap()
                };
                let $vo = $new_vo;

                // Self::$new_fn ( shaders, vo, $( $newarg : $newarg_ty, )* )
                $new_fn
            }
            fn bind(&self) {
                self.shaders.use_for_draw();
                self.vo.bind();
            }
        }
    };
}

pub enum DrawArrays<
    T: Into<gl::types::GLint> + std::ops::Sub<Output = V> + Copy,
    V: Into<gl::types::GLsizei>,
> {
    Triangles {
        range: Range<T>,
    },
    LineStrip {
        range: Range<T>,
        line_width: gl::types::GLfloat,
        point_size: gl::types::GLfloat,
    },
}
impl<T: Into<gl::types::GLint> + std::ops::Sub<Output = V> + Copy, V: Into<gl::types::GLsizei>>
    DrawArrays<T, V>
{
    pub fn exec(self) {
        let (range, mode, scale) = match self {
            Self::Triangles { range } => (range, gl::TRIANGLES, 3),
            Self::LineStrip {
                range,
                line_width,
                point_size,
            } => {
                unsafe {
                    gl::LineWidth(line_width);
                    gl::PointSize(point_size);
                }
                (range, gl::LINE_STRIP, 1)
            }
        };
        unsafe {
            gl::DrawArrays(
                mode,
                range.start.into(),
                (range.end - range.start).into() * scale,
            )
        }
    }
}

pub enum TransparencyMode {
    Normal,
    Replace,
}
impl TransparencyMode {
    pub fn apply(&self) {
        unsafe {
            match self {
                Self::Normal => {
                    gl::BlendEquation(gl::FUNC_ADD);
                    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                }
                Self::Replace => {
                    gl::BlendEquation(gl::FUNC_ADD);
                    gl::BlendFunc(gl::SRC_ALPHA, gl::ZERO);
                }
            }
        }
    }
}

pub enum GLParam {
    V1F(gl::types::GLfloat),
}
#[inline]
pub fn uniform(location: gl::types::GLint, value: GLParam) {
    unsafe {
        match value {
            GLParam::V1F(v) => gl::Uniform1f(location, v),
        }
    }
}

pub fn viewport(
    RectI {
        pos: Vec2I(x, y),
        dim: Vec2I(width, height),
    }: RectI,
) {
    unsafe {
        gl::Viewport(x, y, width, height);
    }
}
