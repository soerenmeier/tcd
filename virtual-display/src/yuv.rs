pub fn bgra_to_yuv420(
	data: &[u8],
	total_width: u32,
	_total_height: u32,
	x: u32,
	y: u32,
	width: u32,
	height: u32,
	// out should be the correct size ((width * height * 3) / 2)
	yuv: &mut [u8]
) {
	assert_eq!(yuv.len() as u32, (width * height * 3) / 2);

	let frame_size = width as usize * height as usize;
	let chroma_size = frame_size / 4;

	let u_offset = frame_size;
	let v_offset = frame_size + chroma_size;
	let mut uv_index = 0;

	// maybe change the values https://stackoverflow.com/questions/1737726/how-to-perform-rgb-yuv-conversion-in-c-c/1737741#1737741
	for h in 0..height {
		for w in 0..width {
			let i = h * width + w;
			// 4 = bgra
			let di = ((y + h) * total_width + x + w) * 4;
			let b = data[di as usize] as i32;
			let g = data[di as usize + 1] as i32;
			let r = data[di as usize + 2] as i32;

			let y = (77 * r + 150 * g + 29 * b + 128) >> 8;
			yuv[i as usize] = y.clamp(0, 255) as u8;

			if h % 2 == 0 && w % 2 == 0 {
				let u = ((-43 * r - 84 * g + 127 * b + 128) >> 8) + 128;
				let v = ((127 * r - 106 * g - 21 * b + 128) >> 8) + 128;

				yuv[u_offset + uv_index] = u.clamp(0, 255) as u8;
				yuv[v_offset + uv_index] = v.clamp(0, 255) as u8;
				uv_index += 1;
			}
		}
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	use std::fs::{self, File};

	use png::{Decoder, ColorType};


	#[test]
	fn test_bgra_to_yuv420() {
		let file = File::open("./test-assets/rainbow.png").unwrap();
		let decoder = Decoder::new(file);
		let mut reader = decoder.read_info().unwrap();
		let mut buf = vec![0; reader.output_buffer_size()];
		let info = reader.next_frame(&mut buf).unwrap();
		buf.truncate(info.buffer_size());
		assert_eq!(info.color_type, ColorType::Rgba);
		assert_eq!(info.width, 16);
		assert_eq!(info.height, 16);
		assert_eq!(buf.len() as u32, info.width * info.height * 4);

		for rgba in buf.chunks_exact_mut(4) {
			rgba.swap(2, 0);
		}

		let yuv_len = (info.width * info.height * 3) / 2;
		let mut yuv_data = vec![0; yuv_len as usize];
		bgra_to_yuv420(
			&buf,
			info.width,
			info.height,
			0,
			0,
			info.width,
			info.height,
			&mut yuv_data
		);

		// fs::write("./test-assets/test.yuv", &yuv_data).unwrap();

		let yuv_data_real = fs::read("./test-assets/rainbow.yuv").unwrap();
		assert_eq!(yuv_data.len(), yuv_data_real.len());
		assert_eq!(yuv_data, yuv_data_real);
	}
}