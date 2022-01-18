use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};

use once_cell::unsync::OnceCell;
use std::cell::Cell;
use std::sync::mpsc;
use enigo::{MouseControllable, KeyboardControllable};
use num_complex::Complex;
use pitch_detection::detector::PitchDetector;
use std::time::Duration;

const CLICK_TIME_MILLI: u64 = 500;
const VOLUME_MIN: f32 = 0.2;

const BUFFER_SIZE: usize = 960;

pub fn listen_from_mic(sender: mpsc::Sender<Vec<f32>>) {
	let host = cpal::default_host();

	let device = host.default_input_device().expect("No default input device available.");
	dbg!(device.name()).ok();
	let config = device.supported_input_configs()
		.expect("Error querying config")
		.next()
		.expect("No configs??");

	dbg!(config.sample_format());
	let config = config.with_max_sample_rate();
	dbg!(&config);
	let mut input = enigo::Enigo::new();
	let last_time = OnceCell::new();

	let mut detector = pitch_detection::detector::mcleod::McLeodDetector::new(800, 800 / 2);

	let stream = device.build_input_stream(
		&config.config(),
		move |data: &[f32], info: &cpal::InputCallbackInfo| {
			let volume = data.iter().map(|c| c.abs()).fold(0.0f32, |acc, c| acc.max(c));
			sender.send(data.to_vec()).expect("Failed to send");
			// println!("VOLUME: {}, Clarity: {}", volume, data.len());
			if volume > VOLUME_MIN && data.len() >= 800 {
				if let Some(pitch) = detector.get_pitch(&data[..800], config.sample_rate().0 as usize, VOLUME_MIN, 0.5) {
					println!("Frequency: {}, Clarity: {}", pitch.frequency, pitch.clarity);
					if pitch.frequency > 1400.0 {
						let callback_instant = info.timestamp().callback;
						let last_instant = last_time.get_or_init(|| Cell::new(callback_instant));
						if callback_instant.duration_since(&last_instant.get()).unwrap().as_millis() as u64 > CLICK_TIME_MILLI {
							last_instant.set(callback_instant);
							println!("Click: {:?}", callback_instant);
							input.key_click(enigo::Key::Space);
						}
					}
				}
			}
		},
		move |err| { // react to errors here.
			dbg!(err);
		},
	).expect("Building stream failed");

	stream.play().expect("Failed to play stream");
	loop {
		std::thread::sleep(Duration::new(10000, 0));
	}
}