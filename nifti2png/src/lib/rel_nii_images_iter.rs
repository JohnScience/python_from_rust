use std::path::PathBuf;

use crate::{
    error_ty::ErrorTy, nii_image::NiiImage, rel_nii_files_iter::RelNiiFilesIter,
    target_path::TargetImageDir,
};
use arrayvec::ArrayVec;
use pyo3::prelude::*;

// Iterator over pairs of (png_stub, nii_image) for all nii files in nii_files
// where png_stub is a path to a directory where the png files for the NIFTI volume will be saved.
pub(crate) struct RelNiiImagesIter<'a>(RelNiiFilesIter<'a>);

impl<'a> RelNiiImagesIter<'a> {
    pub(crate) fn new(
        nib: &'a PyModule,
        os: &'a PyModule,
        nii_files: &'a str,
        base_png_stub: PathBuf,
    ) -> Result<Self, ErrorTy> {
        Ok(Self(RelNiiFilesIter::new(
            nib,
            os,
            nii_files,
            base_png_stub,
        )?))
    }

    fn nii_obj2nii_image(nii_obj: &'a PyAny) -> Result<NiiImage<'a>, ErrorTy> {
        // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L78
        let hdr = nii_obj.getattr("header")?;

        // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L81
        let nii_shape = hdr.call_method("get_data_shape", (), None)?;

        let mut dims = ArrayVec::new_const();

        for i in 0..=2 {
            dims.push(nii_shape.get_item(i)?.extract::<isize>()?);
        }

        dims.push(match nii_shape.len()? {
            3 => 1,
            4 => nii_shape.get_item(3)?.extract::<isize>()?,
            // TODO: specify which image is invalid
            _ => panic!("Unexpected dimensionality of the image (3 or 4 expected)"),
        });

        let fdata = nii_obj.call_method("get_fdata", (), None)?;

        Ok(NiiImage { fdata, dims })
    }

    fn nii_obj_res2nii_image_res(
        res: Result<(TargetImageDir<'a>, &'a PyAny), ErrorTy>,
    ) -> Result<(TargetImageDir<'a>, NiiImage<'a>), ErrorTy> {
        let (png_stub, nii_obj) = res?;
        let nii_image = Self::nii_obj2nii_image(nii_obj)?;
        Ok((png_stub, nii_image))
    }
}

impl<'a> Iterator for RelNiiImagesIter<'a> {
    type Item = Result<(TargetImageDir<'a>, NiiImage<'a>), ErrorTy>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Self::nii_obj_res2nii_image_res)
    }
}
