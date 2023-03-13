use core::marker::PhantomData;
use std::{path::PathBuf, fs::create_dir_all};

mod kind;
pub mod existence;
use kind::Kind;
use existence::Existence;

use crate::error_ty::ErrorTy;

pub(crate) struct TargetPath<'a, const KIND_ID: u8, const EXISTENCE_ID: u8> {
    pub path: PathBuf,
    phantom: PhantomData<&'a ()>,
}

pub(crate) type TargetImageDir<'a> = TargetPath<'a, { Kind::ImageDir as u8 }, { Existence::Unknown as u8 }>;
// pub(crate) type TargetFile<'a> = TargetPath<'a, { Kind::File as u8 }>;
#[allow(non_snake_case)]
pub(crate) fn TargetImageDir<'a>(dir: PathBuf) -> TargetImageDir<'a> {
    TargetPath {
        path: dir,
        phantom: PhantomData,
    }
}

impl<'a> TargetImageDir<'a> {
    pub(crate) fn ensure_exists(&self) -> Result<(), ErrorTy> {
        let Self { path, .. } = &*self;
        if !path.try_exists().map_err(|e| ErrorTy::TryExistsFailed(e, path.to_string_lossy().into_owned()))?
        {
            create_dir_all(path).map_err(|e| ErrorTy::CreateDirAllFailed(e, path.to_string_lossy().into_owned()))?;
        };
        Ok(())
    }
}
