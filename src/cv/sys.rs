#![allow(
    non_snake_case,
    non_camel_case_types,
    clippy::too_many_arguments,
    clippy::missing_safety_doc
)]

use std::{ffi::c_void, ptr};

use bitflags::bitflags;
use libloading::Library;

use crate::ar::sys::CUstream;

#[repr(u32)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NvCVImage_PixelFormat {
    #[default]
    Unknown = 0,
    Y = 1,
    A = 2,
    YA = 3,
    RGB = 4,
    BGR = 5,
    RGBA = 6,
    BGRA = 7,
    ARGB = 8,
    ABGR = 9,
    YUV420 = 10,
    YUV422 = 11,
    YUV444 = 12,
}

#[repr(u32)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NvCVImage_ComponentType {
    #[default]
    Unknown = 0,
    U8 = 1,
    U16 = 2,
    S16 = 3,
    F16 = 4,
    U32 = 5,
    S32 = 6,
    F32 = 7,
    U64 = 8,
    S64 = 9,
    F64 = 10,
}

#[repr(u8)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NvCVImage_Layout {
    #[default]
    Interleaved = 0,
    Planar = 1,
    UYVY = 2,
    VYUY = 4,
    YUYV = 6,
    YVYU = 8,
    CYUV = 10,
    CYVU = 12,
    YUV = 3,
    YVU = 5,
    YCUV = 7,
    YCVU = 9,
}

impl NvCVImage_Layout {
    #[allow(non_upper_case_globals)]
    pub const Chunky: NvCVImage_Layout = NvCVImage_Layout::Interleaved;

    pub const I420: NvCVImage_Layout = NvCVImage_Layout::YUV;
    pub const IYUV: NvCVImage_Layout = NvCVImage_Layout::YUV;
    pub const YV12: NvCVImage_Layout = NvCVImage_Layout::YVU;
    pub const NV12: NvCVImage_Layout = NvCVImage_Layout::YCUV;
    pub const NV21: NvCVImage_Layout = NvCVImage_Layout::YCVU;
    pub const YUY2: NvCVImage_Layout = NvCVImage_Layout::YUYV;
    pub const I444: NvCVImage_Layout = NvCVImage_Layout::YUV;
    pub const YM24: NvCVImage_Layout = NvCVImage_Layout::YUV;
    pub const YM42: NvCVImage_Layout = NvCVImage_Layout::YVU;
    pub const NV24: NvCVImage_Layout = NvCVImage_Layout::YCUV;
    pub const NV42: NvCVImage_Layout = NvCVImage_Layout::YCVU;
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NvCVImage_ColorSpace: u8 {
        const R601 = 0x00;
        const R709 = 0x01;
        const R2020 = 0x02;

        const RANGE_VIDEO = 0x00;
        const RANGE_FULL = 0x04;

        const CHROMA_COSITED = 0x00;
        const CHROMA_INTERSTITIAL = 0x08;
        const CHROMA_TOPLEFT = 0x10;
        const CHROMA_MPEG2 = Self::CHROMA_COSITED.bits();
        const CHROMA_MPEG1 = Self::CHROMA_INTERSTITIAL.bits();
        const CHROMA_JPEG = Self::CHROMA_INTERSTITIAL.bits();
    }
}

#[repr(u8)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NvCVImage_MemorySpace {
    #[default]
    CPU = 0,
    GPU = 1,
    CPUPinned = 2,
    CUDAArray = 3,
}

#[repr(C)]
pub struct NvCVImage {
    pub width: u32,
    pub height: u32,
    pub pitch: i32,
    pub pixel_format: NvCVImage_PixelFormat,
    pub component_type: NvCVImage_ComponentType,
    pub pixel_bytes: u8,
    pub component_bytes: u8,
    pub num_components: u8,
    pub planar: NvCVImage_Layout,
    pub gpu_mem: NvCVImage_MemorySpace,
    pub color_space: NvCVImage_ColorSpace,
    reserved: [u8; 2],
    pub pixels: *mut c_void,
    delete_ptr: *mut c_void,
    delete_proc: Option<fn(p: *mut c_void)>,
    buffer_bytes: u64,
}

