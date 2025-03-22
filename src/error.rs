use std::fmt;

macro_rules! define_error {
    (
		$(#[$attr:meta])*
		$vis:vis enum $name:ident {
			$(
				#[error($m:literal)]
				$variant:ident = $id:expr
			),*
		}
	) => {
		$(#[$attr])*
		$vis enum $name {
			$(
				$variant
			),*
		}

		impl fmt::Display for $name {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				match self {
					$(
						Self::$variant => f.write_str($m)
					),*
				}
			}
		}

		impl std::error::Error for $name {}

		impl From<i32> for $name {
			fn from(value: i32) -> $name {
				match value {
					$($id => $name::$variant,)*
					_ => unimplemented!(concat!("Unknown value `{}` for ", stringify!($name)), value)
				}
			}
		}
	};
}

define_error! {
	#[derive(Debug, Clone)]
	pub enum NvError {
		#[error("An otherwise unspecified error has occurred.")]
		General = -1,
		#[error("The requested feature is not yet implemented.")]
		Unimplemented = -2,
		#[error("There is not enough memory for the requested operation.")]
		OutOfMemory = -3
	}
}

#[derive(Debug)]
pub enum Error {
	Nv(NvError),
	Dylib(libloading::Error)
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Nv(e) => e.fmt(f),
			Self::Dylib(e) => {
				f.write_str("Error loading symbol from dynamic library: ")?;
				e.fmt(f)
			}
		}
	}
}

impl std::error::Error for Error {}

impl From<NvError> for Error {
	fn from(e: NvError) -> Self {
		Self::Nv(e)
	}
}

impl From<libloading::Error> for Error {
	fn from(e: libloading::Error) -> Self {
		Self::Dylib(e)
	}
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[inline]
pub fn to_status(code: i32) -> Result<(), Error> {
	if code == 0 { Ok(()) } else { Err(Error::Nv(NvError::from(code))) }
}
