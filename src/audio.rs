use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Stream, StreamError,
};
use rustfft::{num_complex::Complex32, num_traits::Zero, FftPlanner};

pub const FFT_SIZE: usize = 4096;
pub const FFT_STRIDE: usize = 1024;
pub const HALF_FFT_SIZE: usize = FFT_SIZE / 2;

pub type AudioDataChunk = Box<([[Complex32; FFT_SIZE]; 2], [[f32; FFT_SIZE]; 2])>;

pub struct StreamData {
    data: [Vec<f32>; 2],
    fft_data: VecDeque<AudioDataChunk>,
}
impl StreamData {
    fn new() -> Self {
        Self {
            data: [vec![], vec![]],
            fft_data: VecDeque::new(),
        }
    }
    fn append(&mut self, data: &[f32]) {
        for i in 0..data.len() / 2 {
            for j in 0..2 {
                self.data[j].push(data[i * 2 + j]);
            }
        }
        if self.data[0].len() >= FFT_SIZE {
            let mut planner = FftPlanner::new();
            let fft = planner.plan_fft_forward(FFT_SIZE);
            let mut fft_data = Box::new(([[Complex32::zero(); FFT_SIZE]; 2], [[0.0; FFT_SIZE]; 2]));

            for j in 0..2 {
                let mut data_f32 = [0.0; FFT_SIZE];
                data_f32[..].clone_from_slice(&self.data[j][..FFT_SIZE]);
                let mut data: Vec<_> = (&data_f32).into_iter().map(Complex32::from).collect();

                fft.process(&mut data[..]);

                for i in 0..FFT_SIZE {
                    fft_data.0[j][i] = data[i];
                }
                fft_data.1[j] = data_f32;

                self.data[j].drain(..FFT_STRIDE);
            }
            self.fft_data.push_back(fft_data);
        }
    }
    pub fn take(&mut self) -> Option<AudioDataChunk> {
        self.fft_data.pop_front()
    }
}
pub struct Streamer {
    #[allow(unused)]
    stream: Stream,
    pub data: Arc<Mutex<StreamData>>,
}
impl Streamer {
    fn err_fn(err: StreamError) {
        eprintln!("an error occurred on the output audio stream: {}", err);
    }
    pub fn begin() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap();
        // let device = host.default_input_device().unwrap();
        // let config = device.default_input_config().unwrap();
        // let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        assert_eq!(channels, 2);

        let data = Arc::new(Mutex::new(StreamData::new()));

        println!("device name: {}", device.name().unwrap_or("<>".to_string()));

        let stream = {
            let stream_data = data.clone();
            device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    stream_data.lock().unwrap().append(data);
                },
                Self::err_fn,
                None,
            )?
        };

        stream.play()?;

        Ok(Self { stream, data })
    }
}
