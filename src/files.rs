use std::path::PathBuf;

use time::formatting::Formattable;

#[allow(clippy::ptr_arg)] // the whole function is just to here for making it easier to read
#[must_use] pub fn is_json(p: &PathBuf) -> bool {
    match p.extension() {
        Some(x) => x == "json",
        None => false,
    }
}

pub trait ToFileName {
    type Error;
    fn to_file_name(
        &self,
        time_format_descriptor: &(impl Formattable + ?Sized),
    ) -> Result<String, Self::Error>;
}
