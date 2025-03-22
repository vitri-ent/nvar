use std::sync::Arc;

use nokhwa::{
	Camera,
	pixel_format::RgbFormat,
	utils::{ApiBackend, CameraFormat, FrameFormat, RequestedFormat, RequestedFormatType, Resolution}
};
use nvar::{
	ar::{CUDAStream, FaceExpressions, TemporalFilter},
	cv::{ImageView, PixelFormat}
};
use raqote::{DrawOptions, PathBuilder, SolidSource, Source};
use show_image::{ImageInfo, create_window};

#[show_image::main]
fn main() -> anyhow::Result<()> {
	let camera = nokhwa::query(ApiBackend::MediaFoundation)?;
	let camera = &camera[0];
	let mut camera = Camera::new(
		camera.index().clone(),
		RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(CameraFormat::new(Resolution::new(1280, 720), FrameFormat::MJPEG, 60)))
	)?;
	camera.open_stream()?;

	let res = camera.resolution();

	std::env::set_var("NVAR_ROOT", r#"C:\Program Files\NVIDIA Corporation\NVIDIA AR SDK"#);

	let stream = Arc::new(CUDAStream::new()?);
	let mut nvar = FaceExpressions::builder()?
		.with_cuda_stream(Arc::clone(&stream))?
		.with_pose(true)?
		.with_temporal(TemporalFilter::empty())?
		.load()?;

	let mut frame_buffer = vec![0u8; res.width() as usize * res.height() as usize * 3];

	let window = create_window("image", Default::default())?;

	loop {
		let frame = camera.frame()?;
		frame.decode_image_to_buffer::<RgbFormat>(&mut frame_buffer)?;

		let image = ImageView::new_rgb(res.width(), res.height(), PixelFormat::RGB, &mut frame_buffer);

		nvar.run(&image)?;

		let detected = nvar.bounding_boxes().len() == 1;

		let points = nvar.landmarks().to_owned();

		window.run_function(move |mut f| {
			let mut overlay = raqote::DrawTarget::new(res.width() as _, res.height() as _);

			if detected {
				for (i, kp) in points.iter().enumerate() {
					let hash = fxhash::hash32(&i);

					let mut path = PathBuilder::new();
					path.rect(kp.x, kp.y, 3., 3.);
					overlay.fill(
						&path.finish(),
						&Source::Solid(SolidSource::from_unpremultiplied_argb(255, (hash >> 16 & 0xFF) as u8, (hash >> 8 & 0xFF) as u8, (hash & 0xFF) as u8)),
						&DrawOptions::default()
					);
				}
			}

			let image = show_image::Image::from(overlay);
			f.set_overlay("box", &image.as_image_view().unwrap(), true);
		});
		window.set_image("image", show_image::ImageView::new(ImageInfo::rgb8(res.width(), res.height()), &frame_buffer))?;
	}
}
