#![allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    clippy::too_many_arguments,
    clippy::missing_safety_doc
)]

use std::{
    ffi::{c_char, CStr},
    os::raw::c_void,
    ptr,
};

use bitflags::bitflags;
use libloading::Library;

#[repr(C)]
#[derive(Default, Debug, Clone)]
pub struct NvAR_Vector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Default, Debug, Clone)]
pub struct NvAR_Vector3u16 {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

#[repr(C)]
#[derive(Default, Debug, Clone)]
pub struct NvAR_Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[repr(C)]
pub struct NvAR_BBoxes {
    pub boxes: *mut NvAR_Rect,
    pub num_boxes: u8,
    pub max_boxes: u8,
}

#[repr(C)]
pub struct NvAR_TrackingBBox {
    pub bbox: NvAR_Rect,
    pub tracking_id: u16,
}

#[repr(C)]
pub struct NvAR_TrackingBBoxes {
    pub boxes: *mut NvAR_TrackingBBox,
    pub num_boxes: u8,
    pub max_boxes: u8,
}

#[repr(C)]
pub struct NvAR_FaceMesh {
    pub vertices: *mut NvAR_Vector3f,
    pub num_vertices: usize,
    pub tvi: *mut NvAR_Vector3u16,
    pub num_triangles: usize,
}

#[repr(C)]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct NvAR_Frustum {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
}

