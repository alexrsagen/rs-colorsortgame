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

const COLOR_BLUEGRAY: Color = Color::new(0.12941176470588237, 0.1607843137254902, 0.20392156862745098, 1.0);
const COLOR_LIGHT_BLUEGRAY: Color = Color::new(0.5333333333333333, 0.5843137254901961, 0.6549019607843137, 1.0);
const COLOR_PINK: Color = Color::new(0.8627450980392157, 0.19607843137254902, 0.6745098039215687, 1.0);
const COLOR_PURPLE: Color = Color::new(0.6862745098039216, 0.19607843137254902, 0.8627450980392157, 1.0);
const COLOR_VIOLET: Color = Color::new(0.40784313725490196, 0.19607843137254902, 0.8627450980392157, 1.0);
const COLOR_BLUE: Color = Color::new(0.19215686274509805, 0.5215686274509804, 0.788235294117647, 1.0);
const COLOR_TEAL: Color = Color::new(0.23529411764705882, 0.6862745098039216, 0.6392156862745098, 1.0);
const COLOR_GREEN: Color = Color::new(0.2196078431372549, 0.7607843137254902, 0.44313725490196076, 1.0);
const COLOR_OLIVE: Color = Color::new(0.6352941176470588, 0.7607843137254902, 0.2196078431372549, 1.0);
const COLOR_YELLOW: Color = Color::new(0.9568627450980393, 0.796078431372549, 0.3843137254901961, 1.0);
const COLOR_ORANGE: Color = Color::new(0.8627450980392157, 0.45098039215686275, 0.19607843137254902, 1.0);
const COLOR_RED: Color = Color::new(0.8627450980392157, 0.19607843137254902, 0.19607843137254902, 1.0);

struct ColorTubeContent {
	color: Color,
	amount: f32,
}

impl ColorTubeContent {
	fn new(color: Color) -> Self {
		Self{ color, amount: 1.0 }
	}
}

struct ColorTube {
	dimensions: Rect,
	capacity: usize,
	contents: Vec<ColorTubeContent>,
}

impl ColorTube {
	fn new(capacity: usize) -> Self {
		Self {
			dimensions: Rect::new(0.0, 0.0, 50.0, 50.0 * capacity as f32),
			capacity,
			contents: Vec::new()
		}
	}

	fn try_push(mut self, content: ColorTubeContent) -> Self {
		if self.contents.len() < self.capacity {
			self.contents.push(content);
			self
		} else {
			self
		}
	}
}

impl Drawable for ColorTube {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
		let color_border = COLOR_LIGHT_BLUEGRAY;
		let color_bg = COLOR_BLUEGRAY;
		let w_half = (self.dimensions.w / 2.0).floor();

		// Draw border and bottom/first fill level
		{
			let color_fill = if self.contents.len() > 0 {
				self.contents[0].color
			} else {
				color_bg
			};
			
			let amount = if self.contents.len() > 0 {
				self.contents[0].amount
			} else {
				0.0
			};

			// Base border
			// let mode = DrawMode::stroke(2.0);
			// Mesh::new_rectangle(ctx, mode, self.dimensions, color_border)?.draw(ctx, param)?;

			// Remove bottom of base border
			// Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{
			// 	x: self.dimensions.x - 1.0,
			// 	y: self.dimensions.y + self.dimensions.h - w_half,
			// 	w: self.dimensions.w + 2.0,
			// 	h: w_half + 1.0
			// }, color_bg)?.draw(ctx, param)?;

			// Bottom circle border
			// Mesh::new_circle(ctx, mode, Point2::new(
			// 	/* x: */ self.dimensions.x + w_half,
			// 	/* y: */ self.dimensions.y + self.dimensions.h - w_half
			// ), w_half, 0.125, color_border)?.draw(ctx, param)?;

			// Bottom half of fill (circle)
			// Mesh::new_circle(ctx, DrawMode::fill(), Point2::new(
			// 	/* x: */ self.dimensions.x + w_half,
			// 	/* y: */ self.dimensions.y + self.dimensions.h - w_half
			// ), w_half - 1.0, 0.125, color_fill)?.draw(ctx, param)?;

			// Remove top half of border (circle)
			// Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{
			// 	x: self.dimensions.x + 1.0,
			// 	y: self.dimensions.y + self.dimensions.h - self.dimensions.w - 1.0,
			// 	w: self.dimensions.w - 2.0,
			// 	h: 2.0
			// }, color_bg)?.draw(ctx, param)?;

			// Top half of fill
			// let tophalf_maxh = w_half - 1.0;
			// Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{
			// 	x: self.dimensions.x + 1.0,
			// 	y: self.dimensions.y + self.dimensions.h - self.dimensions.w + 1.0,
			// 	w: self.dimensions.w - 2.0,
			// 	h: tophalf_maxh
			// }, color_fill)?.draw(ctx, param)?;

			let mut fill_points = Vec::new();
			let steps = (self.dimensions.w / 4.0).floor() as u32;
			fill_points.push(Point2::new(self.dimensions.x + 1.0, self.dimensions.y + 1.0 + self.dimensions.h - self.dimensions.w));
			for i in 0..=steps {
				fill_points.push(Point2::new(self.dimensions.x + 1.0 + (i as f32 / steps as f32) * (self.dimensions.w - 2.0), self.dimensions.y + 1.0 + self.dimensions.h - w_half + (i as f32 * std::f32::consts::PI / steps as f32).sin() * (w_half - 2.0)));
			}
			fill_points.push(Point2::new(self.dimensions.x + self.dimensions.w - 1.0, self.dimensions.y + 1.0 + self.dimensions.h - self.dimensions.w));
			Mesh::new_polygon(ctx, DrawMode::fill(), &fill_points, color_fill)?.draw(ctx, param)?;

			let mut border_points = Vec::new();
			let steps = (self.dimensions.w / 4.0).floor() as u32;
			border_points.push(Point2::new(self.dimensions.x, self.dimensions.y));
			for i in 0..=steps {
				border_points.push(Point2::new(self.dimensions.x + (i as f32 / steps as f32) * self.dimensions.w, self.dimensions.y + self.dimensions.h - w_half + (i as f32 * std::f32::consts::PI / steps as f32).sin() * w_half));
			}
			border_points.push(Point2::new(self.dimensions.x + self.dimensions.w, self.dimensions.y));
			Mesh::new_polygon(ctx, DrawMode::stroke(2.0), &border_points, color_border)?.draw(ctx, param)?;
		}

