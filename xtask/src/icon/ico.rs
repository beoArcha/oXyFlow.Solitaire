//! Minimal ICO container with PNG-compressed entries.

use std::io;

const HEADER_LEN: usize = 6;
const ENTRY_LEN: usize = 16;
const TYPE_ICON: u16 = 1;
const PLANES: u16 = 1;
const BITS_PER_PIXEL: u16 = 32;
const FULL_SIZE: u32 = 256;

/// Builds an ICO file from `(size, png bytes)` pairs, in the given order.
pub fn build(entries: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let count = u16::try_from(entries.len()).unwrap_or(u16::MAX);
    let mut out = Vec::new();
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&TYPE_ICON.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    let mut offset = HEADER_LEN + ENTRY_LEN * entries.len();
    for (size, png) in entries {
        let dimension = dimension_byte(*size);
        out.extend_from_slice(&[dimension, dimension, 0, 0]);
        out.extend_from_slice(&PLANES.to_le_bytes());
        out.extend_from_slice(&BITS_PER_PIXEL.to_le_bytes());
        out.extend_from_slice(&u32::try_from(png.len()).unwrap_or(u32::MAX).to_le_bytes());
        out.extend_from_slice(&u32::try_from(offset).unwrap_or(u32::MAX).to_le_bytes());
        offset += png.len();
    }
    for (_, png) in entries {
        out.extend_from_slice(png);
    }
    out
}

fn dimension_byte(size: u32) -> u8 {
    if size >= FULL_SIZE {
        0
    } else {
        u8::try_from(size).unwrap_or(0)
    }
}

/// Reads an ICO file back into `(size, png bytes)` pairs.
pub fn parse(bytes: &[u8]) -> Result<Vec<(u32, &[u8])>, io::Error> {
    if read_u16(bytes, 2)? != TYPE_ICON {
        return Err(invalid("not an icon file"));
    }
    let count = usize::from(read_u16(bytes, 4)?);
    let mut entries = Vec::with_capacity(count);
    for index in 0..count {
        let at = HEADER_LEN + ENTRY_LEN * index;
        let size = match bytes.get(at) {
            Some(0) => FULL_SIZE,
            Some(value) => u32::from(*value),
            None => return Err(invalid("truncated directory")),
        };
        let length = read_usize(bytes, at + 8)?;
        let offset = read_usize(bytes, at + 12)?;
        let end = offset
            .checked_add(length)
            .ok_or_else(|| invalid("entry range overflows"))?;
        let png = bytes
            .get(offset..end)
            .ok_or_else(|| invalid("entry outside the file"))?;
        entries.push((size, png));
    }
    Ok(entries)
}

fn read_u16(bytes: &[u8], at: usize) -> Result<u16, io::Error> {
    match bytes.get(at..at + 2) {
        Some(&[a, b]) => Ok(u16::from_le_bytes([a, b])),
        _ => Err(invalid("truncated header")),
    }
}

fn read_usize(bytes: &[u8], at: usize) -> Result<usize, io::Error> {
    match bytes.get(at..at + 4) {
        Some(&[a, b, c, d]) => usize::try_from(u32::from_le_bytes([a, b, c, d]))
            .map_err(|_| invalid("value does not fit")),
        _ => Err(invalid("truncated directory")),
    }
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_owned())
}