impl Default for NvCVImage {
    fn default() -> Self {
        NvCVImage {
            width: 0,
            height: 0,
            pitch: 0,
            pixel_format: NvCVImage_PixelFormat::Unknown,
            component_type: NvCVImage_ComponentType::Unknown,
            pixel_bytes: 0,
            component_bytes: 0,
            num_components: 0,
            planar: NvCVImage_Layout::Interleaved,
            gpu_mem: NvCVImage_MemorySpace::CPU,
            color_space: NvCVImage_ColorSpace::empty(),
            reserved: [0, 0],
            pixels: ptr::null_mut(),
            delete_ptr: ptr::null_mut(),
            delete_proc: None,
            buffer_bytes: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct NvCVRect2i {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NvCVPoint2i {
    pub x: i32,
    pub y: i32,
}

pub unsafe fn NvCVImage_Init(
    library: &Library,
    im: *mut NvCVImage,
    width: u32,
    height: u32,
    pitch: i32,
    pixels: *mut c_void,
    format: NvCVImage_PixelFormat,
    r#type: NvCVImage_ComponentType,
    layout: NvCVImage_Layout,
    mem_space: NvCVImage_MemorySpace,
) -> crate::Result<()> {
    let sym = library.get::<unsafe extern "C" fn(
        *mut NvCVImage,
        u32,
        u32,
        i32,
        *mut c_void,
        NvCVImage_PixelFormat,
        NvCVImage_ComponentType,
        NvCVImage_Layout,
        NvCVImage_MemorySpace,
    ) -> i32>(b"NvCVImage_Init")?;
    crate::error::to_status(sym(
        im, width, height, pitch, pixels, format, r#type, layout, mem_space,
    ))
}

pub unsafe fn NvCVImage_InitView(
    library: &Library,
    subImg: *mut NvCVImage,
    fullImg: *mut NvCVImage,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> crate::Result<()> {
    let sym = library
        .get::<unsafe extern "C" fn(*mut NvCVImage, *mut NvCVImage, i32, i32, u32, u32)>(
            b"NvCVImage_InitView",
        )?;
    sym(subImg, fullImg, x, y, width, height);
    Ok(())
}

pub unsafe fn NvCVImage_Alloc(
    library: &Library,
    im: *mut NvCVImage,
    width: u32,
    height: u32,
    format: NvCVImage_PixelFormat,
    r#type: NvCVImage_ComponentType,
    layout: NvCVImage_Layout,
    mem_space: NvCVImage_MemorySpace,
    alignment: u32,
) -> crate::Result<()> {
    let sym = library.get::<unsafe extern "C" fn(
        *mut NvCVImage,
        u32,
        u32,
        NvCVImage_PixelFormat,
        NvCVImage_ComponentType,
        u32,
        u32,
        u32,
    ) -> i32>(b"NvCVImage_Alloc")?;
    crate::error::to_status(sym(
        im,
        width,
        height,
        format,
        r#type,
        layout as u8 as u32,
        mem_space as u8 as u32,
        alignment,
    ))
}

pub unsafe fn NvCVImage_Realloc(
    library: &Library,
    im: *mut NvCVImage,
    width: u32,
    height: u32,
    format: NvCVImage_PixelFormat,
    r#type: NvCVImage_ComponentType,
    layout: NvCVImage_Layout,
    mem_space: NvCVImage_MemorySpace,
    alignment: u32,
) -> crate::Result<()> {
    let sym = library.get::<unsafe extern "C" fn(
        *mut NvCVImage,
        u32,
        u32,
        NvCVImage_PixelFormat,
        NvCVImage_ComponentType,
        NvCVImage_Layout,
        NvCVImage_MemorySpace,
        u32,
    ) -> i32>(b"NvCVImage_Realloc")?;
    crate::error::to_status(sym(
        im, width, height, format, r#type, layout, mem_space, alignment,
    ))
}

pub unsafe fn NvCVImage_Dealloc(library: &Library, im: *mut NvCVImage) -> crate::Result<()> {
    let sym = library.get::<unsafe extern "C" fn(*mut NvCVImage) -> ()>(b"NvCVImage_Dealloc")?;
    sym(im);
    Ok(())
}

pub unsafe fn NvCVImage_DeallocAsync(
    library: &Library,
    im: *mut NvCVImage,
    stream: CUstream,
) -> crate::Result<()> {
    let sym = library
        .get::<unsafe extern "C" fn(*mut NvCVImage, CUstream) -> ()>(b"NvCVImage_DeallocAsync")?;
    sym(im, stream);
    Ok(())
}

pub unsafe fn NvCVImage_Create(
    library: &Library,
    width: u32,
    height: u32,
    format: NvCVImage_PixelFormat,
    r#type: NvCVImage_ComponentType,
    layout: NvCVImage_Layout,
    mem_space: NvCVImage_MemorySpace,
    alignment: u32,
) -> crate::Result<*mut NvCVImage> {
    let sym = library.get::<unsafe extern "C" fn(
        u32,
        u32,
        NvCVImage_PixelFormat,
        NvCVImage_ComponentType,
        NvCVImage_Layout,
        NvCVImage_MemorySpace,
        u32,
        *mut *mut NvCVImage,
    ) -> i32>(b"NvCVImage_Create")?;
    let mut output: *mut NvCVImage = ptr::null_mut();
    crate::error::to_status(sym(
        width,
        height,
        format,
        r#type,
        layout,
        mem_space,
        alignment,
        &mut output as *mut _,
    ))?;
    assert!(!output.is_null());
    Ok(output)
}

pub unsafe fn NvCVImage_Destroy(library: &Library, im: *mut NvCVImage) -> crate::Result<()> {
    let sym = library.get::<unsafe extern "C" fn(*mut NvCVImage)>(b"NvCVImage_Destroy")?;
    sym(im);
    Ok(())
}

#[derive(Debug, Default, Clone)]
pub struct NvCVImage_ComponentOffsets {
    pub r: i32,
    pub g: i32,
    pub b: i32,
    pub a: i32,
    pub y: i32,
}

pub unsafe fn NvCVImage_ComponentOffsets(
    library: &Library,
    format: NvCVImage_PixelFormat,
) -> crate::Result<NvCVImage_ComponentOffsets> {
    let sym = library.get::<unsafe extern "C" fn(
        NvCVImage_PixelFormat,
        *mut i32,
        *mut i32,
        *mut i32,
        *mut i32,
        *mut i32,
    )>(b"NvCVImage_ComponentOffsets")?;
    let mut out = NvCVImage_ComponentOffsets::default();
    sym(
        format, &mut out.r, &mut out.g, &mut out.b, &mut out.a, &mut out.y,
    );
    Ok(out)
}

pub unsafe fn NvCVImage_Transfer(
    library: &Library,
    src: *const NvCVImage,
    dst: *mut NvCVImage,
    scale: f32,
    stream: CUstream,
    tmp: *mut NvCVImage,
) -> crate::Result<()> {
    let sym = library.get::<unsafe extern "C" fn(
        *const NvCVImage,
        *mut NvCVImage,
        f32,
        CUstream,
        *mut NvCVImage,
    ) -> i32>(b"NvCVImage_Transfer")?;
    crate::error::to_status(sym(src, dst, scale, stream, tmp))
}

pub unsafe fn NvCVImage_TransferRect(
    library: &Library,
    src: *const NvCVImage,
    src_rect: *const NvCVRect2i,
    dst: *mut NvCVImage,
    dst_pt: *const NvCVPoint2i,
    scale: f32,
    stream: CUstream,
    tmp: *mut NvCVImage,
) -> crate::Result<()> {
    let sym = library.get::<unsafe extern "C" fn(
        *const NvCVImage,
        *const NvCVRect2i,
        *mut NvCVImage,
        *const NvCVPoint2i,
        f32,
        CUstream,
        *mut NvCVImage,
    ) -> i32>(b"NvCVImage_TransferRect")?;
    crate::error::to_status(sym(src, src_rect, dst, dst_pt, scale, stream, tmp))
}

pub unsafe fn NvCVImage_TransferFromYUV(
    library: &Library,
    y: *const c_void,
    y_pix_bytes: i32,
    y_pitch: i32,
    u: *const c_void,
    v: *const c_void,
    uv_pix_bytes: i32,
    uv_pitch: i32,
    yuv_format: NvCVImage_PixelFormat,
    yuv_type: NvCVImage_ComponentType,
    yuv_color_space: NvCVImage_ColorSpace,
    yuv_mem_space: NvCVImage_MemorySpace,
    dst: *mut NvCVImage,
    dst_rect: *const NvCVRect2i,
    scale: f32,
    stream: CUstream,
    tmp: *mut NvCVImage,
) -> crate::Result<()> {
    let sym = library.get::<unsafe extern "C" fn(
        *const c_void,
        i32,
        i32,
        *const c_void,
        *const c_void,
        i32,
        i32,
        NvCVImage_PixelFormat,
        NvCVImage_ComponentType,
        NvCVImage_ColorSpace,
        NvCVImage_MemorySpace,
        *mut NvCVImage,
        *const NvCVRect2i,
        f32,
        CUstream,
        *mut NvCVImage,
    ) -> i32>(b"NvCVImage_TransferFromYUV")?;
    crate::error::to_status(sym(
        y,
        y_pix_bytes,
        y_pitch,
        u,
        v,
        uv_pix_bytes,
        uv_pitch,
        yuv_format,
        yuv_type,
        yuv_color_space,
        yuv_mem_space,
        dst,
        dst_rect,
        scale,
        stream,
        tmp,
    ))
}

pub unsafe fn NvCVImage_TransferToYUV(
    library: &Library,
    src: *const NvCVImage,
    src_rect: *const NvCVRect2i,
    y: *const c_void,
    y_pix_bytes: i32,
    y_pitch: i32,
    u: *const c_void,
    v: *const c_void,
    uv_pix_bytes: i32,
    uv_pitch: i32,
    yuv_format: NvCVImage_PixelFormat,
    yuv_type: NvCVImage_ComponentType,
    yuv_color_space: NvCVImage_ColorSpace,
    yuv_mem_space: NvCVImage_MemorySpace,
    scale: f32,
    stream: CUstream,
    tmp: *mut NvCVImage,
) -> crate::Result<()> {
    let sym = library.get::<unsafe extern "C" fn(
        *const NvCVImage,
        *const NvCVRect2i,
        *const c_void,
        i32,
        i32,
        *const c_void,
        *const c_void,
        i32,
        i32,
        NvCVImage_PixelFormat,
        NvCVImage_ComponentType,
        NvCVImage_ColorSpace,
        NvCVImage_MemorySpace,
        f32,
        CUstream,
        *mut NvCVImage,
    ) -> i32>(b"NvCVImage_TransferToYUV")?;
    crate::error::to_status(sym(
        src,
        src_rect,
        y,
        y_pix_bytes,
        y_pitch,
        u,
        v,
        uv_pix_bytes,
        uv_pitch,
        yuv_format,
        yuv_type,
        yuv_color_space,
        yuv_mem_space,
        scale,
        stream,
        tmp,
    ))
}

pub unsafe fn NvCVImage_MapResource(
    library: &Library,
    im: *mut NvCVImage,
    stream: CUstream,
) -> crate::Result<()> {
    let sym = library
        .get::<unsafe extern "C" fn(*mut NvCVImage, CUstream) -> i32>(b"NvCVImage_MapResource")?;
    crate::error::to_status(sym(im, stream))
}

pub unsafe fn NvCVImage_UnmapResource(
    library: &Library,
    im: *mut NvCVImage,
    stream: CUstream,
) -> crate::Result<()> {
    let sym = library
        .get::<unsafe extern "C" fn(*mut NvCVImage, CUstream) -> i32>(b"NvCVImage_UnmapResource")?;
    crate::error::to_status(sym(im, stream))
}

pub unsafe fn NvCVImage_Composite(
    library: &Library,
    fg: *const NvCVImage,
    bg: *const NvCVImage,
    mat: *const NvCVImage,
    dst: *mut NvCVImage,
    stream: CUstream,
) -> crate::Result<()> {
    let sym = library.get::<unsafe extern "C" fn(
        *const NvCVImage,
        *const NvCVImage,
        *const NvCVImage,
        *mut NvCVImage,
        CUstream,
    ) -> i32>(b"NvCVImage_Composite")?;
    crate::error::to_status(sym(fg, bg, mat, dst, stream))
}

pub unsafe fn NvCVImage_CompositeRect(
    library: &Library,
    fg: *const NvCVImage,
    fg_org: *const NvCVPoint2i,
    bg: *const NvCVImage,
    bg_org: *const NvCVPoint2i,
    mat: *const NvCVImage,
    premultiplied: bool,
    dst: *mut NvCVImage,
    dst_org: *const NvCVPoint2i,
    stream: CUstream,
) -> crate::Result<()> {
    let sym = library.get::<unsafe extern "C" fn(
        *const NvCVImage,
        *const NvCVPoint2i,
        *const NvCVImage,
        *const NvCVPoint2i,
        *const NvCVImage,
        u32,
        *mut NvCVImage,
        *const NvCVPoint2i,
        CUstream,
    ) -> i32>(b"NvCVImage_CompositeRect")?;
    crate::error::to_status(sym(
        fg,
        fg_org,
        bg,
        bg_org,
        mat,
        premultiplied.into(),
        dst,
        dst_org,
        stream,
    ))
}

pub unsafe fn NvCVImage_CompositeOverConstant(
    library: &Library,
    src: *const NvCVImage,
    mat: *const NvCVImage,
    bg_color: *const c_void,
    dst: *mut NvCVImage,
    stream: CUstream,
) -> crate::Result<()> {
    let sym = library.get::<unsafe extern "C" fn(
        *const NvCVImage,
        *const NvCVImage,
        *const c_void,
        *mut NvCVImage,
        CUstream,
    ) -> i32>(b"NvCVImage_CompositeOverConstant")?;
    crate::error::to_status(sym(src, mat, bg_color, dst, stream))
}

pub unsafe fn NvCVImage_FlipY(
    library: &Library,
    src: *const NvCVImage,
    dst: *mut NvCVImage,
) -> crate::Result<()> {
    let sym = library
        .get::<unsafe extern "C" fn(*const NvCVImage, *mut NvCVImage) -> i32>(b"NvCVImage_FlipY")?;
    crate::error::to_status(sym(src, dst))
}

#[derive(Debug)]
pub struct NvCVImage_GetYUVPointers {
    pub y: *mut u8,
    pub u: *mut u8,
    pub v: *mut u8,
    pub y_pix_bytes: i32,
    pub c_pix_bytes: i32,
    pub y_row_bytes: i32,
    pub c_row_bytes: i32,
}
pub unsafe fn NvCVImage_GetYUVPointers(
    library: &Library,
    im: *mut NvCVImage,
) -> crate::Result<NvCVImage_GetYUVPointers> {
    let sym = library.get::<unsafe extern "C" fn(
        *mut NvCVImage,
        *mut *mut u8,
        *mut *mut u8,
        *mut *mut u8,
        *mut i32,
        *mut i32,
        *mut i32,
        *mut i32,
    ) -> i32>(b"NvCVImage_GetYUVPointers")?;
    let mut out = NvCVImage_GetYUVPointers {
        y: ptr::null_mut(),
        u: ptr::null_mut(),
        v: ptr::null_mut(),
        y_pix_bytes: 0,
        c_pix_bytes: 0,
        y_row_bytes: 0,
        c_row_bytes: 0,
    };
    crate::error::to_status(sym(
        im,
        &mut out.y,
        &mut out.u,
        &mut out.v,
        &mut out.y_pix_bytes,
        &mut out.c_pix_bytes,
        &mut out.y_row_bytes,
        &mut out.c_row_bytes,
    ))?;
    Ok(out)
}

pub unsafe fn NvCVImage_Sharpen(
    library: &Library,
    sharpness: f32,
    src: *const NvCVImage,
    dst: *mut NvCVImage,
    stream: CUstream,
    tmp: *mut NvCVImage,
) -> crate::Result<()> {
    let sym = library.get::<unsafe extern "C" fn(
        f32,
        *const NvCVImage,
        *mut NvCVImage,
        CUstream,
        *mut NvCVImage,
    ) -> i32>(b"NvCVImage_Sharpen")?;
    crate::error::to_status(sym(sharpness, src, dst, stream, tmp))
}