		// Draw rest of fill
		{

		}

		Ok(())
	}

    fn dimensions(&self, _ctx: &mut Context) -> Option<Rect> { Some(self.dimensions) }
    fn set_blend_mode(&mut self, _mode: Option<BlendMode>) {}
    fn blend_mode(&self) -> Option<BlendMode> { None }
}

struct MainState {
	imgui_wrapper: ImGuiWrapper,
	hidpi_factor: f32,
	width: f32,
	height: f32,

	tubes: Vec<ColorTube>,
}

impl MainState {
	fn new(mut ctx: &mut Context, hidpi_factor: f32) -> GameResult<MainState> {
		let imgui_wrapper = ImGuiWrapper::new(&mut ctx);
		// let font = Font::new(ctx, "/IBMPlexMono-Regular.ttf")?;
		let capacity = 4;
		let s = MainState {
			imgui_wrapper,
			hidpi_factor,
			width: 1100.0,
			height: 600.0,
			tubes: vec![
				ColorTube::new(capacity).try_push(ColorTubeContent::new(COLOR_RED)),
				ColorTube::new(capacity).try_push(ColorTubeContent::new(COLOR_RED)),
				ColorTube::new(capacity).try_push(ColorTubeContent::new(COLOR_RED)),
				ColorTube::new(capacity).try_push(ColorTubeContent::new(COLOR_RED)),
				ColorTube::new(capacity).try_push(ColorTubeContent::new(COLOR_RED)),
				ColorTube::new(capacity).try_push(ColorTubeContent::new(COLOR_RED)),
				ColorTube::new(capacity).try_push(ColorTubeContent::new(COLOR_RED)),
				ColorTube::new(capacity).try_push(ColorTubeContent::new(COLOR_RED)),
				ColorTube::new(capacity).try_push(ColorTubeContent::new(COLOR_RED)),
				ColorTube::new(capacity).try_push(ColorTubeContent::new(COLOR_RED)),
				ColorTube::new(capacity).try_push(ColorTubeContent::new(COLOR_RED)),
				ColorTube::new(capacity).try_push(ColorTubeContent::new(COLOR_RED)),
				ColorTube::new(capacity).try_push(ColorTubeContent::new(COLOR_RED)),
				ColorTube::new(capacity).try_push(ColorTubeContent::new(COLOR_RED)),
			]
		};
		Ok(s)
	}
}

impl EventHandler for MainState {
	fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
		let tube_width = 50.0;
		let screen_margin = 50.0;
		let tube_margin = 25.0;
		let max_tubes_per_line = ((self.width - screen_margin * 2.0 + tube_margin) / (tube_width + tube_margin)).floor();
		for i in 0..self.tubes.len() {
			let tube = &mut self.tubes[i];
			tube.dimensions.w = tube_width;
			tube.dimensions.h = tube.dimensions.w * tube.capacity as f32;
			tube.dimensions.x = screen_margin + (tube.dimensions.w + tube_margin) * (i as f32 % max_tubes_per_line).floor();
			tube.dimensions.y = screen_margin + (tube.dimensions.h + tube_margin) * (i as f32 / max_tubes_per_line).floor();
		}
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
		graphics::clear(ctx, COLOR_BLUEGRAY);

		// Render graphics
		let param = DrawParam::default();

		// let center_x = (self.width / 2.0).floor();
		// let center_y = (self.height / 2.0).floor();

		for tube in &mut self.tubes {
			tube.draw(ctx, param)?;
		}

		// graphics::draw_queued_text(ctx, param, None, graphics::FilterMode::Linear)?;

		// Render UI
		self.imgui_wrapper.render(ctx, self.hidpi_factor, move |_ui| {
		}).expect("renderer error");

		graphics::present(ctx)?;
		Ok(())
	}

	fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
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
			.dimensions(600.0, 525.0)
			.resizable(true)
		);
	let (ref mut ctx, event_loop) = &mut cb.build()?;

	let hidpi_factor = event_loop.get_primary_monitor().get_hidpi_factor() as f32;
	let state = &mut MainState::new(ctx, hidpi_factor)?;

	event::run(ctx, event_loop, state)
}