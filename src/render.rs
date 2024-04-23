use rustfft::num_complex::{Complex32, ComplexFloat};

use crate::{
    audio::{FFT_SIZE, HALF_FFT_SIZE},
    glrs_renderable,
};

use self::glrs::{GLParam::*, Triangle};

mod glfwrs;
mod glrs;

pub use glfwrs::{Window, Winfo};

pub const NUM_SPECTROGRAM_FRAMES: usize = 512;

pub struct RenderApp {
    render_spectrogram: RenderSpectrogram,
    render_waveline: RenderWaveline,
    render_floatingindicator: RenderFloatingIndicator,

    wave_last: [[f32; FFT_SIZE]; 2],

    frame_n: usize,
}

impl RenderApp {
    pub fn new() -> Self {
        Self {
            render_spectrogram: RenderSpectrogram::new(),
            render_waveline: RenderWaveline::new(),
            render_floatingindicator: RenderFloatingIndicator::new(),

            wave_last: [[0.0; FFT_SIZE]; 2],

            frame_n: 0,
        }
    }

    pub fn draw(&self, winfo: &glfwrs::Winfo) {
        glrs::Rgba::TRANSPARENT.gl_clear_color();

        self.render_spectrogram.render(self.frame_n, winfo);
        self.render_waveline.render();

        if winfo.floating {
            self.render_floatingindicator.render(winfo);
        }
    }

    pub fn set_wave(&mut self, wave: &([[Complex32; FFT_SIZE]; 2], [[f32; FFT_SIZE]; 2])) {
        self.render_waveline.set_wave(wave, &self.wave_last);
        self.wave_last = wave.1;

        {
            self.frame_n += 1;
            if self.frame_n == NUM_SPECTROGRAM_FRAMES {
                self.frame_n = 0;
            }
        }

        self.render_spectrogram.set_wave(self.frame_n, wave);
    }
}

//// Component Renderers ////

const SPECTROGRAM_DISPLAY_VERTS: [Triangle; 2] = [
    [[1.0, 1.0], [-1.0, 1.0], [1.0, -1.0]],
    [[-1.0, -1.0], [1.0, -1.0], [-1.0, 1.0]],
];
glrs_renderable! {
    pub RenderSpectrogram(glrs::TriPosVO<2>) {
        shaders(vert: "./shader/spectrogram.vsh", frag: "./shader/spectrogram.fsh");
        vo(glrs::TriPosVO::new(SPECTROGRAM_DISPLAY_VERTS));
        fn new() {
            Self {
                shaders, vo,
                tex: glrs::GLTexture2d::new(),
                spec_accum: [[0.0; 2]; HALF_FFT_SIZE],
            }
        };

        tex: glrs::GLTexture2d<NUM_SPECTROGRAM_FRAMES, HALF_FFT_SIZE>,
        spec_accum: [[f64; 2]; HALF_FFT_SIZE],
    }
}
impl RenderSpectrogram {
    pub fn render(&self, frame_n: usize, winfo: &glfwrs::Winfo) {
        self.bind();
        self.tex.bind(glrs::GLTextureSlot::Tex0, 1);
        glrs::uniform(2, V1F(frame_n as f32 / NUM_SPECTROGRAM_FRAMES as f32));
        glrs::uniform(3, V1F(winfo.bounds.dim.1 as f32));
        glrs::DrawArrays::Triangles { range: 0..2 }.exec();
    }
    pub fn set_wave(
        &mut self,
        frame_n: usize,
        wave: &([[Complex32; FFT_SIZE]; 2], [[f32; FFT_SIZE]; 2]),
    ) {
        let mut ds = [glrs::Rgba::default(); FFT_SIZE / 2];
        for i in 0..FFT_SIZE / 2 {
            // for j in 0..2 {
            //     let cur = wave.0[j][i].abs() as f64;
            //     self.spec_accum[i][j] = cur + 0.5 * (self.spec_accum[i][j] - cur);
            // }
            // let c0 = self.spec_accum[i][0].log10() as f32 / 100.0 + 0.5;
            // let c1 = self.spec_accum[i][1].log10() as f32 / 100.0 + 0.5;
            let mut c = [0.0; 2];
            for j in 0..2 {
                let cur = wave.0[j][i].abs() as f64;
                self.spec_accum[i][j] = cur + 0.5 * (self.spec_accum[i][j] - cur);
                c[j] = self.spec_accum[i][j].log10() as f32 / 100.0 + 0.5;
                // if i == 200 && j == 0 {
                //     dbg!(c[j]);
                // }
            }
            ds[i] = glrs::Rgba {
                r: ((c[0] * 256.0) % 256.0) as u8,
                g: ((c[0] * 65536.0) % 256.0) as u8,
                b: ((c[1] * 256.0) % 256.0) as u8,
                a: ((c[1] * 65536.0) % 256.0) as u8,
            };
        }
        if frame_n == NUM_SPECTROGRAM_FRAMES - 1 {
            let mut d = [[glrs::Rgba::default(); 1]; FFT_SIZE / 2];
            for i in 0..FFT_SIZE / 2 {
                d[i] = [ds[i]; 1];
            }
            self.tex.update_partial(frame_n, 0, d);
            self.tex.update_partial(0, 0, d);
        } else {
            let mut d = [[glrs::Rgba::default(); 2]; FFT_SIZE / 2];
            for i in 0..FFT_SIZE / 2 {
                d[i] = [ds[i]; 2];
            }
            self.tex.update_partial(frame_n, 0, d);
        }
    }
}

