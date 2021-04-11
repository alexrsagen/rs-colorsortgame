extern crate rand;
extern crate ggez;
extern crate winit;

mod imgui_wrapper;
mod colors;
mod color_tube;

use imgui::*;
use ggez::{input, mint, nalgebra, Context, ContextBuilder, GameResult};
use ggez::conf::{self, NumSamples};
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::{self, Drawable, Font, Scale, DrawParam, Text, TextFragment};
use nalgebra::Point2;
use rand::seq::SliceRandom;
use rand::thread_rng;
use crate::imgui_wrapper::ImGuiWrapper;
use crate::colors::*;
use crate::color_tube::{ColorTube, ColorTubeContent};

// TODO: persist settings on filesystem

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

const WINDOW_WIDTH: f32 = 700.0;
const WINDOW_HEIGHT: f32 = 650.0;

const TUBE_WIDTH: f32 = 50.0;
const SCREEN_MARGIN: f32 = 50.0;
const TUBE_MARGIN: f32 = 25.0;

struct Settings {
	full_screen: bool,
}

impl Settings {
	fn new() -> Self {
		Self {
			full_screen: false
		}
	}
}

pub struct MenuState {
	settings: Settings,
	show_settings: bool,
	full_screen_changed: bool,
	skip_level: bool,
	quit: bool,
}

impl MenuState {
	fn new() -> Self {
		Self {
			settings: Settings::new(),
			full_screen_changed: false,
			show_settings: false,
			skip_level: false,
			quit: false,
		}
	}
}

struct MainState {
	imgui_wrapper: ImGuiWrapper,
	hidpi_factor: f32,
	font: Font,
	width: f32,
	height: f32,
	mouse_pos: mint::Point2<f32>,
	menu_state: MenuState,

	pre_full_screen_pos: winit::dpi::LogicalPosition,
	pre_full_screen_size: (f32, f32),
	full_screen_bug_reset_window_scale: bool,
	full_screen_bug_reset_window_pos: bool,

	tube_capacity: f32,
	tubes: Vec<ColorTube>,
	tubes_factor: usize,
	selected_tube: Option<usize>,

	level: usize,
}

impl MainState {
	fn new(mut ctx: &mut Context, hidpi_factor: f32) -> GameResult<MainState> {
		let imgui_wrapper = ImGuiWrapper::new(&mut ctx);
		let (width, height) = graphics::drawable_size(ctx);

		let mut s = MainState {
			imgui_wrapper,
			hidpi_factor,
			font: Font::new_glyph_font_bytes(ctx, include_bytes!("../IBMPlexMono-Regular.ttf"))?,
			width,
			height,
			mouse_pos: input::mouse::position(ctx),
			menu_state: MenuState::new(),

			pre_full_screen_pos: winit::dpi::LogicalPosition::new(0.0, 0.0),
			pre_full_screen_size: (WINDOW_WIDTH, WINDOW_HEIGHT),
			full_screen_bug_reset_window_scale: false,
			full_screen_bug_reset_window_pos: false,

			tube_capacity: 4.0,
			tubes: Vec::new(),
			tubes_factor: 1,
			selected_tube: None,

			level: 1,
		};
		s.new_tubes();
		Ok(s)
	}

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

	fn skip_level(&mut self) {
		self.level += 1;
		self.new_tubes();
	}
}

impl EventHandler for MainState {
	fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
		// Handle menu state
		if self.menu_state.quit {
			self.menu_state.quit = false;
			event::quit(ctx);
			return Ok(());
		}
		if self.menu_state.skip_level {
			self.menu_state.skip_level = false;
			self.skip_level();
		}

		let win = graphics::window(ctx);
		let current_monitor = win.get_current_monitor();
		let monitor_dpi_factor = current_monitor.get_hidpi_factor();
		let monitor_size = current_monitor.get_dimensions().to_logical(monitor_dpi_factor);
		if self.menu_state.full_screen_changed {
			self.full_screen_bug_reset_window_scale = true;

			if self.menu_state.settings.full_screen {
				self.pre_full_screen_pos = win.get_position()
					.unwrap_or(winit::dpi::LogicalPosition::new(0.0, 0.0));

				graphics::set_fullscreen(ctx, conf::FullscreenType::Desktop)?;
				graphics::set_drawable_size(ctx, monitor_size.width as f32, monitor_size.height as f32)?;
			} else {
				graphics::set_fullscreen(ctx, conf::FullscreenType::Windowed)?;
				let (size_w, size_h) = self.pre_full_screen_size;
				graphics::set_drawable_size(ctx, size_w, size_h)?;
			}
		} else if self.full_screen_bug_reset_window_scale {
			self.full_screen_bug_reset_window_scale = false;
			self.full_screen_bug_reset_window_pos = true;

			// workaround for ggez(?) bug, where:
			// - window is not always correct size when entering fullscreen mode
			win.set_maximized(self.menu_state.settings.full_screen);
		} else if self.full_screen_bug_reset_window_pos {
			self.full_screen_bug_reset_window_pos = false;

			// workaround for ggez(?) bug, where:
			// - window can be slightly offset after entering fullscreen mode
			// - title bar is out of view after restoring to windowed mode
			win.set_position(if self.menu_state.settings.full_screen {
				current_monitor.get_position().to_logical(monitor_dpi_factor)
			} else {
				self.pre_full_screen_pos
			});
		}

		// Main game logic
		let tube_count = self.tubes.len();
		let max_cols = ((self.width - SCREEN_MARGIN * 2.0 + TUBE_MARGIN) / (TUBE_WIDTH + TUBE_MARGIN)).floor();
		let cols = (tube_count as f32 / self.tubes_factor as f32).ceil().min(max_cols).max(1.0);
		let total_w = cols * (TUBE_WIDTH + TUBE_MARGIN) - TUBE_MARGIN;

