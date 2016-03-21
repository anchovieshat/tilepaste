#[macro_use]
extern crate glium;
extern crate image;

pub mod map;

use glium::{DisplayBuild, Surface};
use std::io::Cursor;

use map::{Map, View};

#[derive(Copy, Clone)]
struct Vert {
	position: [f32; 2],
	tex_coords: [f32; 2],
}

implement_vertex!(Vert, position, tex_coords);

fn atlas_verts(entry: usize) -> Vec<Vert> {
	let num_entries = 4;

	let scalar = 1.0 / ((num_entries as f32) / 2.0);

	let base_x = entry % (num_entries / 2);
	let base_y = entry / (num_entries / 2);
	let base_x = (base_x as f32) * scalar;
	let base_y = (base_y as f32) * scalar;

	let bottom_left =  [base_x, base_y];
	let bottom_right = [base_x + scalar, base_y];
	let top_left = 	   [base_x, base_y + scalar];
	let top_right =	   [base_x + scalar, base_y + scalar];

	let vert1 = Vert { position: [-0.5, -0.5], tex_coords: bottom_left };
	let vert2 = Vert { position: [-0.5,  0.5], tex_coords: top_left };
	let vert3 = Vert { position: [ 0.5, -0.5], tex_coords: bottom_right };
	let vert4 = Vert { position: [ 0.5, -0.5], tex_coords: bottom_right };
	let vert5 = Vert { position: [-0.5,  0.5], tex_coords: top_left };
	let vert6 = Vert { position: [ 0.5,  0.5], tex_coords: top_right };
	vec![vert1, vert2, vert3, vert4, vert5, vert6]
}

fn handle_input(key: Option<glium::glutin::VirtualKeyCode>, state: glium::glutin::ElementState, view: &mut View) {
	if key.is_some() && state == glium::glutin::ElementState::Pressed {
		let key = key.unwrap();
		match key {
			glium::glutin::VirtualKeyCode::W => { view.up(); },
			glium::glutin::VirtualKeyCode::S => { view.down(); },
			glium::glutin::VirtualKeyCode::A => { view.left(); },
			glium::glutin::VirtualKeyCode::D => { view.right(); },
			_ => (),
		}
	}
}

fn main() {
	let display = glium::glutin::WindowBuilder::new()
		.with_dimensions(640, 480)
		.with_title(format!("TilePaste"))
		.with_vsync()
		.build_glium().unwrap();

	let image = image::load(Cursor::new(&include_bytes!("../assets/atlas.png")[..]), image::PNG).unwrap().to_rgba();
	let image_dimensions = image.dimensions();
	let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
	let texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();

	let map = Map::new(100, 100);

	let one = atlas_verts(0);
	let two = atlas_verts(1);
	let three = atlas_verts(2);
	let four = atlas_verts(3);
	let one_buffer = glium::VertexBuffer::immutable(&display, &one).unwrap();
	let two_buffer = glium::VertexBuffer::immutable(&display, &two).unwrap();
	let three_buffer = glium::VertexBuffer::immutable(&display, &three).unwrap();
	let four_buffer = glium::VertexBuffer::immutable(&display, &four).unwrap();

	let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

	let vert_shader_src = r#"
		#version 140

		in vec2 position;
		in vec2 tex_coords;
		out vec2 v_tex_coords;

		uniform mat4 matrix;
		void main() {
			v_tex_coords = tex_coords;
			gl_Position = matrix * vec4(position * 0.225, 0.0, 1.0);
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

	let mut view = View::new(0, 0, 10, 10);

	loop {
		let mut target = display.draw();
		target.clear_color(0.0, 0.0, 1.0, 1.0);

		for x in 0..view.width {
			for y in 0..view.height {
				let tile = map.uniform(x, y, x + view.x, y + view.y, view.width as u32, view.height as u32);
				let tile_uniforms = uniform! {
					matrix: tile.0,
					tex: texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
				};

				let buffer;
				match tile.1 {
					0 => { buffer = &one_buffer; },
					1 => { buffer = &two_buffer; },
					2 => { buffer = &three_buffer; },
					3 => { buffer = &four_buffer; },
					_ => { buffer = &one_buffer; },
				}
				target.draw(buffer, &indices, &program, &tile_uniforms, &Default::default()).unwrap();
			}
		}

		target.finish().unwrap();

		for event in display.poll_events() {
			match event {
				glium::glutin::Event::Closed => return,
				glium::glutin::Event::KeyboardInput(state, _, key) => handle_input(key, state, &mut view),
				_ => (),
			}
		}
	}
}
