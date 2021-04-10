extern crate rand;
extern crate ggez;

mod imgui_wrapper;

use crate::imgui_wrapper::ImGuiWrapper;
use imgui::*;
use ggez::conf;
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::{self, Drawable, Font, Color, Scale, Mesh, DrawMode, DrawParam, BlendMode, Rect, Text, TextFragment};
use ggez::nalgebra as alg;
use alg::{Vector2, Point2};
use ggez::{Context, GameResult};
use rand::seq::SliceRandom;
use rand::thread_rng;

fn smallest_factor(mut n: usize) -> usize {
	let mut out = vec![];
	for i in 2..(n+1) {
		while n % i == 0 {
			out.push(i);
			n /= i;
		}
		if n == 1 { break; }
	}
	out.into_iter().min().unwrap_or(n)
}


const TUBE_WIDTH: f32 = 50.0;
const SCREEN_MARGIN: f32 = 50.0;
const TUBE_MARGIN: f32 = 25.0;

const COLOR_PINK: Color = Color::new(0.8823529411764706, 0.12941176470588237, 0.7098039215686275, 1.0);
const COLOR_PURPLE: Color = Color::new(0.6549019607843137, 0.17647058823529413, 0.8666666666666667, 1.0);
const COLOR_VIOLET: Color = Color::new(0.3843137254901961, 0.2, 0.8274509803921568, 1.0);
const COLOR_BLUE: Color = Color::new(0.050980392156862744, 0.08627450980392157, 0.49019607843137253, 1.0);
const COLOR_LIGHTBLUE: Color = Color::new(0.2235294117647059, 0.27058823529411763, 0.8313725490196079, 1.0);
const COLOR_CYAN: Color = Color::new(0.1411764705882353, 0.8705882352941177, 0.8705882352941177, 1.0);
const COLOR_GREEN: Color = Color::new(0.043137254901960784, 0.396078431372549, 0.07058823529411765, 1.0);
const COLOR_LIGHTGREEN: Color = Color::new(0.1803921568627451, 0.7803921568627451, 0.2235294117647059, 1.0);
const COLOR_OLIVE: Color = Color::new(0.5686274509803921, 0.7254901960784313, 0.08235294117647059, 1.0);
const COLOR_YELLOW: Color = Color::new(0.8235294117647058, 0.803921568627451, 0.16470588235294117, 1.0);
const COLOR_ORANGE: Color = Color::new(0.8117647058823529, 0.44313725490196076, 0.17647058823529413, 1.0);
const COLOR_RED: Color = Color::new(0.796078431372549, 0.1568627450980392, 0.1450980392156863, 1.0);

const COLOR_BG: Color = Color::new(0.125, 0.125, 0.125, 1.0);
const COLOR_TUBE_BORDER: Color = Color::new(0.5, 0.5, 0.5, 1.0);
const COLOR_TUBE_BORDER_HOVER: Color = Color::new(1.0, 1.0, 1.0, 1.0);
const COLOR_TUBE_BORDER_FOCUS: Color = COLOR_LIGHTBLUE;

#[derive(Debug, Clone, PartialEq)]
struct ColorTubeContent {
	color: Color,
	amount: f32,
}

impl ColorTubeContent {
	fn new(color: Color, amount: f32) -> Self {
		Self{ color, amount }
	}
}

#[derive(Debug, Clone, PartialEq)]
struct ColorTube {
	hovered: bool,
	mousedown: bool,
	mouseup: bool,
	dimensions: Rect,
	capacity: f32,
	contents: Vec<ColorTubeContent>,
	font: Font,
}

impl ColorTube {
	fn new(capacity: f32, contents: Vec<ColorTubeContent>, font: Font) -> Self {
		Self {
			hovered: false,
			mousedown: false,
			mouseup: false,
			dimensions: Rect::new(0.0, 0.0, 50.0, 50.0 * capacity),
			capacity,
			contents,
			font
		}
	}

	fn amount(&self) -> f32 {
		self.contents.iter().map(|c| c.amount).sum()
	}

	fn remaining_capacity(&self) -> f32 {
		self.capacity - self.amount()
	}

