mod audio;

use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use std::collections::VecDeque;
use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;
use num_complex::Complex;

const WIDTH: u32 = 1000;
const HEIGHT: u32 = 500;

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
	data: Vec<f32>,
}

fn main() -> Result<(), Error> {
	let event_loop = EventLoop::new();

	let window = {
		let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
		WindowBuilder::new()
			.with_title("Hello Pixels")
			.with_inner_size(size)
			.with_min_inner_size(size)
			.build(&event_loop)
			.unwrap()
	};

	let mut pixels = {
		let window_size = window.inner_size();
		let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
		Pixels::new(WIDTH, HEIGHT, surface_texture)?
	};
	let mut world = World::new();

	let (send, recv) = std::sync::mpsc::channel();
	std::thread::spawn(|| {
		audio::listen_from_mic(send);
		println!("Stopped drawing");
	});


	let mut signal_queue = VecDeque::new();
	// let mut detector = McLeodDetector::new(1024, 512);
	let mut input = Vec::new();

	const MULTIPLIER: f32 = 1.0 / 48000.0;

	event_loop.run(move |event, _, control_flow| {
		// Draw the current frame
		// println!("polling {:?}", event);
		*control_flow = ControlFlow::Poll;
		match event {
			Event::MainEventsCleared => {
				signal_queue.extend(recv.try_iter().flatten());
				if signal_queue.len() > 1024 {
					input.truncate(0);
					let length = signal_queue.len();
					input.extend(signal_queue.drain(length - 1024..length));
					signal_queue.clear();
					// dbg!("aa");
					// pitch_detection::detector::internals::normalized_square_difference(&input, &mut scratch0, &mut scratch1, &mut scratch2, &mut world.data);

					// let pitch = detector.get_pitch(&input, 48000, 0.2, 0.1);

					// if let Some(pitch) = pitch {
					// println!("Frequency: {}, Clarity: {}", pitch.frequency, pitch.clarity);
					world.update(&input);
					world.draw(pixels.get_frame());
					if pixels
						.render()
						.map_err(|e| dbg!(e))
						.is_err()
					{
						*control_flow = ControlFlow::Exit;
						return;
					}
					// }
				}
			}
			// Event::RedrawRequested(_) => {}
			Event::WindowEvent { event, .. } => {
				match event {
					winit::event::WindowEvent::CloseRequested => {
						*control_flow = ControlFlow::Exit;
					}
					_ => ()
				}
			}
			_ => ()
		}
	})
}

impl World {
	/// Create a new `World` instance that can draw a moving box.
	fn new() -> Self {
		Self {
			data: vec![0.0; 1024],
		}
	}

	/// Update the `World` internal state; bounce the box around the screen.
	fn update(&mut self, data: &[f32]) {
		// dbg!(data);
		self.data.copy_from_slice(data);
	}

	/// Draw the `World` state to the frame buffer.
	///
	/// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
	fn draw(&self, frame: &mut [u8]) {
		for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
			let x = (i % WIDTH as usize);
			let y = (i / WIDTH as usize);

			if let Some(height) = self.data.get(x) {
				let height = height * 500.0 + 250.0;
				let rgba = [255, 0, 0, 255];
				let y = y as f32;
				if (y - height).abs() < 3.0 {
					pixel.copy_from_slice(&rgba);
				} else {
					pixel.copy_from_slice(&[0x48, 0xb2, 0xe8, 0xff])
				}
			} else {
				pixel.copy_from_slice(&[0x48, 0xb2, 0xe8, 0xff])
			}
		}
	}
}