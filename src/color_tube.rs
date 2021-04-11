use ggez::{nalgebra, Context, GameResult};
use ggez::graphics::{self, Drawable, Font, Color, Scale, Mesh, DrawMode, DrawParam, BlendMode, Rect, Text};
use ggez::event::KeyCode;
use nalgebra::Point2;
use crate::colors::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ColorTubeContent {
	color: Color,
	amount: f32,
}

impl ColorTubeContent {
	pub fn new(color: Color, amount: f32) -> Self {
		Self{ color, amount }
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorTube {
	pub hovered: bool,
	pub mousedown: bool,
	pub clicked: bool,
	pub dimensions: Rect,
	pub capacity: f32,
	pub keycode: Option<KeyCode>,
	contents: Vec<ColorTubeContent>,
	font: Font,
}

impl ColorTube {
	pub fn new(capacity: f32, contents: Vec<ColorTubeContent>, font: Font) -> Self {
		Self {
			hovered: false,
			mousedown: false,
			clicked: false,
			dimensions: Rect::new(0.0, 0.0, 50.0, 50.0 * capacity),
			capacity,
			keycode: None,
			contents,
			font
		}
	}

	pub fn amount(&self) -> f32 {
		self.contents.iter().map(|c| c.amount).sum()
	}

	pub fn remaining_capacity(&self) -> f32 {
		self.capacity - self.amount()
	}

	pub fn main_color(&self) -> Option<Color> {
		let mut occurrences = std::collections::HashMap::new();
		for content in &self.contents {
			*occurrences.entry(content.color.to_rgb_u32()).or_insert(0) += content.amount.floor() as u32;
		}
		occurrences
			.into_iter()
			.max_by_key(|&(_, count)| count)
			.map(|(val, _)| Color::from_rgb_u32(val))
	}

	// returns 0.0 (0%) .. 1.0 (100%)
	pub fn color_pct(&self, color: Color) -> f32 {
		let mut amount = 0.0;
		for content in &self.contents {
			if content.color == color {
				amount += content.amount;
			}
		}
		(amount / self.capacity - self.remaining_capacity() / self.capacity - (self.amount() - self.remaining_capacity() - amount) / self.capacity).max(0.0).min(1.0)
	}

	// returns 0.0 (0%) .. 1.0 (100%)
	pub fn complete_pct(&self) -> f32 {
		if let Some(color) = self.main_color() {
			self.color_pct(color)
		} else {
			1.0
		}
	}

	pub fn fill_unchecked(&mut self, content: ColorTubeContent) -> Option<ColorTubeContent> {
		if self.remaining_capacity() < content.amount {
			return Some(content);
		}
		let count = self.contents.len();
		if count == 0 || self.contents[count - 1].color != content.color {
			self.contents.push(content);
		} else {
			self.contents[count - 1].amount += content.amount;
		}
		None
	}

	pub fn fill(&mut self, content: ColorTubeContent) -> Option<ColorTubeContent> {
		if self.remaining_capacity() < content.amount {
			return Some(content);
		}
		let count = self.contents.len();
		if count == 0 {
			self.contents.push(content);
		} else if self.contents[count - 1].color == content.color {
			self.contents[count - 1].amount += content.amount;
		} else {
			return Some(content);
		}
		None
	}

	pub fn drain(&mut self, mut amount: f32) -> Option<ColorTubeContent> {
		if amount > self.amount() {
			amount = self.amount();
		}
		if amount <= 0.0 {
			return None;
		}
		if let Some(mut content) = self.contents.pop() {
			if amount > content.amount {
				amount = content.amount;
			}
			if amount <= 0.0 {
				self.contents.push(content);
				return None;
			}
			if content.amount == amount {
				return Some(content);
			} else {
				content.amount -= amount;
				let new_content = ColorTubeContent::new(content.color, amount);
				self.contents.push(content);
				return Some(new_content);
			}
		}
		None
	}
}

impl Drawable for ColorTube {
	fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
		let scale = 1.0;
		let w_scaled = self.dimensions.w * scale;
		let w_inner_scaled = (self.dimensions.w - 1.0) * scale;
		let w_half = (w_scaled / 2.0).floor();
		let h_scaled = self.dimensions.h * scale;
		let mut color_border = if self.clicked {
			COLOR_TUBE_BORDER_FOCUS
		} else if self.hovered || self.mousedown {
			COLOR_TUBE_BORDER_HOVER
		} else {
			COLOR_TUBE_BORDER
		};
		if self.mousedown {
			color_border.a = 0.5;
		}

		// Draw fill
		let mut filled_amount = 0.0;
		for content in &self.contents {
			let total_amount = filled_amount + content.amount;
			let fill_startx = self.dimensions.x + 1.0;
			let fill_starty = self.dimensions.y + h_scaled - w_scaled * total_amount;
			let fill_h = w_scaled * content.amount;
			if filled_amount < 0.5 {
				// Draw fill with rounded bottom
				let mut fill_points = Vec::new();
				if content.amount >= 0.5 {
					fill_points.push(Point2::new(fill_startx, fill_starty));
				}
				if content.amount > 0.0 {
					let steps = (w_scaled / 4.0).floor() as u32;
					for i in 0..=steps {
						let step_x = fill_startx + (i as f32 / steps as f32) * w_inner_scaled;
						let step_y = fill_starty + fill_h - w_half + (i as f32 * std::f32::consts::PI / steps as f32).sin() * w_half;
						if step_y >= fill_starty {
							fill_points.push(Point2::new(step_x, step_y));
						}
					}
				}
				if content.amount >= 0.5 {
					fill_points.push(Point2::new(self.dimensions.x + w_inner_scaled, fill_starty));
				}
				if fill_points.len() >= 3 {
					Mesh::new_polygon(ctx, DrawMode::fill(), &fill_points, content.color)?.draw(ctx, param)?;
				}
			} else {
				// Draw normal square fill
				Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{
					x: fill_startx,
					y: fill_starty,
					w: w_inner_scaled,
					h: fill_h
				}, content.color)?.draw(ctx, param)?;
			}
			filled_amount = total_amount;
		}

		// Draw border
		let mut border_points = Vec::new();
		let steps = (w_scaled / 4.0).floor() as u32;
		border_points.push(Point2::new(self.dimensions.x, self.dimensions.y));
		for i in 0..=steps {
			border_points.push(Point2::new(self.dimensions.x + (i as f32 / steps as f32) * w_scaled, self.dimensions.y + h_scaled - w_half + (i as f32 * std::f32::consts::PI / steps as f32).sin() * w_half));
		}
		border_points.push(Point2::new(self.dimensions.x + w_scaled, self.dimensions.y));
		Mesh::new_polygon(ctx, DrawMode::stroke(2.0), &border_points, color_border)?.draw(ctx, param)?;

		// Draw keycode text
		if let Some(keycode) = self.keycode {
			let keystr = match keycode {
				KeyCode::Key1 => "1",
				KeyCode::Key2 => "2",
				KeyCode::Key3 => "3",
				KeyCode::Key4 => "4",
				KeyCode::Key5 => "5",
				KeyCode::Key6 => "6",
				KeyCode::Key7 => "7",
				KeyCode::Q => "Q",
				KeyCode::W => "W",
				KeyCode::E => "E",
				KeyCode::R => "R",
				KeyCode::T => "T",
				KeyCode::Y => "Y",
				KeyCode::U => "U",
				KeyCode::A => "A",
				KeyCode::S => "S",
				KeyCode::D => "D",
				KeyCode::F => "F",
				KeyCode::G => "G",
				KeyCode::H => "H",
				KeyCode::J => "J",
				KeyCode::Z => "Z",
				KeyCode::X => "X",
				KeyCode::C => "C",
				KeyCode::V => "V",
				KeyCode::B => "B",
				KeyCode::N => "N",
				KeyCode::M => "M",
				_ => "",
			};
			if keystr.len() > 0 {
				let mut keytext = Text::new(keystr);
				keytext.set_font(self.font, Scale::uniform(18.0));
				let keytext_h = keytext.height(ctx) as f32;
				graphics::queue_text(ctx, &keytext, Point2::new(self.dimensions.x, self.dimensions.y - keytext_h), Some(COLOR_YELLOW));
			}
		}

		// Draw completed text
		let mut pcttext = Text::new(format!("{}%", (self.complete_pct() * 100.0).floor()));
		pcttext.set_font(self.font, Scale::uniform(18.0));
		let pcttext_h = pcttext.height(ctx) as f32;
		let pcttext_w = pcttext.width(ctx) as f32;
		graphics::queue_text(ctx, &pcttext, Point2::new(self.dimensions.x + (self.dimensions.w - pcttext_w), self.dimensions.y - pcttext_h), Some(color_border));

		Ok(())
	}

	fn dimensions(&self, _ctx: &mut Context) -> Option<Rect> { Some(self.dimensions) }
	fn set_blend_mode(&mut self, _mode: Option<BlendMode>) {}
	fn blend_mode(&self) -> Option<BlendMode> { None }
}