	fn main_color(&self) -> Option<Color> {
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
	fn color_pct(&self, color: Color) -> f32 {
		let mut amount = 0.0;
		for content in &self.contents {
			if content.color == color {
				amount += content.amount;
			}
		}
		(amount / self.capacity - self.remaining_capacity() / self.capacity - (self.amount() - self.remaining_capacity() - amount) / self.capacity).max(0.0).min(1.0)
	}

	// returns 0.0 (0%) .. 1.0 (100%)
	fn complete_pct(&self) -> f32 {
		if let Some(color) = self.main_color() {
			self.color_pct(color)
		} else {
			1.0
		}
	}

	fn fill_unchecked(&mut self, content: ColorTubeContent) -> Option<ColorTubeContent> {
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

	fn fill(&mut self, content: ColorTubeContent) -> Option<ColorTubeContent> {
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

	fn drain(&mut self, mut amount: f32) -> Option<ColorTubeContent> {
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
		let mut color_border = if self.mouseup {
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

		// Draw completed text
		let mut pcttext = Text::new(format!("{}%", (self.complete_pct() * 100.0).floor()));
		pcttext.set_font(self.font, Scale::uniform(18.0));
		let pcttext_h = pcttext.height(ctx) as f32;
		let pcttext_w = pcttext.width(ctx) as f32;
		graphics::queue_text(ctx, &pcttext, Point2::new(self.dimensions.x + (self.dimensions.w / 2.0 - pcttext_w / 2.0), self.dimensions.y - pcttext_h), Some(color_border));

		Ok(())
	}

	fn dimensions(&self, _ctx: &mut Context) -> Option<Rect> { Some(self.dimensions) }
	fn set_blend_mode(&mut self, _mode: Option<BlendMode>) {}
	fn blend_mode(&self) -> Option<BlendMode> { None }
}

struct MainState {
	imgui_wrapper: ImGuiWrapper,
	hidpi_factor: f32,
	font: Font,
	width: f32,
	height: f32,
	mouse_x: f32,
	mouse_y: f32,

	tube_capacity: f32,
	tubes: Vec<ColorTube>,
	tubes_factor: usize,
	selected_tube: Option<usize>,
}

impl MainState {
	fn new_tubes(&mut self) {
		let mut tubes_src = vec![
			ColorTube::new(self.tube_capacity, vec![ColorTubeContent::new(COLOR_PINK, self.tube_capacity)], self.font),
			ColorTube::new(self.tube_capacity, vec![ColorTubeContent::new(COLOR_PURPLE, self.tube_capacity)], self.font),
			ColorTube::new(self.tube_capacity, vec![ColorTubeContent::new(COLOR_VIOLET, self.tube_capacity)], self.font),
			ColorTube::new(self.tube_capacity, vec![ColorTubeContent::new(COLOR_BLUE, self.tube_capacity)], self.font),
			ColorTube::new(self.tube_capacity, vec![ColorTubeContent::new(COLOR_LIGHTBLUE, self.tube_capacity)], self.font),
			ColorTube::new(self.tube_capacity, vec![ColorTubeContent::new(COLOR_CYAN, self.tube_capacity)], self.font),
			ColorTube::new(self.tube_capacity, vec![ColorTubeContent::new(COLOR_GREEN, self.tube_capacity)], self.font),
			ColorTube::new(self.tube_capacity, vec![ColorTubeContent::new(COLOR_LIGHTGREEN, self.tube_capacity)], self.font),
			ColorTube::new(self.tube_capacity, vec![ColorTubeContent::new(COLOR_OLIVE, self.tube_capacity)], self.font),
			ColorTube::new(self.tube_capacity, vec![ColorTubeContent::new(COLOR_YELLOW, self.tube_capacity)], self.font),
			ColorTube::new(self.tube_capacity, vec![ColorTubeContent::new(COLOR_ORANGE, self.tube_capacity)], self.font),
			ColorTube::new(self.tube_capacity, vec![ColorTubeContent::new(COLOR_RED, self.tube_capacity)], self.font),
		];
		let mut rng = thread_rng();
		tubes_src.shuffle(&mut rng);

		let mut tubes = Vec::<ColorTube>::with_capacity(tubes_src.len());
		tubes.resize(tubes_src.len(), ColorTube::new(self.tube_capacity, Vec::new(), self.font));

		let mut filled_amount = 0.0;
		while filled_amount < self.tube_capacity {
			for i in 0..tubes_src.len() {
				if let Some(content) = tubes_src[i].drain(1.0) {
					tubes[i].fill_unchecked(content);
				} else {
					panic!("failed to drain from {:?}", tubes_src[i]);
				}
			}
			tubes.shuffle(&mut rng);
			filled_amount += 1.0;
		}

		tubes.resize(tubes_src.len() + 2, ColorTube::new(self.tube_capacity, Vec::new(), self.font));

		self.tubes_factor = smallest_factor(tubes.len());
		self.tubes = tubes;
	}

	// returns 0.0 (0%) .. 1.0 (100%)
	fn complete_pct(&self) -> f32 {
		let mut empty_tubes = 0;
		self.tubes
			.iter()
			.map(|t| if t.remaining_capacity() == t.capacity {
				empty_tubes += 1;
				0.0
			} else {
				t.complete_pct()
			})
			.sum::<f32>() / (self.tubes.len() - empty_tubes) as f32
	}

	fn new(mut ctx: &mut Context, hidpi_factor: f32) -> GameResult<MainState> {
		let imgui_wrapper = ImGuiWrapper::new(&mut ctx);
		let mouse_pos = ggez::input::mouse::position(ctx);
		let (width, height) = ggez::graphics::drawable_size(ctx);
		let font = Font::new_glyph_font_bytes(ctx, include_bytes!("../IBMPlexMono-Regular.ttf"))?;
		let mut s = MainState {
			imgui_wrapper,
			hidpi_factor,
			font,
			width,
			height,
			mouse_x: mouse_pos.x,
			mouse_y: mouse_pos.y,
			tube_capacity: 4.0,
			tubes: Vec::new(),
			tubes_factor: 1,
			selected_tube: None,
		};
		s.new_tubes();
		Ok(s)
	}
}

impl EventHandler for MainState {
	fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
		let tube_count = self.tubes.len();
		let max_cols = ((self.width - SCREEN_MARGIN * 2.0 + TUBE_MARGIN) / (TUBE_WIDTH + TUBE_MARGIN)).floor();
		let cols = (tube_count as f32 / self.tubes_factor as f32).ceil().min(max_cols).max(1.0);
		let total_w = cols * (TUBE_WIDTH + TUBE_MARGIN) - TUBE_MARGIN;

		let rows = (tube_count as f32 / cols).ceil();
		let total_h = rows * (self.tube_capacity * TUBE_WIDTH + TUBE_MARGIN);

		let mousedown = ggez::input::mouse::button_pressed(ctx, MouseButton::Left);

		for i in 0..tube_count {
			let (tubes_before, tubes_after) = self.tubes.split_at_mut(i);
			let tubes_after = tubes_after.split_first_mut();
			let (tube, tubes_after) = tubes_after.unwrap();

			// Update dimensions
			tube.dimensions.w = TUBE_WIDTH;
			tube.dimensions.h = tube.dimensions.w * tube.capacity;
			tube.dimensions.x = SCREEN_MARGIN + (self.width - SCREEN_MARGIN * 2.0) / 2.0 - total_w / 2.0 + (tube.dimensions.w + TUBE_MARGIN) * (i as f32 % cols).floor();
			tube.dimensions.y = SCREEN_MARGIN + (self.height - SCREEN_MARGIN * 2.0 + TUBE_MARGIN) / 2.0 - total_h / 2.0 + (tube.dimensions.h + TUBE_MARGIN) * (i as f32 / cols).floor();

			// Detect hover
			let hovered = self.mouse_x >= tube.dimensions.x &&
				self.mouse_x <= tube.dimensions.x + tube.dimensions.w &&
				self.mouse_y >= tube.dimensions.y &&
				self.mouse_y <= tube.dimensions.y + tube.dimensions.h;

			// Store previous mouse states
			let has_selected_tube = self.selected_tube.is_some();
			let is_selected_tube = has_selected_tube && self.selected_tube.unwrap() == i;
			let was_mousedown = tube.mousedown && !mousedown;
			let was_clicked = was_mousedown && hovered;

			// Detect mouse states
			tube.mousedown = mousedown && (tube.mousedown || tube.hovered);
			tube.mouseup = is_selected_tube || was_clicked;
			tube.hovered = !mousedown && hovered;

			// Handle click
			if is_selected_tube && was_clicked {
				// Deselect current tube
				self.selected_tube = None;
			} else if tube.mouseup && has_selected_tube && !is_selected_tube {
				// Get previously selected tube
				let prev_i = self.selected_tube.unwrap();
				let prev_tube = if prev_i < i {
					&mut tubes_before[prev_i]
				} else {
					&mut tubes_after[prev_i - i - 1]
				};

				// Attempt to move color from previously selected
				// to newly selected tube
				if let Some(content) = prev_tube.drain(tube.remaining_capacity()) {
					// println!("drain {:?}", content);
					if let Some(content) = tube.fill(content) {
						// Color doesn't match, put the color back into the previous tube
						// println!("could not fill {:?} with drained content", tube);
						prev_tube.fill_unchecked(content);
					}
				}

				// Deselect previously selected tube
				self.selected_tube = None;
			} else if tube.mouseup && !has_selected_tube {
				// Select current tube
				self.selected_tube = Some(i);
			}
		}
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
		graphics::clear(ctx, COLOR_BG);

		// Draw tubes
		let param = DrawParam::default();
		for tube in &mut self.tubes {
			tube.draw(ctx, param)?;
		}

		// Draw total completed text
		let mut pcttext = Text::new(format!("Level 1 ({}% completed)", (self.complete_pct() * 100.0).floor()));
		pcttext.set_font(self.font, Scale::uniform(18.0));
		// let pcttext_h = pcttext.height(ctx) as f32;
		let pcttext_w = pcttext.width(ctx) as f32;
		graphics::queue_text(ctx, &pcttext, Point2::new(self.width / 2.0 - pcttext_w / 2.0, SCREEN_MARGIN), Some(graphics::WHITE));

		graphics::draw_queued_text(ctx, param, None, graphics::FilterMode::Linear)?;

		// Draw UI
		self.imgui_wrapper.render(ctx, self.hidpi_factor, move |_ui| {
		}).expect("renderer error");

		graphics::present(ctx)?;
		Ok(())
	}

	fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
		self.mouse_x = x;
		self.mouse_y = y;
		self.imgui_wrapper.update_mouse_pos(x, y);
	}

	fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
		self.imgui_wrapper.update_mouse_down(button);
	}

	fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
		self.imgui_wrapper.update_mouse_up(button);
	}

	fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, keymods: KeyMods, _repeat: bool) {
		self.imgui_wrapper.update_key_down(keycode, keymods);
	}

	fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, keymods: KeyMods) {
		self.imgui_wrapper.update_key_up(keycode, keymods);
	}

	fn text_input_event(&mut self, _ctx: &mut Context, val: char) {
		self.imgui_wrapper.update_text(val);
	}

	fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
		self.width = width;
		self.height = height;
		graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, width, height)).expect("window resize error");
	}

	fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: f32, y: f32) {
		self.imgui_wrapper.update_scroll(x, y);
	}
}

fn main() -> ggez::GameResult {
	let cb = ggez::ContextBuilder::new("Color sorting game", "alexrsagen")
		.window_setup(conf::WindowSetup::default()
			.title("Color sorting game")
			.srgb(true)
			.vsync(true)
		).window_mode(conf::WindowMode::default()
			// .fullscreen_type(conf::FullscreenType::Desktop)
			// .maximized(true)
			.dimensions(700.0, 650.0)
			.resizable(true)
		);
	let (ref mut ctx, event_loop) = &mut cb.build()?;

	let hidpi_factor = event_loop.get_primary_monitor().get_hidpi_factor() as f32;
	let state = &mut MainState::new(ctx, hidpi_factor)?;

	event::run(ctx, event_loop, state)
}