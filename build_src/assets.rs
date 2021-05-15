use std::{fs, path::Path};

use siru::logging::log_info;

use crate::errors::Result;
use crate::BuildContext;

fn copy_dir_recursive(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_recursive(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn copy_assets(ctx: &BuildContext) -> Result<()> {
    log_info("Copying assets…");
    copy_dir_recursive(ctx.source_dir.join("assets"), ctx.output_dir.join("assets"))?;
    Ok(())
}
