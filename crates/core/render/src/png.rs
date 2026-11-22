use std::io;
use std::path::Path;

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;
    for &d in data {
        a = (a + d as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let mut body = Vec::with_capacity(data.len() + 4);
    body.extend_from_slice(kind);
    body.extend_from_slice(data);
    out.extend_from_slice(&body);
    out.extend_from_slice(&crc32(&body).to_be_bytes());
}

pub fn encode_png(width: usize, height: usize, rgba: &[u8]) -> Vec<u8> {
    assert_eq!(rgba.len(), width * height * 4, "rgba length mismatch");
    let mut raw = Vec::with_capacity((width * 4 + 1) * height);
    for row in rgba.chunks_exact(width * 4) {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    let mut z = vec![0x78, 0x01];
    let mut blocks = raw.chunks(65535).peekable();
    if blocks.peek().is_none() {
        z.extend_from_slice(&[1, 0, 0, 0xFF, 0xFF]);
    }
    while let Some(b) = blocks.next() {
        let last = blocks.peek().is_none();
        z.push(u8::from(last));
        z.extend_from_slice(&(b.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(b.len() as u16)).to_le_bytes());
        z.extend_from_slice(b);
    }
    z.extend_from_slice(&adler32(&raw).to_be_bytes());
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    out
}

pub fn write_png(path: impl AsRef<Path>, width: usize, height: usize, rgba: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, encode_png(width, height, rgba))
}

pub fn decode_png_rgba(data: &[u8]) -> Option<(usize, usize, Vec<u8>)> {
    if data.len() < 8 || data[..8] != [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
        return None;
    }
    let mut pos = 8;
    let mut width = 0usize;
    let mut height = 0usize;
    let mut idat = Vec::new();
    while pos + 8 <= data.len() {
        let len = u32::from_be_bytes(data[pos..pos + 4].try_into().ok()?) as usize;
        let kind = &data[pos + 4..pos + 8];
        let body = data.get(pos + 8..pos + 8 + len)?;
        if kind == b"IHDR" {
            width = u32::from_be_bytes(body[0..4].try_into().ok()?) as usize;
            height = u32::from_be_bytes(body[4..8].try_into().ok()?) as usize;
            if body[8] != 8 || body[9] != 6 {
                return None;
            }
        } else if kind == b"IDAT" {
            idat.extend_from_slice(body);
        }
        pos += 12 + len;
    }
    let mut raw = Vec::new();
    let mut p = 2;
    loop {
        let hdr = *idat.get(p)?;
        let len = u16::from_le_bytes([*idat.get(p + 1)?, *idat.get(p + 2)?]) as usize;
        raw.extend_from_slice(idat.get(p + 5..p + 5 + len)?);
        p += 5 + len;
        if hdr & 1 == 1 {
            break;
        }
    }
    let stride = width * 4 + 1;
    if raw.len() != stride * height {
        return None;
    }
    let mut out = Vec::with_capacity(width * height * 4);
    for row in raw.chunks_exact(stride) {
        if row[0] != 0 {
            return None;
        }
        out.extend_from_slice(&row[1..]);
    }
    Some((width, height, out))
}
