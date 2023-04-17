use std::ffi::c_void;

use rustfft::num_complex::{Complex32, ComplexFloat};

use crate::audio::{FFT_SIZE, HALF_FFT_SIZE};

mod glrs;

struct TriPosVO<const N: usize> {
    tris: [[[f32; 2]; 3]; N],
    vbo: gl::types::GLuint,
    vao: gl::types::GLuint,
}
impl<const N: usize> TriPosVO<N> {
    fn new(tris: [[[f32; 2]; 3]; N]) -> Self {
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
    fn bind(&self) {
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

pub struct F32VO<const N: usize, const S: usize> {
    vbo: gl::types::GLuint,
    vao: gl::types::GLuint,
    pub data: [[f32; S]; N],
}
impl<const L: usize, const S: usize> F32VO<L, S> {
    fn new(data: [[f32; S]; L]) -> Self {
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
    fn update(&self) {
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
    fn bind(&self) {
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

const SPECTROGRAM_VERTS: [[[f32; 2]; 3]; 2] = [
    [[1.0, 1.0], [-1.0, 1.0], [1.0, -1.0]],
    [[-1.0, -1.0], [1.0, -1.0], [-1.0, 1.0]],
];
const WAVE_VO_SIZE: usize = 5 * FFT_SIZE;
const SPEC_WIDTH: usize = 512;
pub struct H {
    spectrogram_vo: TriPosVO<2>,
    spectrogram_shaders: glrs::GLShaderProgramLinked,
    spectrogram_tex: glrs::GLTexture2d<512, HALF_FFT_SIZE>,
    waveline_vo: F32VO<WAVE_VO_SIZE, 2>,
    waveline_shaders: glrs::GLShaderProgramLinked,

    n: usize,
    wave_x_off: i32,

    spec_accum: [[f64;2]; HALF_FFT_SIZE],
    wave_last: [[f32; FFT_SIZE]; 2],

    pub window_height: i32,
}

impl H {
    pub fn new(window_height: i32) -> Self {
        let spectrogram_shaders = glrs::GLShaderProgramBuilder::new();
        let spectrogram_shaders = {
            let vert = glrs::GLShader::load(
                glrs::GLShaderType::Vertex,
                include_str!("shader/spectrogram.vsh"),
            )
            .unwrap();
            let frag = glrs::GLShader::load(
                glrs::GLShaderType::Fragment,
                include_str!("shader/spectrogram.fsh"),
            )
            .unwrap();

            spectrogram_shaders.attatch_shader(&vert);
            spectrogram_shaders.attatch_shader(&frag);

            spectrogram_shaders.link().unwrap()
        };
        let spectrogram_vo = TriPosVO::new(SPECTROGRAM_VERTS);

        let waveline_shaders = glrs::GLShaderProgramBuilder::new();
        let waveline_shaders = {
            let vert = glrs::GLShader::load(
                glrs::GLShaderType::Vertex,
                include_str!("shader/waveline.vsh"),
            )
            .unwrap();
            let frag = glrs::GLShader::load(
                glrs::GLShaderType::Fragment,
                include_str!("shader/waveline.fsh"),
            )
            .unwrap();

            waveline_shaders.attatch_shader(&vert);
            waveline_shaders.attatch_shader(&frag);

            waveline_shaders.link().unwrap()
        };
        let waveline_vo = F32VO::new([[0.0; 2]; WAVE_VO_SIZE]);

        let spectrogram_tex = glrs::GLTexture2d::new();

        Self {
            spectrogram_shaders,
            spectrogram_vo,
            spectrogram_tex,
            waveline_vo,
            waveline_shaders,
            n: 0,
            wave_x_off: 0,
            spec_accum: [[0.0;2];HALF_FFT_SIZE],
            wave_last: [[0.0; FFT_SIZE]; 2],

            window_height,
        }
    }

    pub unsafe fn draw(&self) {
        gl::Clear(gl::COLOR_BUFFER_BIT);

        self.spectrogram_shaders.use_for_draw();
        self.spectrogram_vo.bind();
        self.spectrogram_tex.bind(glrs::GLTextureSlot::Tex0, 1);
        gl::Uniform1f(2, self.n as f32 / SPEC_WIDTH as f32);
        gl::Uniform1f(3, self.window_height as f32);
        gl::DrawArrays(gl::TRIANGLES, 0, 3 * SPECTROGRAM_VERTS.len() as i32);

        self.waveline_shaders.use_for_draw();
        self.waveline_vo.update();
        self.waveline_vo.bind();
        gl::Uniform1f(1, self.wave_x_off as f32 / FFT_SIZE as f32);
        gl::LineWidth(1.0);
        gl::PointSize(1.0);
        for j in 0..2 {
            gl::DrawArrays(gl::LINE_STRIP, j * FFT_SIZE as i32, FFT_SIZE as i32);
        }
        // gl::PointSize(1.0);
        // gl::LineWidth(4.0);
        // gl::DrawArrays(gl::LINE_STRIP, 4 * FFT_SIZE as i32, FFT_SIZE as i32);
    }

    pub fn set_wave(&mut self, wave: &([[Complex32; FFT_SIZE]; 2], [[f32; FFT_SIZE]; 2])) {
        for i in 0..FFT_SIZE {
            let k = i as f32 / FFT_SIZE as f32;
            let x = k * 2.0 - 1.0;
            for ch in 0..2 {
                self.waveline_vo.data[i + ch * FFT_SIZE] = [x, wave.1[ch][i]];
                self.waveline_vo.data[i + (ch + 2) * FFT_SIZE] = [
                    ((k + 0.5 / FFT_SIZE as f32).ln() * 0.2 + 1.0) * 2.0 - 1.0,
                    // k,
                    wave.0[ch][i / 2].abs().log10() / 5.0,
                ];
            }
            self.waveline_vo.data[i + 4 * FFT_SIZE] = [wave.1[0][i], wave.1[1][i]];
        }
        {
            let last_off = self.wave_x_off;

            let mut min_mse = f32::INFINITY;
            let mut best_off = 0;
            for off in (-512 .. 512).step_by(2) {
                let total_off = off - last_off;
                let bounds = (600.max(-total_off), 900.min(FFT_SIZE as i32 - total_off));
                let mut mse = 0.0;
                for i in (bounds.0 .. bounds.1).step_by(1) {
                    mse += (wave.1[0][(i + total_off) as usize] - self.wave_last[0][i as usize]).powf(2.0);
                }
                mse /= (bounds.1 - bounds.0) as f32;

                if mse < min_mse {
                    min_mse = mse;
                    best_off = off;
                }
            }
            self.wave_x_off = best_off;
        }
        self.wave_last = wave.1;
        
        self.n += 1;
        if self.n == SPEC_WIDTH {
            self.n = 0;
        }
        let mut ds = [glrs::GLTexPixel::default(); FFT_SIZE / 2];
        for i in 0..FFT_SIZE / 2 {
            for j in 0 .. 2 {
                let cur = wave.0[j][i].abs() as f64;
                self.spec_accum[i][j] = cur + 0.5 * (self.spec_accum[i][j] - cur);
            }
            let c0 = self.spec_accum[i][0].log10() as f32 / 100.0 + 0.5;
            let c1 = self.spec_accum[i][1].log10() as f32 / 100.0 + 0.5;
            ds[i] = glrs::GLTexPixel {
                r: ((c0 * 256.0) % 256.0) as u8,
                g: ((c0 * 65536.0) % 256.0) as u8,
                b: ((c1 * 256.0) % 256.0) as u8,
                a: ((c1 * 65536.0) % 256.0) as u8,
            };
        }
        if self.n == SPEC_WIDTH - 1  {
            let mut d = [[glrs::GLTexPixel::default(); 1]; FFT_SIZE / 2];
            for i in 0..FFT_SIZE / 2 {
                d[i] = [ds[i]; 1];
            }
            self.spectrogram_tex.update_partial(self.n, 0, d);
            self.spectrogram_tex.update_partial(0, 0, d);
        } else {
            let mut d = [[glrs::GLTexPixel::default(); 2]; FFT_SIZE / 2];
            for i in 0..FFT_SIZE / 2 {
                d[i] = [ds[i]; 2];
            }
            self.spectrogram_tex.update_partial(self.n, 0, d);
        }
    }
}
