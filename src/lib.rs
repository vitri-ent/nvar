pub mod ar;
pub mod cv;
mod error;

use std::{
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use libloading::Library;

pub use self::error::{Error, Result};

pub(crate) static NVCV_DYLIB_PATH: OnceLock<Arc<PathBuf>> = OnceLock::new();
pub(crate) static NVAR_DYLIB_PATH: OnceLock<Arc<PathBuf>> = OnceLock::new();
pub(crate) static NVCV_LIBRARY: OnceLock<Arc<Library>> = OnceLock::new();
pub(crate) static NVAR_LIBRARY: OnceLock<Arc<Library>> = OnceLock::new();

pub(crate) fn nvcv_path() -> &'static PathBuf {
    NVCV_DYLIB_PATH.get_or_init(|| {
        let path = match std::env::var("NVAR_ROOT") {
            Ok(s) if !s.is_empty() => PathBuf::from(s).join("NVCVImage.dll"),
            #[cfg(target_os = "windows")]
            _ => "NVCVImage.dll".into(),
        };
        Arc::new(path)
    })
}

pub(crate) fn nvar_path() -> &'static PathBuf {
    NVAR_DYLIB_PATH.get_or_init(|| {
        let path = match std::env::var("NVAR_ROOT") {
            Ok(s) if !s.is_empty() => PathBuf::from(s).join("nvARPose.dll"),
            #[cfg(target_os = "windows")]
            _ => "nvARPose.dll".into(),
        };
        Arc::new(path)
    })
}

#[inline]
pub fn nvcv_lib_handle() -> &'static libloading::Library {
    NVCV_LIBRARY.get_or_init(|| {
        let path = nvcv_path();
        let lib = unsafe { Library::new(path) }.unwrap_or_else(|e| {
            panic!(
                "An error occurred while attempting to load the nvCV binary at `{}`: {e}",
                path.display()
            )
        });
        Arc::new(lib)
    })
}

#[inline]
pub fn nvar_lib_handle() -> &'static libloading::Library {
    NVAR_LIBRARY.get_or_init(|| {
        let path = nvar_path();
        let lib = unsafe { Library::new(path) }.unwrap_or_else(|e| {
            panic!(
                "An error occurred while attempting to load the nvAR binary at `{}`: {e}",
                path.display()
            )
        });
        Arc::new(lib)
    })
}
