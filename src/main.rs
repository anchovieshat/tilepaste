#[macro_use]
extern crate glium;
extern crate image;
extern crate glium_text;

pub mod utils;
pub mod map;
pub mod tile;
pub mod player;
pub mod vert;

use glium::{DisplayBuild, Surface};

use map::Map;
use player::Player;
use tile::TileAtlas;

fn handle_input(key: Option<glium::glutin::VirtualKeyCode>, state: glium::glutin::ElementState, player: &mut Player) -> bool {
	if key.is_some() && state == glium::glutin::ElementState::Pressed {
		let key = key.unwrap();
		match key {
			glium::glutin::VirtualKeyCode::W => { player.up(); return false; },
			glium::glutin::VirtualKeyCode::S => { player.down(); return false; },
			glium::glutin::VirtualKeyCode::A => { player.left(); return false; },
			glium::glutin::VirtualKeyCode::D => { player.right(); return false; },
			glium::glutin::VirtualKeyCode::Q => { return true; }
			_ => { return false; },
		}
	}
	false
}

fn main() {
	let width = 800;
	let height = 800;

	let display = glium::glutin::WindowBuilder::new()
		.with_dimensions(width, height)
		.with_title(format!("TilePaste"))
		.with_vsync()
		.build_glium().unwrap();

	let ratio = width as f32 / height as f32;
	let atlas = TileAtlas::new(&display, 16, 16);
	let mut map = Map::new(11, 11, 11, 11, &atlas);

	let vert_shader_src = r#"
		#version 140

		in vec2 position;
		in vec2 tex_coords;
		out vec2 v_tex_coords;

		uniform mat4 matrix;
		void main() {
			v_tex_coords = tex_coords;
			gl_Position = matrix * vec4(position * 0.18, 0.0, 1.0);
		}
	"#;

	let frag_shader_src = r#"
		#version 140

		in vec2 v_tex_coords;
		out vec4 color;

		uniform sampler2D tex;
		void main() {
			color = texture(tex, v_tex_coords);
			if (color.a == 0.0) { discard; }
		}
	"#;

	let program = glium::Program::from_source(&display, vert_shader_src, frag_shader_src, None).unwrap();

	let mut player = Player::new(13, &atlas, 0, 0);

	let text_system = glium_text::TextSystem::new(&display);
	let font_file = std::fs::File::open(&std::path::Path::new("assets/font.ttf")).unwrap();
	let font = glium_text::FontTexture::new(&display, font_file, 24).unwrap();
	let mut score = 0;

	loop {
		let mut target = display.draw();
		target.clear_color(0.0, 0.0, 1.0, 1.0);

		if map.get(player.x as u32, player.y as u32).unwrap().tex_id == 14 {
			map.set(player.x as u32, player.y as u32, 12);
			score += 1;
		}

		map.draw(&mut target, &program, ratio);
		player.draw(&mut target, &program, &map, ratio);

		let score_text = glium_text::TextDisplay::new(&text_system, &font, format!("score: {}", score).as_str());
		let title_text = glium_text::TextDisplay::new(&text_system, &font, "TilePaste");

		let score_matrix = [
			[0.05 / ratio, 0.0, 0.0, 0.0],
			[0.0, 0.05, 0.0, 0.0],
			[0.0, 0.0, 1.0, 0.0],
			[0.75, -0.99, 0.0, 1.0],
		];

		let title_matrix = [
			[0.05 / ratio, 0.0, 0.0, 0.0],
			[0.0, 0.05, 0.0, 0.0],
			[0.0, 0.0, 1.0, 0.0],
			[-1.0, -0.99, 0.0, 1.0],
		];

		glium_text::draw(&score_text, &text_system, &mut target, score_matrix, (1.0, 1.0, 0.0, 1.0));
		glium_text::draw(&title_text, &text_system, &mut target, title_matrix, (1.0, 1.0, 0.0, 1.0));

		target.finish().unwrap();

		for event in display.poll_events() {
			match event {
				glium::glutin::Event::Closed => return,
				glium::glutin::Event::KeyboardInput(state, _, key) => if handle_input(key, state, &mut player) { return; },
				_ => (),
			}
		}
	}
}
