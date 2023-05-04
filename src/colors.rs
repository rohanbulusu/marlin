
pub const BLACK: Color = Color { channels: [0, 0, 0, 255] };
pub const WHITE: Color = Color { channels: [255, 255, 255,255] };
pub const RED: Color = Color { channels: [235, 64, 52, 255] };
pub const BLUE: Color = Color { channels: [20, 152, 252, 255] };

#[derive(Clone, Copy, PartialEq)]
pub struct Color {
	channels: [u32; 4]
}

impl Color {

	fn rgba_value_within_bounds(value: u32) -> bool {
		value <= 255
	}

	pub fn new(r: u32, g: u32, b: u32) -> Color {
		Self::with_alpha(r, g, b, 255)
	}

	pub fn with_alpha(r: u32, g: u32, b: u32, a: u32) -> Color {
		if !Self::rgba_value_within_bounds(r) {
			panic!("r value {} out of bounds", r)
		}
		if !Self::rgba_value_within_bounds(g) {
			panic!("g value {} out of bounds", g)
		}
		if !Self::rgba_value_within_bounds(b) {
			panic!("b value {} out of bounds", b)
		}
		if !Self::rgba_value_within_bounds(a) {
			panic!("a value {} out of bounds", a)
		}
		Self {
			channels: [r, g, b, a],
		}
	}

	pub fn as_slice(&self) -> &[u32; 4] {
		&self.channels
	}

	pub fn in_percentages(&self) -> [f32; 3] {
		[
			self.channels[0] as f32 / 255.0,
			self.channels[1] as f32 / 255.0,
			self.channels[2] as f32 / 255.0
		]
	}

	pub fn mix(colors: &[Color]) -> Color {
		let mut mixing_channels = [0; 4];

		for color in colors {
			for (i, mixing_channel) in mixing_channels.iter_mut().enumerate() {
				*mixing_channel += color.channels[i];
			}
		}

		for mixing_channel in &mut mixing_channels {
			*mixing_channel /= colors.len() as u32;
		}

		Color::with_alpha(
			mixing_channels[0],
			mixing_channels[1],
			mixing_channels[2],
			mixing_channels[3]
		)
	}

}

impl From<[f32; 3]> for Color {
	fn from(rgb: [f32; 3]) -> Color {
		Self::new(
			(rgb[0] * 255.0) as u32, 
			(rgb[1] * 255.0) as u32, 
			(rgb[2] * 255.0) as u32
		)
	}
}