#[repr(C)]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct NvAR_Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl NvAR_Quaternion {
    pub fn euler(&self) -> (f32, f32, f32) {
        let t0 = 2. * (self.w * self.x + self.y * self.z);
        let t1 = 1. - 2. * (self.x * self.x + self.y * self.y);
        let roll_x = t0.atan2(t1);
        let t2 = (2. * (self.w * self.y - self.z * self.x)).clamp(-1.0, 1.0);
        let pitch_y = t2.asin();
        let t3 = 2. * (self.w * self.z + self.x * self.y);
        let t4 = 1. - 2. * (self.y * self.y + self.z * self.z);
        let yaw_z = t3.atan2(t4);
        (roll_x, pitch_y, yaw_z)
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct NvAR_Point2f {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct NvAR_Point3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct NvAR_Vector2f {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
pub struct NvAR_RenderingParams {
    pub frustum: NvAR_Frustum,
    pub rotation: NvAR_Quaternion,
    pub translation: NvAR_Vector3f,
}

pub type NvAR_FeatureID = *const u8;

pub const NvAR_Feature_FaceBoxDetection: &CStr = c"FaceBoxDetection";
pub const NvAR_Feature_LandmarkDetection: &CStr = c"LandmarkDetection";
pub const NvAR_Feature_Face3DReconstruction: &CStr = c"Face3DReconstruction";
pub const NvAR_Feature_BodyDetection: &CStr = c"BodyDetection";
pub const NvAR_Feature_BodyPoseEstimation: &CStr = c"BodyPoseEstimation";
pub const NvAR_Feature_GazeRedirection: &CStr = c"GazeRedirection";
pub const NvAR_Feature_FaceExpressions: &CStr = c"FaceExpressions";

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NvAR_TemporalFilter: u32 {
        const FACE_BOX = 1 << 0;
        const FACIAL_LANDMARKS = 1 << 1;
        const FACE_ROTATIONAL_POSE = 1 << 2;
        const FACIAL_EXPRESSIONS = 1 << 4;
        const FACIAL_GAZE = 1 << 5;
        const ENHANCE_EXPRESSIONS = 1 << 8;
    }
}

pub type CUstream = *mut ();

#[doc(hidden)]
pub struct NvAR_Feature;
pub type NvAR_FeatureHandle = *mut NvAR_Feature;

pub unsafe fn NvAR_GetVersion(library: &Library) -> crate::Result<(u8, u8, u8)> {
    let sym = library.get::<unsafe extern "C" fn(*mut u32) -> i32>(b"NvAR_GetVersion")?;
    let mut out = 0u32;
    crate::error::to_status(sym(&mut out))?;
    Ok((
        (out >> 24 & 0xFF) as u8,
        (out >> 16 & 0xFF) as u8,
        (out >> 8 & 0xFF) as u8,
    ))
}

pub unsafe fn NvAR_Create(
    library: &Library,
    feature: *const c_char,
) -> crate::Result<NvAR_FeatureHandle> {
    let sym = library.get::<unsafe extern "C" fn(*const c_char, *mut NvAR_FeatureHandle) -> i32>(
        b"NvAR_Create",
    )?;
    let mut feature_handle = ptr::null_mut();
    crate::error::to_status(sym(feature, &mut feature_handle))?;
    Ok(feature_handle)
}

pub unsafe fn NvAR_Load(library: &Library, handle: NvAR_FeatureHandle) -> crate::Result<()> {
    let sym = library.get::<unsafe extern "C" fn(NvAR_FeatureHandle) -> i32>(b"NvAR_Load")?;
    crate::error::to_status(sym(handle))
}

pub unsafe fn NvAR_Run(library: &Library, handle: NvAR_FeatureHandle) -> crate::Result<()> {
    let sym = library.get::<unsafe extern "C" fn(NvAR_FeatureHandle) -> i32>(b"NvAR_Run")?;
    crate::error::to_status(sym(handle))
}

pub unsafe fn NvAR_Destroy(library: &Library, handle: NvAR_FeatureHandle) -> crate::Result<()> {
    let sym = library.get::<unsafe extern "C" fn(NvAR_FeatureHandle) -> i32>(b"NvAR_Destroy")?;
    crate::error::to_status(sym(handle))
}

pub unsafe fn NvAR_CudaStreamCreate(library: &Library) -> crate::Result<CUstream> {
    let sym =
        library.get::<unsafe extern "C" fn(*mut CUstream) -> i32>(b"NvAR_CudaStreamCreate")?;
    let mut out = ptr::null_mut();
    crate::error::to_status(sym(&mut out))?;
    Ok(out)
}

pub unsafe fn NvAR_CudaStreamDestroy(library: &Library, stream: CUstream) -> crate::Result<()> {
    let sym = library.get::<unsafe extern "C" fn(CUstream) -> i32>(b"NvAR_CudaStreamDestroy")?;
    crate::error::to_status(sym(stream))
}

macro_rules! define_set {
    ($(pub unsafe fn $name:ident = $ty:ty;)+) => {
        $(pub unsafe fn $name(
            library: &Library,
            handle: NvAR_FeatureHandle,
            name: *const c_char,
            val: $ty,
        ) -> crate::Result<()> {
            let sym = library
                .get::<unsafe extern "C" fn(NvAR_FeatureHandle, *const c_char, $ty) -> i32>(
                    stringify!($name).as_bytes(),
                )?;
            crate::error::to_status(sym(handle, name, val))
        })+
    };
}

define_set!(
    pub unsafe fn NvAR_SetU32 = u32;
    pub unsafe fn NvAR_SetS32 = i32;
    pub unsafe fn NvAR_SetF32 = f32;
    pub unsafe fn NvAR_SetF64 = f64;
    pub unsafe fn NvAR_SetU64 = u64;
    pub unsafe fn NvAR_SetString = *const c_char;
    pub unsafe fn NvAR_SetCudaStream = CUstream;
);

pub unsafe fn NvAR_SetObject<T>(
    library: &Library,
    handle: NvAR_FeatureHandle,
    name: *const c_char,
    ptr: *mut T,
) -> crate::Result<()> {
    let sym = library.get::<unsafe extern "C" fn(
        NvAR_FeatureHandle,
        *const c_char,
        *mut c_void,
        u32,
    ) -> i32>(b"NvAR_SetObject")?;
    crate::error::to_status(sym(
        handle,
        name,
        ptr.cast(),
        std::mem::size_of::<T>() as u32,
    ))
}

pub unsafe fn NvAR_SetF32Array(
    library: &Library,
    handle: NvAR_FeatureHandle,
    name: *const c_char,
    vals: &mut [f32],
) -> crate::Result<()> {
    let sym =
        library
            .get::<unsafe extern "C" fn(NvAR_FeatureHandle, *const c_char, *mut f32, i32) -> i32>(
                b"NvAR_SetF32Array",
            )?;
    crate::error::to_status(sym(handle, name, vals.as_mut_ptr(), vals.len() as i32))
}

macro_rules! define_get {
    ($(pub unsafe fn $name:ident: $ty:ty = $def:expr;)+) => {
        $(pub unsafe fn $name(
            library: &Library,
            handle: NvAR_FeatureHandle,
            name: *const c_char,
        ) -> crate::Result<$ty> {
            let sym = library
                .get::<unsafe extern "C" fn(NvAR_FeatureHandle, *const c_char, *mut $ty) -> i32>(
                    stringify!($name).as_bytes(),
                )?;
            let mut val = $def;
            crate::error::to_status(sym(handle, name, &mut val))?;
            Ok(val)
        })+
    };
}

define_get!(
    pub unsafe fn NvAR_GetU32: u32 = 0;
    pub unsafe fn NvAR_GetS32: i32 = 0;
    pub unsafe fn NvAR_GetF32: f32 = 0.0;
    pub unsafe fn NvAR_GetF64: f64 = 0.0;
    pub unsafe fn NvAR_GetU64: u64 = 0;
    pub unsafe fn NvAR_GetString: *const c_char = ptr::null();
    pub unsafe fn NvAR_GetCudaStream: CUstream = ptr::null_mut();
);

pub unsafe fn NvAR_GetObject<T>(
    library: &Library,
    handle: NvAR_FeatureHandle,
    name: *const c_char,
) -> crate::Result<*const T> {
    let sym = library.get::<unsafe extern "C" fn(
        NvAR_FeatureHandle,
        *const c_char,
        *mut *const c_void,
        u32,
    ) -> i32>(b"NvAR_GetObject")?;
    let mut val = ptr::null();
    crate::error::to_status(sym(handle, name, &mut val, std::mem::size_of::<T>() as u32))?;
    Ok(val.cast())
}

pub unsafe fn NvAR_GetF32Array<'v>(
    library: &Library,
    handle: NvAR_FeatureHandle,
    name: *const c_char,
) -> crate::Result<&'v [f32]> {
    let sym = library.get::<unsafe extern "C" fn(
        NvAR_FeatureHandle,
        *const c_char,
        *mut *const f32,
        *mut i32,
    ) -> i32>(b"NvAR_GetF32Array")?;
    let mut val = ptr::null();
    let mut len = 0;
    crate::error::to_status(sym(handle, name, &mut val, &mut len))?;
    Ok(std::slice::from_raw_parts(val, len as usize))
}
