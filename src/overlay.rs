use vecmath::*;
use message::TICK_HZ;
use message::OverlayRect;
use std::io::Cursor;
use std::mem;
use config::Config;
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
		self.sprite_rect(rect, [ 511, 511, 1, 1 ]);
	}

	pub fn sprite_rect(&mut self, rect: Vector4<f32>, src: Vector4<i32>) {
		if self.current.color[3] > 0.0 && rect[2] > 0.0 && rect[3] > 0.0 {
			self.scene.push(OverlayRect {
				src,
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

	pub fn text(&mut self, pos: Vector2<f32>, anchor: Vector2<f32>, text: &str) -> Result<Vector4<f32>, StringParseError> {
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

pub struct ParticleDrawing {
	particles: Vec<Particle>,
}

struct Particle {
	position: Vector2<f32>,
	velocity: Vector2<f32>,
}

impl ParticleDrawing {
	pub fn new() -> ParticleDrawing {
		ParticleDrawing {
			particles: Vec::new(),
		}
	}

	fn update_particle_count(&mut self, config: &Config) {
		self.particles.truncate(config.overlay.particle_count);

		while self.particles.len() < config.overlay.particle_count {
			self.particles.push(Particle {
				position: vec2_rand_from_centered_unit_square(),
				velocity: [ 0.0, 0.0 ]
			});
		}
	}

	fn update_collisions(&mut self, config: &Config) {
		let mdist = config.overlay.particle_min_distance;
		let mdist2 = mdist * mdist;

		for p in 0 .. self.particles.len() {
			for q in p+1 .. self.particles.len() {
				let diff = vec2_sub(self.particles[p].position, self.particles[q].position);
				let l2 = vec2_square_len(diff);

				if l2 > 0.0 {
					if l2 < mdist2 {
						let l = l2.sqrt();
						let scalar = (mdist - l) * config.overlay.particle_min_distance_gain;
						let v_repel = vec2_scale(diff, scalar / l);

						self.particles[p].velocity = vec2_add(self.particles[p].velocity, v_repel);
						self.particles[q].velocity = vec2_sub(self.particles[q].velocity, v_repel);
					}
				}
			}
		}		
	}

	fn update_dynamics(&mut self, config: &Config) {
		for p in 0 .. self.particles.len() {
			let dt = 1.0 / TICK_HZ as f32;
			let v_damped = vec2_scale(self.particles[p].velocity, 1.0 - config.overlay.particle_damping);
			let v_rand = vec2_scale(vec2_rand_from_centered_unit_square(), config.overlay.particle_random_gain);
			let v = vec2_add(v_damped, v_rand);
			self.particles[p].velocity = v;
			self.particles[p].position = vec2_add(self.particles[p].position, vec2_scale(v, dt));
		}		
	}

	pub fn follow_rect(&mut self, config: &Config, rect: Vector4<f32>) {
		self.update_particle_count(config);

		for p in 0 .. self.particles.len() {
			let pos = self.particles[p].position;

			let edge_diff = vec2_sub(rect_nearest_perimeter_point(rect, pos), pos);
			let center_diff = vec2_sub(rect_center(rect), pos);
			let perpendicular = [ center_diff[1], -center_diff[0] ];

			let v_edge = vec2_scale(edge_diff, config.overlay.particle_edge_gain);
			let v_perp = vec2_scale(perpendicular, config.overlay.particle_perpendicular_gain);
			let v = vec2_add(v_edge, v_perp);
			
			self.particles[p].velocity = vec2_add(self.particles[p].velocity, v);
		}

		self.update_collisions(config);
		self.update_dynamics(config);
	}

	pub fn render(&self, config: &Config, draw: &mut DrawingContext) {
		let rect = rect_centered_on_origin(config.overlay.particle_size, config.overlay.particle_size);
		draw.current.color = config.overlay.particle_color;
		for particle in &self.particles {
			draw.sprite_rect(rect_translate(rect, particle.position), config.overlay.particle_sprite);
		}
	}
}