//! Standard error type for ocl futures.
//!

use std;
// use std::sync::mpsc::{SendError as StdMpscSendError, RecvError as StdMpscRecvError};
use thiserror::Error;
use futures::sync::oneshot::Canceled as OneshotCanceled;
use crate::core::error::{Error as OclCoreError};
use crate::core::Status;
use crate::standard::{DeviceError, PlatformError, KernelError};

use crate::BufferCmdError;

pub type Result<T> = std::result::Result<T, Error>;


/// An enum containing either a `String` or one of several other error types.
///
/// Implements the usual error traits.
#[derive(Debug, Error)]
pub enum Error {
    #[error("{}", _0)]
    OclCore(#[from] OclCoreError),
    #[error("{}", _0)]
    FuturesMpscSend(String),
    // #[error("{}", _0)]
    // StdMpscSend(String),
    // #[error("{}", _0)]
    // StdMpscRecv(StdMpscRecvError),
    #[error("{}", _0)]
    OneshotCanceled(#[from] OneshotCanceled),
    #[error("{}", _0)]
    BufferCmd(#[from] BufferCmdError),
    #[error("{}", _0)]
    Device(#[from] DeviceError),
    #[error("{}", _0)]
    Platform(#[from] PlatformError),
    #[error("{}", _0)]
    Kernel(#[from] KernelError),
}

impl Error {
    /// Returns the error status code for `OclCore` variants.
    pub fn api_status(&self) -> Option<Status> {
        match *self {
            Error::OclCore(ref err) => err.api_status(),
            _ => None,
        }
    }
}


// TODO: Remove eventually
impl From<String> for Error {
    fn from(desc: String) -> Error {
        Error::OclCore(desc.into())
    }
}

// TODO: Remove eventually
impl<'a> From<&'a str> for Error {
    fn from(desc: &'a str) -> Error {
        Error::OclCore(desc.into())
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(err: std::ffi::NulError) -> Error {
        Error::OclCore(err.into())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::OclCore(err.into())
    }
}


unsafe impl Send for Error {}
unsafe impl Sync for Error {}
