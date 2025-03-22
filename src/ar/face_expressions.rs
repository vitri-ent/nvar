use std::{num::NonZeroUsize, pin::Pin, sync::Arc};

use super::{CUDAStream, FeatureBase, Point2D, Quaternion, Rect, TemporalFilter, Vector3, sys};
use crate::{
	Result,
	cv::{ComponentType, Image, ImageLayout, MemorySpace, PixelFormat}
};

pub struct FaceExpressionsBuilder {
	feature: FeatureBase,
	stream: Option<Arc<CUDAStream>>
}

impl FaceExpressionsBuilder {
	pub(crate) fn new() -> Result<Self> {
		let feature = FeatureBase::new("FaceExpressions")?;
		Ok(FaceExpressionsBuilder { feature, stream: None })
	}

	pub fn with_cuda_stream(mut self, stream: Arc<CUDAStream>) -> Result<Self> {
		self.feature.set_config("CUDAStream", &*stream)?;
		self.stream = Some(stream);
		Ok(self)
	}

	pub fn with_temporal(mut self, temporal_filter: TemporalFilter) -> Result<Self> {
		self.feature.set_config("Temporal", temporal_filter.bits())?;
		Ok(self)
	}

	pub fn with_pose(mut self, pose: bool) -> Result<Self> {
		self.feature.set_config("PoseMode", u32::from(pose))?;
		Ok(self)
	}

	pub fn with_cheek_puff(mut self, enable: bool) -> Result<Self> {
		self.feature.set_config("EnableCheekPuff", u32::from(enable))?;
		Ok(self)
	}

	pub fn load(mut self) -> Result<FaceExpressions> {
		self.feature.load()?;
		FaceExpressions::new(self.feature, self.stream)
	}
}

pub struct FaceExpressions {
	feature: FeatureBase,
	input_image: Image,
	pose_rotation: Pin<Box<Quaternion>>,
	pose_translation: Pin<Box<Vector3>>,
	expression_coefficients: Vec<f32>,
	expression_zero_point: Vec<f32>,
	expression_scale: Vec<f32>,
	landmarks: Vec<Point2D>,
	landmark_confidence: Vec<f32>,
	bounding_boxes: Pin<Box<sys::NvAR_BBoxes>>,
	bounding_boxes_confidence: Vec<f32>,
	bounding_boxes_data: Vec<Rect>,
	stream: Option<Arc<CUDAStream>>,
	needs_calibration: bool
}

impl FaceExpressions {
	pub fn builder() -> Result<FaceExpressionsBuilder> {
		FaceExpressionsBuilder::new()
	}

	pub(crate) fn new(mut feature: FeatureBase, stream: Option<Arc<CUDAStream>>) -> Result<Self> {
		let mut output_bbox_data = vec![Rect::default(); 25];
		let mut output_bboxes = Box::pin(sys::NvAR_BBoxes {
			boxes: output_bbox_data.as_mut_ptr(),
			max_boxes: output_bbox_data.len() as u8,
			num_boxes: 0
		});
		feature.set_output("BoundingBoxes", output_bboxes.as_mut())?;

		let mut output_bbox_confidence = vec![0.0; 25];
		feature.set_output("BoundingBoxesConfidence", &mut output_bbox_confidence[..])?;

		let landmarks_size = feature.get_config::<u32>("Landmarks_Size")? as usize;

		let mut landmarks = vec![Point2D::default(); landmarks_size];
		feature.set_output("Landmarks", &mut landmarks[..])?;

		let mut landmark_confidence = vec![0.0f32; landmarks_size];
		feature.set_output("LandmarksConfidence", &mut landmark_confidence[..])?;

		let expr_count = feature.get_config::<u32>("ExpressionCount")? as usize;
		let mut expression_coefficients = vec![0.0; expr_count];
		let expression_zero_point = vec![0.0; expr_count];
		let expression_scale = vec![1.0; expr_count];
		feature.set_output("ExpressionCoefficients", &mut expression_coefficients[..])?;

		let mut pose_rotation: Pin<Box<Quaternion>> = Box::pin(Quaternion::default());
		feature.set_output("Pose", pose_rotation.as_mut())?;

		let mut pose_translation = Box::pin(Vector3::default());
		feature.set_output("PoseTranslation", pose_translation.as_mut())?;

		let mut input_image = Image::new(32, 32, PixelFormat::BGR, ComponentType::U8, ImageLayout::Interleaved, MemorySpace::GPU, NonZeroUsize::new(1))?;
		feature.set_input("Image", &mut input_image)?;

		Ok(Self {
			feature,
			input_image,
			pose_rotation,
			pose_translation,
			expression_coefficients,
			expression_scale,
			expression_zero_point,
			landmarks,
			landmark_confidence,
			bounding_boxes: output_bboxes,
			bounding_boxes_confidence: output_bbox_confidence,
			bounding_boxes_data: output_bbox_data,
			stream,
			needs_calibration: true
		})
	}

