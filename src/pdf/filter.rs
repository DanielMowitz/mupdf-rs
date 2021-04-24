use mupdf_sys::*;

use std::ffi::CStr;
use std::mem;
use std::os::raw::{c_char, c_void};
use std::ptr;

use crate::pdf::PdfDocument;
use crate::{Image, Matrix, Rect};

#[derive(Clone, Copy)]
pub struct PdfFilterOptions {
    pub(crate) inner: pdf_filter_options,
}

// Callback types
pub type TextFilter = fn(ucsbuf: i32, ucslen: i32, trm: &Matrix, ctm: &Matrix, bbox: &Rect) -> i32;
pub type AfterTextObject = fn(doc: &PdfDocument, chain: &pdf_processor, ctm: &Matrix);
pub type EndPage = fn();

impl Default for PdfFilterOptions {
    fn default() -> Self {
        Self {
            inner: unsafe { mem::zeroed() },
        }
    }
}

impl PdfFilterOptions {
    pub fn ascii(&self) -> bool {
        self.inner.ascii != 0
    }

    pub fn set_ascii(&mut self, value: bool) -> &mut Self {
        self.inner.ascii = if value { 1 } else { 0 };
        self
    }

    pub fn recurse(&self) -> bool {
        self.inner.recurse != 0
    }

    pub fn set_recurse(&mut self, value: bool) -> &mut Self {
        self.inner.recurse = if value { 1 } else { 0 };
        self
    }

    pub fn sanitize(&self) -> bool {
        self.inner.sanitize != 0
    }

    pub fn set_sanitize(&mut self, value: bool) -> &mut Self {
        self.inner.sanitize = if value { 1 } else { 0 };
        self
    }

    pub fn instance_forms(&self) -> bool {
        self.inner.instance_forms != 0
    }

    pub fn set_instance_forms(&mut self, value: bool) -> &mut Self {
        self.inner.instance_forms = if value { 1 } else { 0 };
        self
    }

    /// Sets a callback for the filter, which will be given the initial
    /// transformation matrix, the image name (or "<inline>") and the image.
    ///
    /// The returned image has to be a new one, so `image.clone()` can be used
    /// to keep the same.
    pub fn set_image_filter<Cb>(&mut self, mut wrapper: Cb) -> &mut Self
    where
        Cb: FnMut(Matrix, &str, &Image) -> Option<Image>,
    {
        // The opaque field can be used to have data easily accessible in the
        // callback, in this case the user's closure.
        self.inner.opaque = &mut wrapper as *mut _ as *mut c_void;

        unsafe extern "C" fn image_filter_callback<Cb>(
            // TODO: `context()` inside this function should probably use the
            // parameter's value instead of the global value, right?
            _ctx: *mut fz_context,
            opaque: *mut c_void,
            ctm: fz_matrix,
            name: *const c_char,
            image: *mut fz_image, // Will be dropped after this callback is done
        ) -> *mut mupdf_sys::fz_image
        where
            Cb: FnMut(Matrix, &str, &Image) -> Option<Image>,
        {
            let ret = std::panic::catch_unwind(move || {
                // Reading the closure again
                let wrapper = &mut *(opaque as *mut Cb);

                wrapper(
                    Matrix::from(ctm),
                    CStr::from_ptr(name).to_str().unwrap(),
                    &Image::from_raw(image),
                )
            });

            if let Ok(Some(ret)) = ret {
                ret.inner
            } else {
                ptr::null_mut()
            }
        }

        self.inner.image_filter = Some(image_filter_callback::<Cb>);
        self
    }
}