		let rows = (tube_count as f32 / cols).ceil();
		let total_h = rows * (self.tube_capacity * TUBE_WIDTH + TUBE_MARGIN);

		let mousedown = input::mouse::button_pressed(ctx, MouseButton::Left);

		for i in 0..tube_count {
			let (tubes_before, tubes_after) = self.tubes.split_at_mut(i);
			let tubes_after = tubes_after.split_first_mut();
			let (tube, tubes_after) = tubes_after.unwrap();

			// Update dimensions
			tube.dimensions.w = TUBE_WIDTH;
			tube.dimensions.h = tube.dimensions.w * tube.capacity;
			tube.dimensions.x = SCREEN_MARGIN + (self.width - SCREEN_MARGIN * 2.0) / 2.0 - total_w / 2.0 + (tube.dimensions.w + TUBE_MARGIN) * (i as f32 % cols).floor();
			tube.dimensions.y = SCREEN_MARGIN + (self.height - SCREEN_MARGIN * 2.0 + TUBE_MARGIN) / 2.0 - total_h / 2.0 + (tube.dimensions.h + TUBE_MARGIN) * (i as f32 / cols).floor();

			if !self.menu_state.show_settings {
				// Detect hover
				let hovered = self.mouse_pos.x >= tube.dimensions.x &&
					self.mouse_pos.x <= tube.dimensions.x + tube.dimensions.w &&
					self.mouse_pos.y >= tube.dimensions.y &&
					self.mouse_pos.y <= tube.dimensions.y + tube.dimensions.h;

				// Store previous mouse states
				let has_selected_tube = self.selected_tube.is_some();
				let is_selected_tube = has_selected_tube && self.selected_tube.unwrap() == i;
				let was_mousedown = tube.mousedown && !mousedown;
				let was_clicked = was_mousedown && hovered;

				// Detect mouse states
				tube.mousedown = mousedown && (tube.mousedown || tube.hovered);
				tube.clicked = is_selected_tube || was_clicked;
				tube.hovered = !mousedown && hovered;

				// Handle click
				if is_selected_tube && was_clicked {
					// Deselect current tube
					self.selected_tube = None;
				} else if tube.clicked && has_selected_tube && !is_selected_tube {
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
				} else if tube.clicked && !has_selected_tube {
					// Select current tube
					self.selected_tube = Some(i);
				}
			}
		}

		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
		graphics::clear(ctx, COLOR_BG);

		let complete_pct = self.complete_pct();
		let (width, height) = (self.width, self.height);

		// Draw tubes
		let param = DrawParam::default();
		for tube in &mut self.tubes {
			tube.draw(ctx, param)?;
		}

		// Draw total completed text
		let completed_color = if complete_pct == 1.0 {
			COLOR_LIGHTGREEN
		} else if complete_pct >= 0.75 {
			COLOR_CYAN
		} else if complete_pct >= 0.5 {
			COLOR_YELLOW
		} else if complete_pct >= 0.25 {
			COLOR_ORANGE
		} else {
			COLOR_RED
		};
		let mut pcttext = Text::new(format!("Level {} (", self.level));
		pcttext.add(TextFragment::new(format!("{}% completed", (complete_pct * 100.0).floor())).color(completed_color));
		pcttext.add(TextFragment::new(")"));
		pcttext.set_font(self.font, Scale::uniform(18.0));
		let pcttext_w = pcttext.width(ctx) as f32;
		graphics::queue_text(ctx, &pcttext, Point2::new(width / 2.0 - pcttext_w / 2.0, SCREEN_MARGIN), Some(graphics::WHITE));

		// Draw all queued text
		graphics::draw_queued_text(ctx, param, None, graphics::FilterMode::Linear)?;

		// Draw UI
		self.imgui_wrapper.render(ctx, self.hidpi_factor, &mut self.menu_state, move |ui, state| {
			// Top/main menu bar
			if let Some(menu_bar) = ui.begin_main_menu_bar() {
				if let Some(game_menu) = ui.begin_menu(im_str!("Game"), true) {
					let item = MenuItem::new(im_str!("Settings"));
					if item.build(ui) {
						state.show_settings = true;
					}

					let item = MenuItem::new(im_str!("Exit game"));
					state.quit = item.build(ui);

					game_menu.end(ui);
				}

				if let Some(level_menu) = ui.begin_menu(im_str!("Level"), true) {
					let item = MenuItem::new(if complete_pct == 1.0 {
						im_str!("Next level")
					} else {
						im_str!("Skip level")
					});
					state.skip_level = item.build(ui);

					level_menu.end(ui);
				}

				menu_bar.end(ui);
			}

			// Settings window
			if state.show_settings {
				if let Some(settings_window) = {
					let window_w = 300.0;
					let window_h = window_w * 1.25;
					Window::new(im_str!("Settings"))
						.size([window_w, window_h], Condition::Appearing)
						.position([width / 2.0 - window_w / 2.0, height / 2.0 - window_h / 2.0], Condition::Appearing)
						.opened(&mut state.show_settings)
						.collapsible(false)
						.focused(true)
						.begin(ui)
				} {
					state.full_screen_changed = ui.checkbox(im_str!("Fullscreen"), &mut state.settings.full_screen);

					settings_window.end(ui);
				}
			}
		}).expect("renderer error");

		graphics::present(ctx)
	}

	fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
		self.mouse_pos.x = x;
		self.mouse_pos.y = y;
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

fn main() -> GameResult {
	let cb = ContextBuilder::new("Color sorting game", "alexrsagen")
		.window_setup(conf::WindowSetup::default()
			.title("Color sorting game")
			.srgb(true)
			.vsync(true)
			.samples(NumSamples::Eight)
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