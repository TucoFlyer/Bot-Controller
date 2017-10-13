use message::OverlayRect;
use vecmath::{Vector2, Vector4};
use std::io::Cursor;
use std::mem;
use bmfont::{BMFont, OrdinateOrientation, CharPosition, StringParseError};

pub const VIDEO_HZ : u32 = 60;

#[derive(Debug, Clone)]
pub struct DrawingContext {
	pub scene: Vec<OverlayRect>,
	pub default_font: BMFont,
	pub current: DrawingState,
}

#[derive(Debug, Clone)]
pub struct DrawingState {
	pub color: Vector4<f64>,
	pub background_color: Vector4<f64>,
	pub font: BMFont,
	pub pixel_unit_size: f64,
	pub line_thickness: f64,
	pub text_height: f64,
	pub text_margin: f64,
}

fn load_default_font() -> BMFont {
	let fnt = include_bytes!("../images/din-alternate.fnt");
	BMFont::new(Cursor::new(&fnt[..]), OrdinateOrientation::TopToBottom).unwrap()
}

impl DrawingState {
	fn new(default_font: &BMFont) -> DrawingState {
		let pix_1080p = 2.0 / 1920.0;
		DrawingState {
			color: [1.0, 1.0, 1.0, 1.0],
			background_color: [0.0, 0.0, 0.0, 0.25],
			font: default_font.clone(),
			pixel_unit_size: pix_1080p,
			line_thickness: pix_1080p * 2.0,
			text_height: pix_1080p * 24.0,
			text_margin: pix_1080p * 3.0,
		}
	}

	pub fn swap_colors(&mut self) {
		mem::swap(&mut self.color, &mut self.background_color);
	}
}

impl DrawingContext {
	pub fn new() -> DrawingContext {
		let scene = Vec::new();
		let default_font = load_default_font();
		let current = DrawingState::new(&default_font);
		DrawingContext { scene, default_font, current }
	}

	pub fn clear(&mut self) {
		self.scene.clear();
		self.current = DrawingState::new(&self.default_font);
	}

	pub fn pixels(&self, pix: f64) -> f64 {
		pix * self.current.pixel_unit_size
	}

	pub fn ems(&self, em: f64) -> f64 {
		em * self.current.text_height
	}

	pub fn solid_rect(&mut self, rect: Vector4<f64>) {
		self.scene.push(OverlayRect {
			src: [ 511.0, 511.0, 1.0, 1.0 ],
			dest: rect,
			rgba: self.current.color
		});
	}

	pub fn outline_rect(&mut self, rect: Vector4<f64>) {
		// Thin rectangular outline outside the rect
		// ---
		// | |
		// ---
	
		let (x, y, w, h) = (rect[0], rect[1], rect[2], rect[3]);
		let t = self.current.line_thickness;
		let t2 = t * 2.0;

		self.solid_rect([ x-t, y-t, w+t2, t ]);
		self.solid_rect([ x-t, y+h, w+t2, t ]);
		self.solid_rect([ x-t, y, t, h ]);
		self.solid_rect([ x+w, y, t, h ]);
	}

	pub fn text(&mut self, top_left: Vector2<f64>, text: &str) -> Result<(), StringParseError> {
		let shape = TextShape::parse(&self.current.font, self.current.text_height, text)?;
		shape.draw(&mut self.scene, self.current.color, top_left);
		Ok(())
	}

	pub fn text_box(&mut self, top_left: Vector2<f64>, text: &str) -> Result<Vector4<f64>, StringParseError> {
		let shape = TextShape::parse(&self.current.font, self.current.text_height, text)?;
		let size = shape.size();
		let m = self.current.text_margin;
		let bg_rect = [ top_left[0] - m, top_left[1] - m, size[0] + m * 2.0, size[1] + m * 2.0];
		self.current.swap_colors();
		self.solid_rect(bg_rect);
		self.current.swap_colors();
		shape.draw(&mut self.scene, self.current.color, top_left);
		Ok(bg_rect)
	}
}

struct TextShape {
	scale: f64,
	chars: Vec<CharPosition>,
}

impl TextShape {
	fn parse(font: &BMFont, height: f64, s: &str) -> Result<TextShape, StringParseError> {
		Ok(TextShape {
			scale: height / (font.base_height() as f64),
			chars: font.parse(s)?
		})
	}

	fn size(&self) -> Vector2<f64> {
		self.chars.iter().fold([0.0, 0.0], |size, char| { [
			size[0].max((char.screen_rect.x + (char.screen_rect.width as i32)) as f64 * self.scale),
			size[1].max((char.screen_rect.y + (char.screen_rect.height as i32)) as f64 * self.scale),
		]})
	}

	fn draw(&self, scene: &mut Vec<OverlayRect>, rgba: Vector4<f64>, top_left: Vector2<f64>) {
		for char in &self.chars {
			let src = [
				char.page_rect.x as f64 * self.scale,
				char.page_rect.y as f64 * self.scale,
				char.page_rect.width as f64 * self.scale,
				char.page_rect.height as f64 * self.scale,
			];
			let dest = [
				char.screen_rect.x as f64 * self.scale + top_left[0],
				char.screen_rect.y as f64 * self.scale + top_left[1],
				char.screen_rect.width as f64 * self.scale,
				char.screen_rect.height as f64 * self.scale,
			];
			scene.push(OverlayRect { src, dest, rgba });
		}
	}
}
