pub mod sys;

use std::{
	ffi::c_void,
	marker::PhantomData,
	num::NonZeroUsize,
	ops::Deref,
	ptr::{self, NonNull}
};

pub use self::sys::{
	NvCVImage_ColorSpace as ColorSpace, NvCVImage_ComponentType as ComponentType, NvCVImage_Layout as ImageLayout, NvCVImage_MemorySpace as MemorySpace,
	NvCVImage_PixelFormat as PixelFormat
};
use crate::{Result, ar::CUDAStream};

impl ImageLayout {
	#[inline]
	pub fn is_planar(&self) -> bool {
		(*self as u8) & 0b1 == 0b1
	}

	#[inline]
	pub fn is_interleaved(&self) -> bool {
		!self.is_planar()
	}
}

enum ImageInner {
	Allocated(*mut sys::NvCVImage),
	Owned(Box<sys::NvCVImage>)
}

pub struct Image {
	inner: ImageInner,
	drop: bool
}

unsafe impl Send for Image {}

impl Image {
	pub fn new(
		width: u32,
		height: u32,
		format: PixelFormat,
		r#type: ComponentType,
		layout: ImageLayout,
		memory_space: MemorySpace,
		alignment: Option<NonZeroUsize>
	) -> Result<Image> {
		let mut img = sys::NvCVImage::default();
		unsafe {
			sys::NvCVImage_Alloc(
				crate::nvcv_lib_handle(),
				&mut img,
				width,
				height,
				format,
				r#type,
				layout,
				memory_space,
				alignment.map(|c| c.get() as u32).unwrap_or(0)
			)
		}?;
		Ok(Image {
			inner: ImageInner::Owned(Box::new(img)),
			drop: true
		})
	}

	pub(crate) fn from_ptr(ptr: *mut sys::NvCVImage, drop: bool) -> Self {
		Image {
			inner: ImageInner::Allocated(ptr),
			drop
		}
	}

	#[inline]
	pub(crate) fn ref_inner(&self) -> &sys::NvCVImage {
		unsafe { &*self.as_ptr() }
	}

	#[inline]
	pub(crate) fn as_ptr(&self) -> *mut sys::NvCVImage {
		match &self.inner {
			ImageInner::Allocated(p) => *p,
			ImageInner::Owned(p) => (p.as_ref() as *const sys::NvCVImage).cast_mut()
		}
	}

	#[inline]
	pub fn width(&self) -> u32 {
		self.ref_inner().width
	}

	#[inline]
	pub fn height(&self) -> u32 {
		self.ref_inner().height
	}

	#[inline]
	pub fn stride(&self) -> isize {
		self.ref_inner().pitch as isize
	}

	#[inline]
	pub fn pixels_ptr(&self) -> Option<NonNull<c_void>> {
		NonNull::new(self.ref_inner().pixels)
	}

	#[inline]
	pub fn pixels(&self) -> &[u8] {
		unsafe {
			std::slice::from_raw_parts(self.pixels_ptr().unwrap().as_ptr().cast(), (self.height() as isize * self.stride()) as usize * self.pixel_bytes())
		}
	}

	#[inline]
	pub fn pixel_format(&self) -> PixelFormat {
		self.ref_inner().pixel_format
	}

	#[inline]
	pub fn component_type(&self) -> ComponentType {
		self.ref_inner().component_type
	}

	#[inline]
	pub fn layout(&self) -> ImageLayout {
		self.ref_inner().planar
	}

	#[inline]
	pub fn memory_space(&self) -> MemorySpace {
		self.ref_inner().gpu_mem
	}

	#[inline]
	pub fn color_space(&self) -> ColorSpace {
		self.ref_inner().color_space
	}

	#[inline]
	pub fn pixel_bytes(&self) -> usize {
		self.ref_inner().pixel_bytes as usize
	}

	#[inline]
	pub fn component_bytes(&self) -> usize {
		self.ref_inner().component_bytes as usize
	}

	#[inline]
	pub fn num_components(&self) -> usize {
		self.ref_inner().num_components as usize
	}

	pub fn transfer_to(&self, dst: &mut Image) -> Result<()> {
		unsafe { sys::NvCVImage_Transfer(crate::nvcv_lib_handle(), self.as_ptr().cast_const(), dst.as_ptr(), 1.0, ptr::null_mut(), ptr::null_mut()) }?;
		Ok(())
	}

