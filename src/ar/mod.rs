use std::{
	ffi::{CStr, CString},
	pin::Pin
};

use crate::{Result, cv::Image};

pub mod sys;

pub use self::sys::{
	NvAR_FaceMesh as FaceMesh, NvAR_Frustum as Frustum, NvAR_Point2f as Point2D, NvAR_Point3f as Point3D, NvAR_Quaternion as Quaternion, NvAR_Rect as Rect,
	NvAR_RenderingParams as RenderingParams, NvAR_TemporalFilter as TemporalFilter, NvAR_TrackingBBox as TrackingBoundingBox, NvAR_Vector2f as Vector2,
	NvAR_Vector3f as Vector3, NvAR_Vector3u16 as Vector3U16
};

mod face_expressions;
pub use self::face_expressions::{FaceExpressions, FaceExpressionsBuilder};

pub struct CUDAStream(pub(crate) sys::CUstream);

unsafe impl Send for CUDAStream {}
unsafe impl Sync for CUDAStream {}

impl CUDAStream {
	pub fn new() -> Result<Self> {
		Ok(CUDAStream(unsafe { sys::NvAR_CudaStreamCreate(crate::nvar_lib_handle())? }))
	}
}

impl Drop for CUDAStream {
	fn drop(&mut self) {
		unsafe { sys::NvAR_CudaStreamDestroy(crate::nvar_lib_handle(), self.0) }.unwrap();
	}
}

pub(crate) struct FeatureBase(sys::NvAR_FeatureHandle);

impl FeatureBase {
	pub fn new(feature_name: &'static str) -> Result<FeatureBase> {
		let feature_name = CString::new(feature_name).unwrap();
		let ptr = unsafe { sys::NvAR_Create(crate::nvar_lib_handle(), feature_name.as_ptr()) }?;
		Ok(FeatureBase(ptr))
	}

	pub fn set_config<'s, T: SetNvARValue + 's>(&'s mut self, option: &'static str, value: T) -> Result<()> {
		let option = CString::new(format!("NvAR_Parameter_Config_{option}")).unwrap();
		value.set(option.as_ptr(), self.0)
	}
	pub fn set_input<'s, T: SetNvARValue + 's>(&'s mut self, option: &'static str, value: T) -> Result<()> {
		let option = CString::new(format!("NvAR_Parameter_Input_{option}")).unwrap();
		value.set(option.as_ptr(), self.0)
	}
	pub fn set_output<'s, T: SetNvARValue + 's>(&'s mut self, option: &'static str, value: T) -> Result<()> {
		let option = CString::new(format!("NvAR_Parameter_Output_{option}")).unwrap();
		value.set(option.as_ptr(), self.0)
	}
	pub fn set_in_out<'s, T: SetNvARValue + 's>(&'s mut self, option: &'static str, value: T) -> Result<()> {
		let option = CString::new(format!("NvAR_Parameter_InOut_{option}")).unwrap();
		value.set(option.as_ptr(), self.0)
	}

	pub fn get_config<'s, T: GetNvARValue + 's>(&'s self, option: &'static str) -> Result<T> {
		let option = CString::new(format!("NvAR_Parameter_Config_{option}")).unwrap();
		T::get(option.as_ptr(), self.0)
	}

	pub fn load(&mut self) -> Result<()> {
		unsafe { sys::NvAR_Load(crate::nvar_lib_handle(), self.0) }?;
		Ok(())
	}

	pub fn run(&self) -> Result<()> {
		unsafe { sys::NvAR_Run(crate::nvar_lib_handle(), self.0) }?;
		Ok(())
	}
}

impl Drop for FeatureBase {
	fn drop(&mut self) {
		unsafe { sys::NvAR_Destroy(crate::nvar_lib_handle(), self.0) }.unwrap();
	}
}

