use crate::{
    error_ty::ErrorTy::{self, *},
    target_path::TargetImageDir,
};
use pyo3::{
    prelude::*,
    types::{PyIterator, PyUnicode},
};

/// Iterator over pairs of (png_stub, nii_obj) for all nii files in nii_files
/// where png_stub is a path to a directory where the png files
/// for the NIFTI volume will be saved and nii_obj is a NIFTI object (with a header and data)
pub(crate) struct RelNiiFilesIter<'a>
where
    Self: 'a,
{
    nib: &'a PyModule,
    os: &'a PyModule,
    nii_files: &'a str,
    base_png_stub: &'a PyUnicode,
    listdir_iter: &'a PyIterator,
}

impl<'a> RelNiiFilesIter<'a> {
    pub(crate) fn new(
        nib: &'a PyModule,
        os: &'a PyModule,
        nii_files: &'a str,
        base_png_stub: &'a PyUnicode,
    ) -> Result<Self, ErrorTy> {
        let listdir_iter = match os.call_method("listdir", (nii_files,), None) {
            Ok(listdir_res) => listdir_res.iter()?,
            Err(e) => return Err(ListDirFailed(e, nii_files.to_string())),
        };
        Ok(Self {
            nib,
            os,
            nii_files,
            listdir_iter,
            base_png_stub,
        })
    }

    fn png_stub(&self, nii_file: &PyAny) -> Result<TargetImageDir<'a>, ErrorTy> {
        // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L66
        let png_stub =
            self.os
                .getattr("path")?
                .call_method("join", (self.base_png_stub, nii_file), None)?;
        Ok(TargetImageDir(png_stub))
    }

    fn nii_obj(&self, nii_file: &PyAny) -> Result<&'a PyAny, ErrorTy> {
        // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L75
        let nii_obj = self.nib.call_method(
            "load",
            ({
                // nii_files + "\\" + nii_file
                self.os
                    .getattr("path")?
                    .call_method("join", (self.nii_files, nii_file), None)?
            },),
            None,
        )?;
        Ok(nii_obj)
    }

    // TODO: improve naming
    fn item(
        &self,
        nii_file: PyResult<&'a PyAny>,
    ) -> Result<
        (
            // png_stub
            TargetImageDir<'a>,
            // nii_obj
            &'a PyAny,
        ),
        ErrorTy,
    > {
        let nii_file = nii_file?;

        let png_stub = self.png_stub(nii_file)?;
        let nii_obj = self.nii_obj(nii_file)?;

        Ok((png_stub, nii_obj))
    }
}

impl<'a> Iterator for RelNiiFilesIter<'a> {
    type Item = Result<
        (
            // png_stub
            TargetImageDir<'a>,
            // nii_obj
            &'a PyAny,
        ),
        ErrorTy,
    >;

    fn next(&mut self) -> Option<Self::Item> {
        self.listdir_iter
            .next()
            .map(|nii_file_res| self.item(nii_file_res))
    }
}