	pub fn set_temporal(&mut self, filter: TemporalFilter) -> Result<()> {
		self.feature.set_config("Temporal", filter.bits())?;
		Ok(())
	}

	pub fn run(&mut self, image: &Image) -> Result<bool> {
		if self.input_image.width() != image.width() || self.input_image.height() != image.height() {
			let mut input_image = Image::new(
				image.width(),
				image.height(),
				PixelFormat::BGR,
				ComponentType::U8,
				ImageLayout::Interleaved,
				MemorySpace::GPU,
				NonZeroUsize::new(1)
			)?;
			self.feature.set_input("Image", &mut input_image)?;
			self.input_image = input_image;
		}

		if let Some(stream) = &self.stream {
			image.transfer_to_opt(&mut self.input_image, 1.0, Some(stream), None)?;
		} else {
			image.transfer_to(&mut self.input_image)?;
		}

		self.feature.run()?;

		if self.needs_calibration {
			self.needs_calibration = false;

			self.expression_zero_point.copy_from_slice(&self.expression_coefficients);
			for i in 0..self.expression_coefficients.len() {
				self.expression_scale[i] = 1.0 / (1.0 - self.expression_zero_point[i]);
			}

			return Ok(false);
		}

		for i in 0..self.expression_coefficients.len() {
			let temp = self.expression_coefficients[i];
			self.expression_coefficients[i] =
				1.0 - (1.0 - (self.expression_coefficients[i] - self.expression_zero_point[i]).max(0.0) * self.expression_scale[i]);
			self.expression_coefficients[i] = (1.0 * self.expression_coefficients[i] + 0.0 * temp).clamp(0.0, 1.0);
		}

		for x in self.expression_coefficients.iter_mut() {
			*x = x.clamp(0.0, 1.0);
		}

		Ok(true)
	}

	pub fn expressions(&self) -> &[f32] {
		&self.expression_coefficients
	}

	pub fn rotation(&self) -> &Quaternion {
		&self.pose_rotation
	}

	pub fn translation(&self) -> &Vector3 {
		&self.pose_translation
	}

	pub fn landmarks(&self) -> &[Point2D] {
		&self.landmarks
	}

	pub fn landmark_confidence(&self) -> &[f32] {
		&self.landmark_confidence
	}

	pub fn calibrate(&mut self) {
		self.needs_calibration = true;
	}

	pub fn bounding_boxes(&self) -> &[Rect] {
		&self.bounding_boxes_data[..self.bounding_boxes.num_boxes as usize]
	}

	pub fn bounding_boxes_confidence(&self) -> &[f32] {
		&self.bounding_boxes_confidence[..self.bounding_boxes.num_boxes as usize]
	}

	pub fn bounding_boxes_with_confidence(&self) -> Vec<(&Rect, f32)> {
		self.bounding_boxes_data[..self.bounding_boxes.num_boxes as usize]
			.iter()
			.zip(self.bounding_boxes_confidence[..self.bounding_boxes.num_boxes as usize].iter().copied())
			.collect()
	}
}
