//! Standard error type for ocl.
//!

use thiserror::Error;
use crate::util::UtilError;
use crate::functions::{ApiError, VersionLowError, ProgramBuildError, ApiWrapperError};
use crate::{Status, EmptyInfoResultError};


/// Ocl error result type.
pub type Result<T> = ::std::result::Result<T, Error>;


/// An enum one of several error types.
#[derive(Debug, Error)]
pub enum Error {
    // String: An arbitrary error:
    //
    // TODO: Remove this eventually. We need to replace every usage
    // (conversion from String/str) with a dedicated error type/variant for
    // each. In the meanwhile, refrain from creating new instances of this by
    // converting strings to `Error`!
    #[error("{}", _0)]
    String(String),
    // FfiNul: Ffi string conversion error:
    #[error("{}", _0)]
    FfiNul(#[from] ::std::ffi::NulError),
    // Io: std::io error:
    #[error("{}", _0)]
    Io(#[from] ::std::io::Error),
    // FromUtf8: String conversion error:
    #[error("{}", _0)]
    FromUtf8(#[from] ::std::string::FromUtf8Error),
    // IntoString: Ffi string conversion error:
    #[error("{}", _0)]
    IntoString(#[from] ::std::ffi::IntoStringError),
    // EmptyInfoResult:
    #[error("{}", _0)]
    EmptyInfoResult(#[from] EmptyInfoResultError),
    // Util:
    #[error("{}", _0)]
    Util(#[from] UtilError),
    // Api:
    #[error("{}", _0)]
    Api(#[from] ApiError),
    // VersionLow:
    #[error("{}", _0)]
    VersionLow(#[from] VersionLowError),
    // ProgramBuild:
    #[error("{}", _0)]
    ProgramBuild(#[from] ProgramBuildError),
    // ApiWrapper:
    #[error("{}", _0)]
    ApiWrapper(#[from] ApiWrapperError),
}


impl Error {
   /// Returns the error status code for `Status` variants.
   pub fn api_status(&self) -> Option<Status> {
       match *self {
           Error::Api(ref err) => Some(err.status()),
           _ => None,
       }
   }
}

// TODO: Remove eventually
impl<'a> From<&'a str> for Error {
    fn from(desc: &'a str) -> Self {
        Error::String(String::from(desc))
    }
}

// TODO: Remove eventually
impl From<String> for Error {
    fn from(desc: String) -> Self {
        Error::String(desc)
    }
}