pub trait GetNvARValue: Sized {
	fn get(name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<Self>;
}
impl GetNvARValue for u32 {
	fn get(name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<Self> {
		unsafe { sys::NvAR_GetU32(crate::nvar_lib_handle(), feature, name) }
	}
}
impl GetNvARValue for i32 {
	fn get(name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<Self> {
		unsafe { sys::NvAR_GetS32(crate::nvar_lib_handle(), feature, name) }
	}
}
impl GetNvARValue for f32 {
	fn get(name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<Self> {
		unsafe { sys::NvAR_GetF32(crate::nvar_lib_handle(), feature, name) }
	}
}
impl GetNvARValue for f64 {
	fn get(name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<Self> {
		unsafe { sys::NvAR_GetF64(crate::nvar_lib_handle(), feature, name) }
	}
}
impl GetNvARValue for u64 {
	fn get(name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<Self> {
		unsafe { sys::NvAR_GetU64(crate::nvar_lib_handle(), feature, name) }
	}
}
impl GetNvARValue for &CStr {
	fn get(name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<Self> {
		let ptr = unsafe { sys::NvAR_GetString(crate::nvar_lib_handle(), feature, name) }?;
		Ok(unsafe { CStr::from_ptr(ptr) })
	}
}
impl GetNvARValue for &[f32] {
	fn get(name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<Self> {
		unsafe { sys::NvAR_GetF32Array(crate::nvar_lib_handle(), feature, name) }
	}
}

pub trait SetNvARValue {
	fn set(self, name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<()>;
}

impl SetNvARValue for CUDAStream {
	fn set(self, name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<()> {
		unsafe { sys::NvAR_SetCudaStream(crate::nvar_lib_handle(), feature, name, self.0) }
	}
}
impl SetNvARValue for u32 {
	fn set(self, name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<()> {
		unsafe { sys::NvAR_SetU32(crate::nvar_lib_handle(), feature, name, self) }
	}
}
impl SetNvARValue for i32 {
	fn set(self, name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<()> {
		unsafe { sys::NvAR_SetS32(crate::nvar_lib_handle(), feature, name, self) }
	}
}
impl SetNvARValue for f32 {
	fn set(self, name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<()> {
		unsafe { sys::NvAR_SetF32(crate::nvar_lib_handle(), feature, name, self) }
	}
}
impl SetNvARValue for f64 {
	fn set(self, name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<()> {
		unsafe { sys::NvAR_SetF64(crate::nvar_lib_handle(), feature, name, self) }
	}
}
impl SetNvARValue for u64 {
	fn set(self, name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<()> {
		unsafe { sys::NvAR_SetU64(crate::nvar_lib_handle(), feature, name, self) }
	}
}
impl SetNvARValue for &mut Vec<f32> {
	fn set(self, name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<()> {
		unsafe { sys::NvAR_SetF32Array(crate::nvar_lib_handle(), feature, name, self) }
	}
}
impl SetNvARValue for &mut [f32] {
	fn set(self, name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<()> {
		unsafe { sys::NvAR_SetF32Array(crate::nvar_lib_handle(), feature, name, self) }
	}
}
impl SetNvARValue for &str {
	fn set(self, name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<()> {
		let s = CString::new(self).unwrap();
		unsafe { sys::NvAR_SetString(crate::nvar_lib_handle(), feature, name, s.as_ptr()) }
	}
}
impl SetNvARValue for String {
	fn set(self, name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<()> {
		let s = CString::new(self).unwrap();
		unsafe { sys::NvAR_SetString(crate::nvar_lib_handle(), feature, name, s.as_ptr()) }
	}
}
impl SetNvARValue for &CUDAStream {
	fn set(self, name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<()> {
		unsafe { sys::NvAR_SetCudaStream(crate::nvar_lib_handle(), feature, name, self.0) }
	}
}
impl SetNvARValue for Pin<&mut sys::NvAR_BBoxes> {
	fn set(self, name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<()> {
		unsafe { sys::NvAR_SetObject(crate::nvar_lib_handle(), feature, name, self.get_mut() as *mut _) }
	}
}
impl SetNvARValue for Pin<&mut Quaternion> {
	fn set(self, name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<()> {
		unsafe { sys::NvAR_SetObject(crate::nvar_lib_handle(), feature, name, self.get_mut() as *mut _) }
	}
}
impl SetNvARValue for Pin<&mut Vector3> {
	fn set(self, name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<()> {
		unsafe { sys::NvAR_SetObject(crate::nvar_lib_handle(), feature, name, self.get_mut() as *mut _) }
	}
}
impl SetNvARValue for &mut Image {
	fn set(self, name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<()> {
		unsafe { sys::NvAR_SetObject(crate::nvar_lib_handle(), feature, name, self.as_ptr()) }
	}
}
impl SetNvARValue for &mut [Point2D] {
	fn set(self, name: *const i8, feature: sys::NvAR_FeatureHandle) -> Result<()> {
		unsafe { sys::NvAR_SetObject(crate::nvar_lib_handle(), feature, name, self.as_mut_ptr()) }
	}
}
