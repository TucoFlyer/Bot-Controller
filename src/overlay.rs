use vecmath::*;
use message::OverlayRect;
use std::io::Cursor;
use std::mem;
use bmfont::{BMFont, OrdinateOrientation, CharPosition, StringParseError};

pub const OVERLAY_HZ : u32 = 60;

#[derive(Debug, Clone)]
pub struct DrawingContext {
	pub scene: Vec<OverlayRect>,
	pub default_font: BMFont,
	pub current: DrawingState,
}

#[derive(Debug, Clone)]
pub struct DrawingState {
	pub color: Vector4<f32>,
	pub background_color: Vector4<f32>,
	pub outline_color: Vector4<f32>,
	pub font: BMFont,
	pub outline_thickness: f32,
	pub text_height: f32,
	pub text_margin: f32,
}

fn load_default_font() -> BMFont {
	let fnt = include_bytes!("../images/din-alternate.fnt");
	BMFont::new(Cursor::new(&fnt[..]), OrdinateOrientation::TopToBottom).unwrap()
}

impl DrawingState {
	fn new(default_font: &BMFont) -> DrawingState {
		DrawingState {
			color: [1.0, 1.0, 1.0, 1.0],
			background_color: [0.0, 0.0, 0.0, 0.25],
			outline_color: [1.0, 1.0, 1.0, 0.33],
			font: default_font.clone(),
			outline_thickness: 2.0/1920.0 * 2.0,
			text_height: 2.0/1920.0 * 24.0,
			text_margin: 2.0/1920.0 * 6.0,
		}
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

	pub fn solid_rect(&mut self, rect: Vector4<f32>) {
		if self.current.color[3] > 0.0 && rect[2] > 0.0 && rect[3] > 0.0 {
			self.scene.push(OverlayRect {
				src: [ 511, 511, 1, 1 ],
				dest: rect,
				rgba: self.current.color
			});
		}
	}

	pub fn background_rect(&mut self, rect: Vector4<f32>) {
		mem::swap(&mut self.current.color, &mut self.current.background_color);
		self.solid_rect(rect);
		mem::swap(&mut self.current.color, &mut self.current.background_color);
	}

	pub fn outline_rect(&mut self, rect: Vector4<f32>) {
		// Thin rectangular outline outside the rect
		// ---
		// | |
		// ---

		if self.current.outline_color[3] > 0.0 && rect[2] > 0.0 && rect[3] > 0.0 {	
			let (x, y, w, h) = (rect[0], rect[1], rect[2], rect[3]);
			let t = self.current.outline_thickness;
			if t > 0.0 {
				let t2 = t * 2.0;

				mem::swap(&mut self.current.color, &mut self.current.outline_color);

				self.solid_rect([ x-t, y-t, w+t2, t ]);
				self.solid_rect([ x-t, y+h, w+t2, t ]);
				self.solid_rect([ x-t, y, t, h ]);
				self.solid_rect([ x+w, y, t, h ]);

				mem::swap(&mut self.current.color, &mut self.current.outline_color);
			}
		}
	}

	pub fn text(&mut self, pos: Vector2<f32>, anchor: Vector2<f32>, text: &str) -> Result<(), StringParseError> {
		let shape = TextShape::parse(&self.current.font, self.current.text_height, text)?;
		let size = shape.size();
		let top_left = vec2_sub(pos, vec2_mul(size, anchor));

		if self.current.color[3] > 0.0 {
			shape.draw(&mut self.scene, self.current.color, top_left);
		}

		Ok(())
	}

	pub fn text_box(&mut self, pos: Vector2<f32>, anchor: Vector2<f32>, text: &str) -> Result<Vector4<f32>, StringParseError> {
		let shape = TextShape::parse(&self.current.font, self.current.text_height, text)?;
		let size = shape.size();
		let m = self.current.text_margin;
		let box_size = [ size[0] + m * 2.0, size[1] + m * 2.0 ];
		let box_corner = vec2_sub(pos, vec2_mul(box_size, anchor));
		let box_rect = [ box_corner[0], box_corner[1], box_size[0], box_size[1] ];
		let text_corner = [ box_corner[0] + m, box_corner[1] + m ];

		self.background_rect(box_rect);
		if self.current.color[3] > 0.0 {
			shape.draw(&mut self.scene, self.current.color, text_corner);
		}
		self.outline_rect(box_rect);

		Ok(box_rect)
	}
}

struct TextShape {
	scale: f32,
	chars: Vec<CharPosition>,
}

impl TextShape {
	fn parse(font: &BMFont, height: f32, s: &str) -> Result<TextShape, StringParseError> {
		Ok(TextShape {
			scale: height / (font.base_height() as f32),
			chars: font.parse(s)?
		})
	}

	fn size(&self) -> Vector2<f32> {
		self.chars.iter().fold([0.0, 0.0], |size, char| { [
			size[0].max((char.screen_rect.x + (char.screen_rect.width as i32)) as f32 * self.scale),
			size[1].max((char.screen_rect.y + (char.screen_rect.height as i32)) as f32 * self.scale),
		]})
	}

	fn draw(&self, scene: &mut Vec<OverlayRect>, rgba: Vector4<f32>, top_left: Vector2<f32>) {
		for char in &self.chars {
			let src = [
				char.page_rect.x,
				char.page_rect.y,
				char.page_rect.width as i32,
				char.page_rect.height as i32,
			];
			let dest = [
				char.screen_rect.x as f32 * self.scale + top_left[0],
				char.screen_rect.y as f32 * self.scale + top_left[1],
				char.screen_rect.width as f32 * self.scale,
				char.screen_rect.height as f32 * self.scale,
			];
			scene.push(OverlayRect { src, dest, rgba });
		}
	}
}
