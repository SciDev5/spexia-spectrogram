use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host, Stream, StreamError, SupportedStreamConfig,
};
use rustfft::{num_complex::Complex32, num_traits::Zero, FftPlanner};

// pub const FFT_SIZE: usize = 4096;
pub const FFT_SIZE: usize = 2048;
pub const FFT_STRIDE: usize = 512;
pub const HALF_FFT_SIZE: usize = FFT_SIZE / 2;

pub type AudioDataChunk = Box<(
    [[Complex32; FFT_SIZE]; 2],
    [[f32; FFT_SIZE]; 2],
    [[f32; HALF_FFT_SIZE]; 2],
)>;

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
        let window_fn: [f32; FFT_SIZE] = std::array::from_fn(|i| {
            ((i as f32 / HALF_FFT_SIZE as f32 - 1.0) * 3.14159).cos() + 1.0
        });
        for i in 0..data.len() / 2 {
            for j in 0..2 {
                self.data[j].push(data[i * 2 + j]);
            }
        }
        if self.data[0].len() >= FFT_SIZE + 2 {
            let mut planner = FftPlanner::new();
            let fft = planner.plan_fft_forward(FFT_SIZE);
            let mut fft_data = Box::new((
                [[Complex32::zero(); FFT_SIZE]; 2],
                [[0.0; FFT_SIZE]; 2],
                [[0.0; HALF_FFT_SIZE]; 2],
            ));

            for j in 0..2 {
                let mut data_f32 = [0.0; FFT_SIZE];
                let mut data_f32_shifted = [0.0; FFT_SIZE];
                data_f32[..].clone_from_slice(&self.data[j][..FFT_SIZE]);
                data_f32_shifted[..].clone_from_slice(&self.data[j][1..][..FFT_SIZE]);
                let mut data: Vec<_> = (&data_f32)
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| Complex32::new(*v * window_fn[i], 0.0))
                    .collect();
                let mut data_shifted: Vec<_> = (&data_f32_shifted)
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| Complex32::new(*v * window_fn[i], 0.0))
                    .collect();

                fft.process(&mut data[..]);
                fft.process(&mut data_shifted[..]);

                for i in 0..FFT_SIZE {
                    fft_data.0[j][i] = data[i];
                }
                for i in 0..HALF_FFT_SIZE {
                    fft_data.2[j][i] = (data[i].conj() * data_shifted[i]).arg().abs();
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

struct StreamerInternalState {
    lost_device: bool,
}
type StreamerInternalStateRef = Arc<Mutex<StreamerInternalState>>;

pub struct Streamer {
    #[allow(unused)]
    stream: Stream,
    internals: StreamerInternalStateRef,
    pub data: Arc<Mutex<StreamData>>,
}
impl Streamer {
    fn err_fn(err: StreamError, internals: StreamerInternalStateRef) {
        eprintln!("an error occurred on the output audio stream: {}", err);
        if let StreamError::DeviceNotAvailable = err {
            internals.lock().unwrap().lost_device = true;
        }
    }
    pub fn update_stream(&mut self, device_selector: &DeviceSelector) {
        self.stream = Self::get_stream(self.data.clone(), device_selector, self.internals.clone());
    }
    fn get_stream(
        stream_data: Arc<Mutex<StreamData>>,
        device_selector: &DeviceSelector,
        internals: StreamerInternalStateRef,
    ) -> Stream {
        let (device, config) = device_selector.get_device_and_config();
        let device = device.unwrap();
        let config = config.unwrap();

        assert_eq!(config.channels(), 2);
        println!("device name: {}", device.name().unwrap_or("<>".to_string()));

        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    stream_data.lock().unwrap().append(data);
                },
                {
                    let internals = internals.clone();
                    move |err| Self::err_fn(err, internals.clone())
                },
                None,
            )
            .unwrap();

        stream.play().unwrap();
        stream
    }

    pub fn did_lose_device(&self) -> bool {
        self.internals.lock().unwrap().lost_device
    }
    pub fn begin(device_selector: &DeviceSelector) -> Result<Self, Box<dyn std::error::Error>> {
        let data = Arc::new(Mutex::new(StreamData::new()));
        let internals = Arc::new(Mutex::new(StreamerInternalState { lost_device: false }));
        let stream = Self::get_stream(data.clone(), device_selector, internals.clone());

        Ok(Self {
            stream,
            data,
            internals,
        })
    }
}

pub struct DeviceSelector {
    use_input: bool,
    host: Host,
    current_device: Option<Device>,
    last_poll: SystemTime,
}

impl DeviceSelector {
    pub fn new(use_input: bool) -> Self {
        let mut this = Self {
            use_input,
            host: cpal::default_host(),
            current_device: None,
            last_poll: SystemTime::now(),
        };
        this.current_device = this.get_device();
        this
    }
    pub fn poll_device_has_changed(&mut self, skip_waiting: bool) -> bool {
        if skip_waiting {
            self.last_poll = SystemTime::now();
            // continue
        } else {
            if let Ok(elapsed) = self.last_poll.elapsed() {
                if elapsed.as_secs() >= 1 {
                    self.last_poll = SystemTime::now();
                    // continue
                } else {
                    return false;
                }
            } else {
                eprintln!("Failed to get elapsed time since last audio device poll.");
                self.last_poll = SystemTime::now(); // reset to give it another chance to work.
                return false;
            }
        }

        let prev_device = self.current_device.as_ref();
        let device = self.get_device();

        // If there were a better way I would use it.
        let updated = prev_device.map(|it| it.name().ok()).flatten()
            != device.as_ref().map(|it| it.name().ok()).flatten();

        if updated {
            self.current_device = device;
        }

        updated
    }
    fn get_device(&self) -> Option<Device> {
        if self.use_input {
            self.host.default_input_device()
        } else {
            self.host.default_output_device()
        }
    }
    fn get_config_from_device(&self, device: &Device) -> Option<SupportedStreamConfig> {
        (if self.use_input {
            device.default_input_config()
        } else {
            device.default_output_config()
        })
        .ok()
    }

    pub fn get_device_and_config(&self) -> (Option<Device>, Option<SupportedStreamConfig>) {
        let device = self.get_device();
        let config = device
            .as_ref()
            .map(|d| self.get_config_from_device(d))
            .flatten();
        (device, config)
    }
}