	pub fn scale_to(&self, dst: &mut Image, scale: f32) -> Result<()> {
		unsafe { sys::NvCVImage_Transfer(crate::nvcv_lib_handle(), self.as_ptr().cast_const(), dst.as_ptr(), scale, ptr::null_mut(), ptr::null_mut()) }?;
		Ok(())
	}

	pub fn transfer_to_opt(&self, dst: &mut Image, scale: f32, stream: Option<&CUDAStream>, tmp: Option<&mut Image>) -> Result<()> {
		unsafe {
			sys::NvCVImage_Transfer(
				crate::nvcv_lib_handle(),
				self.as_ptr().cast_const(),
				dst.as_ptr(),
				scale,
				stream.map(|c| c.0).unwrap_or_else(ptr::null_mut),
				tmp.map(|c| c.as_ptr()).unwrap_or_else(ptr::null_mut)
			)
		}?;
		Ok(())
	}

	pub fn view_rect(&self, x: u32, y: u32, width: u32, height: u32) -> Result<Image> {
		let view = Image::new(width, height, self.pixel_format(), self.component_type(), self.layout(), self.memory_space(), NonZeroUsize::new(1))?;
		unsafe {
			sys::NvCVImage_TransferRect(
				crate::nvcv_lib_handle(),
				self.as_ptr(),
				&sys::NvCVRect2i {
					x: x as i32,
					y: y as i32,
					width: width as i32,
					height: height as i32
				},
				view.as_ptr(),
				ptr::null(),
				1.0,
				ptr::null_mut(),
				ptr::null_mut()
			)
		}?;
		Ok(view)
	}

	pub fn view(&self, x: u32, y: u32, width: u32, height: u32) -> Result<ImageView<'_>> {
		assert!(self.width() > x + width);
		assert!(self.height() > y + height);

		let mut view = sys::NvCVImage::default();
		unsafe { sys::NvCVImage_InitView(crate::nvcv_lib_handle(), &mut view, self.as_ptr(), x as i32, y as i32, width, height) }?;
		dbg!(view.width, view.height, view.pixel_bytes, view.pitch);
		Ok(ImageView {
			image: Image {
				inner: ImageInner::Owned(Box::new(view)),
				drop: true
			},
			_phantom: PhantomData
		})
	}
}

impl Clone for Image {
	fn clone(&self) -> Self {
		let inf = self.ref_inner();
		let mut new = Image::new(inf.width, inf.height, inf.pixel_format, inf.component_type, inf.planar, inf.gpu_mem, None)
			.expect("Failed to allocate new Image buffer");
		self.transfer_to(&mut new).expect("Failed to transfer image data to new buffer");
		new
	}

	fn clone_from(&mut self, source: &Self) {
		source.transfer_to(self).expect("Failed to transfer image data")
	}
}

impl Drop for Image {
	fn drop(&mut self) {
		if self.drop {
			match &mut self.inner {
				ImageInner::Allocated(p) => {
					unsafe { sys::NvCVImage_Destroy(crate::nvcv_lib_handle(), *p) }.unwrap();
				}
				ImageInner::Owned(p) => {
					unsafe { sys::NvCVImage_Dealloc(crate::nvcv_lib_handle(), p.as_mut()) }.unwrap();
				}
			}
		}
	}
}

pub struct ImageView<'i> {
	image: Image,
	_phantom: PhantomData<&'i ()>
}

impl<'i> ImageView<'i> {
	pub fn new_rgb(width: u32, height: u32, format: PixelFormat, data: &'i mut [u8]) -> ImageView<'i> {
		let mut image = sys::NvCVImage::default();
		image.width = width;
		image.height = height;
		image.pixel_format = format;
		image.num_components = 3;
		image.pixel_bytes = 3;
		image.component_bytes = 1;
		image.component_type = ComponentType::U8;
		image.color_space = ColorSpace::empty();
		image.pitch = width as i32 * 3;
		image.gpu_mem = MemorySpace::CPU;
		image.pixels = data.as_mut_ptr().cast();
		image.planar = ImageLayout::Interleaved;
		ImageView {
			image: Image {
				inner: ImageInner::Owned(Box::new(image)),
				drop: false
			},
			_phantom: PhantomData
		}
	}
}

impl<'i> Deref for ImageView<'i> {
	type Target = Image;

	fn deref(&self) -> &Self::Target {
		&self.image
	}
}
