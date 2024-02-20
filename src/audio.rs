use std::{cell::RefCell, rc::Rc, thread::JoinHandle};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use oddio::MixerControl;

pub struct AudioState {
    device: std::sync::Arc::<cpal::platform::Device>,
    config: cpal::StreamConfig,
    main_audio_thread: JoinHandle<()>,
    mixer_handle: Rc<RefCell<MixerControl<[f32; 2]>>>,
}

impl AudioState {

    pub fn new() -> Self {

        let host = cpal::default_host();
        let device = std::sync::Arc::new(host.default_output_device().expect("Failed top find a default output device"));
        let config = device.default_output_config().unwrap().config();

        let (mut mixer_handle_t, mut mixer) = oddio::Mixer::new();
        let sample_rate = config.sample_rate.0;

        let mut mixer_handle = Rc::new(RefCell::new(mixer_handle_t));

        let main_thread_config = config.clone();
        let main_thread_device = device.clone();
        let main_audio_thread = std::thread::spawn(move || {
            let stream = main_thread_device.build_output_stream(
                &main_thread_config, 
                move |out_flat: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let out_stereo = oddio::frame_stereo(out_flat);
                    oddio::run(&mut mixer, sample_rate, out_stereo);
                },
                move |err| {
                    eprintln!("{}", err);
                },
                None,
            ).unwrap();

            stream.play().unwrap();

            while (true) {
                std::thread::sleep(std::time::Duration::from_secs_f32(1.0));
            }
        });

        AudioState { device: device.clone(), config, main_audio_thread, mixer_handle }
    }

    pub fn play_wav(&self) {
        let mut reader = hound::WavReader::open("./assets/hitsound480.wav").unwrap();
        
        let hound::WavSpec {
            sample_rate: source_sample_rate,
            sample_format,
            bits_per_sample,
            ..
        } = reader.spec();

        let samples_result: Result<Vec<f32>, _> = match sample_format {
            hound::SampleFormat::Int => {
                let max_value = 2_u32.pow(bits_per_sample as u32 - 1) - 1;
                reader.samples::<i32>().map(|sample| sample.map(|sample| sample as f32 / max_value as f32)).collect()
            }
            hound::SampleFormat::Float => reader.samples::<f32>().collect(),
        };

        let mut samples = samples_result.unwrap();

        let samples_stereo = oddio::frame_stereo(&mut samples);
        let sound_frames = oddio::Frames::from_slice(source_sample_rate, samples_stereo);

        self.mixer_handle.borrow_mut().play(oddio::FramesSignal::from(sound_frames));
    }
}