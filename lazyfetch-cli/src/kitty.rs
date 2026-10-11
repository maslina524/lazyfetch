use core::fmt::Write;

use crate::{base64, format, image::Image, print::Stdout};

const CHUNK: usize = 4096;

pub fn print_image(
    image: &Image,
    cols: Option<usize>,
    rows: Option<usize>,
    image_id: u32,
) -> core::fmt::Result {
    let (w, h) = image.get_size();

    let raw = image.as_rgba_bytes();
    let b64 = base64::encode(&raw);

    let mut ctrl = format!("a=T,f=32,s={w},v={h},C=1");
    if image_id != 0 {
        ctrl.push_str(&format!(",i={image_id}"));
    }
    if let Some(c) = cols {
        ctrl.push_str(&format!(",c={c}"));
    }
    if let Some(r) = rows {
        ctrl.push_str(&format!(",r={r}"));
    }

    let bytes = b64.as_bytes();

    let mut first = true;
    let mut offset = 0usize;
    while offset < bytes.len() {
        let end = (offset + CHUNK).min(bytes.len());
        let more = end < bytes.len();
        let m = i32::from(more);

        if first {
            let ctrl_with_m = format_args!("{ctrl},m={m}");
            Stdout::get().write_fmt(format_args!("\x1b_G{ctrl_with_m};"))?;
            first = false;
        } else {
            Stdout::get().write_fmt(format_args!("\x1b_Gm={m};"))?;
        }

        Stdout::get().write_bytes(&bytes[offset..end]);
        Stdout::get().write_bytes(b"\x1b\\");

        offset = end;
    }

    Ok(())
}
