use std::fmt::Display;

use java_spaghetti::Local;
use rnfc_traits::iso14443a;

use crate::bindings::java::lang::Throwable;

macro_rules! impl_from_throwable {
    ($t:ident) => {
        impl<'env> From<Local<'env, Throwable>> for $t {
            fn from(e: Local<'env, Throwable>) -> Self {
                Self::Exception(format!("{e:?}"))
            }
        }
    };
}

#[derive(Clone, Debug)]
pub enum NewReaderError {
    NfcNotSupported,
    Exception(String),
}
impl Display for NewReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for NewReaderError {}
impl_from_throwable!(NewReaderError);

#[derive(Clone, Debug)]
pub enum AsTechError {
    TechNotSupported,
    Exception(String),
}
impl Display for AsTechError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for AsTechError {}
impl_from_throwable!(AsTechError);

#[derive(Clone, Debug)]
pub enum TransceiveError {
    BufferTooSmall,
    Exception(String),
}
impl Display for TransceiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for TransceiveError {}
impl iso14443a::Error for TransceiveError {
    fn kind(&self) -> rnfc_traits::iso14443a_ll::ErrorKind {
        rnfc_traits::iso14443a_ll::ErrorKind::Other
    }
}
impl_from_throwable!(TransceiveError);