const WAVE_VO_SIZE: usize = 5 * FFT_SIZE;
glrs_renderable! {
    pub RenderWaveline(glrs::F32VO<WAVE_VO_SIZE, 2>) {
        shaders(vert: "./shader/waveline.vsh", frag: "./shader/waveline.fsh");
        vo(glrs::F32VO::new([[0.0; 2]; WAVE_VO_SIZE]));
        fn new() {
            Self {
                shaders, vo,
                wave_x_off: 0,
            }
        };

        wave_x_off: i32,
    }
}
impl RenderWaveline {
    pub fn render(&self) {
        self.bind();
        glrs::uniform(1, V1F(self.wave_x_off as f32 / FFT_SIZE as f32));
        for j in 0..2 {
            glrs::DrawArrays::LineStrip {
                range: (j * FFT_SIZE as i32)..((j + 1) * FFT_SIZE as i32),
                line_width: 1.0,
                point_size: 1.0,
            }
            .exec();
        }
    }
    pub fn set_wave(
        &mut self,
        wave: &([[Complex32; FFT_SIZE]; 2], [[f32; FFT_SIZE]; 2]),
        wave_last: &[[f32; FFT_SIZE]; 2],
    ) {
        for i in 0..FFT_SIZE {
            let k = i as f32 / FFT_SIZE as f32;
            let x = k * 2.0 - 1.0;
            for ch in 0..2 {
                self.vo.data[i + ch * FFT_SIZE] = [x, wave.1[ch][i]];
                self.vo.data[i + (ch + 2) * FFT_SIZE] = [
                    ((k + 0.5 / FFT_SIZE as f32).ln() * 0.2 + 1.0) * 2.0 - 1.0,
                    // k,
                    wave.0[ch][i / 2].abs().log10() / 5.0,
                ];
            }
            self.vo.data[i + 4 * FFT_SIZE] = [wave.1[0][i], wave.1[1][i]];
        }
        {
            let last_off = self.wave_x_off;

            let mut min_mse = f32::INFINITY;
            let mut best_off = 0;
            for off in (-512..512).step_by(2) {
                let total_off = off - last_off;
                let bounds = (600.max(-total_off), 900.min(FFT_SIZE as i32 - total_off));
                let mut mse = 0.0;
                for i in (bounds.0..bounds.1).step_by(1) {
                    mse +=
                        (wave.1[0][(i + total_off) as usize] - wave_last[0][i as usize]).powf(2.0);
                }
                mse /= (bounds.1 - bounds.0) as f32;

                if mse < min_mse {
                    min_mse = mse;
                    best_off = off;
                }
            }
            self.wave_x_off = best_off;
        }
        self.vo.update();
    }
}

const FLOATING_INDICATOR_VERTS: [Triangle; 1] = [[[1.0, -1.0], [0.9, -1.0], [1.0, -0.9]]];
glrs_renderable! {
    pub RenderFloatingIndicator(glrs::TriPosVO<1>) {
        shaders(vert: "./shader/floating_indicator.vsh", frag: "./shader/floating_indicator.fsh");
        vo(glrs::TriPosVO::new(FLOATING_INDICATOR_VERTS));
        fn new() {
            Self {
                shaders, vo,
            }
        };
    }
}
impl RenderFloatingIndicator {
    pub fn render(&self, winfo: &glfwrs::Winfo) {
        self.bind();
        glrs::uniform(1, V1F(winfo.bounds.aspect()));
        glrs::TransparencyMode::Replace.apply();
        glrs::DrawArrays::Triangles { range: 0..1 }.exec();
        glrs::TransparencyMode::Normal.apply();
    }
}
