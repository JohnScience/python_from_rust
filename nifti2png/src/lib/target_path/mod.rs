use pyo3::PyAny;
mod kind;
pub mod existence;
use kind::Kind;
use existence::Existence;

use crate::error_ty::ErrorTy;

pub(crate) struct TargetPath<'a, const KIND_ID: u8, const EXISTENCE_ID: u8>(pub &'a PyAny);

pub(crate) type TargetImageDir<'a> = TargetPath<'a, { Kind::ImageDir as u8 }, { Existence::Unknown as u8 }>;
// pub(crate) type TargetFile<'a> = TargetPath<'a, { Kind::File as u8 }>;
#[allow(non_snake_case)]
pub(crate) fn TargetImageDir<'a>(dir: &'a PyAny) -> TargetImageDir<'a> {
    TargetPath(dir)
}

impl<'a> TargetImageDir<'a> {
    pub(crate) fn ensure_exists(&self, os: &'a PyAny) -> Result<(), ErrorTy> {
        let Self(path) = *self;
        if !(os
            .getattr("path")?
            .call_method("exists", (path,), None)?
            .extract::<bool>()?)
        {
            os.call_method("makedirs", (path,), None)?;
        };
        Ok(())
    }
}
