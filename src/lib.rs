#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod barcode;
pub mod generate;
pub mod merchant;
pub mod validate;
pub use barcode::BotBarcode;

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidHeader,
    InvalidLength,
    InvalidChecksum,
    InvalidField(&'static str),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHeader => f.write_str("expected a two-digit tag and a two-digit length"),
            Self::InvalidLength => f.write_str("invalid TLV value length"),
            Self::InvalidChecksum => f.write_str("invalid or missing checksum"),
            Self::InvalidField(name) => write!(f, "invalid or missing {name}"),
        }
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

/// An ordered TLV field borrowing its identifier and value from the payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tag<'a> {
    pub id: &'a str,
    pub value: &'a str,
}
impl<'a> Tag<'a> {
    pub fn new(id: &'a str, value: &'a str) -> Self {
        Self { id, value }
    }
    /// Decode the value when the enclosing profile defines it as a template.
    pub fn children(self) -> Result<Vec<Tag<'a>>> {
        decode(self.value)
    }
}
fn valid_id(id: &str) -> bool {
    id.len() == 2 && id.bytes().all(|b| b.is_ascii_digit())
}
/// Decode a complete payload, counting Unicode scalar values for lengths.
pub fn decode(mut data: &str) -> Result<Vec<Tag<'_>>> {
    let mut fields = Vec::new();
    while !data.is_empty() {
        let h = data.as_bytes().get(..4).ok_or(Error::InvalidHeader)?;
        if !h.iter().all(u8::is_ascii_digit) {
            return Err(Error::InvalidHeader);
        }
        let len = usize::from(h[2] - b'0') * 10 + usize::from(h[3] - b'0');
        let value = &data[4..];
        let (i, c) = value
            .char_indices()
            .nth(len.checked_sub(1).ok_or(Error::InvalidLength)?)
            .ok_or(Error::InvalidLength)?;
        let end = i + c.len_utf8();
        fields.push(Tag::new(&data[..2], &value[..end]));
        data = &value[end..];
    }
    if fields.is_empty() {
        return Err(Error::InvalidHeader);
    }
    Ok(fields)
}
/// Encode fields in input order, recalculating every length.
pub fn encode(tags: &[Tag<'_>]) -> Result<String> {
    use fmt::Write;
    let mut out = String::new();
    for tag in tags {
        if !valid_id(tag.id) {
            return Err(Error::InvalidHeader);
        }
        let n = tag.value.chars().count();
        if !(1..=99).contains(&n) {
            return Err(Error::InvalidLength);
        }
        write!(out, "{}{:02}{}", tag.id, n, tag.value).unwrap();
    }
    Ok(out)
}
/// CRC-16 with polynomial 0x1021 and initial value 0xFFFF over UTF-8 bytes.
pub fn checksum(payload: &str) -> u16 {
    let mut crc = 0xffffu16;
    for b in payload.bytes() {
        crc ^= u16::from(b) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}
pub fn with_crc_tag(body: &str, id: &str) -> Result<String> {
    if !valid_id(id) {
        return Err(Error::InvalidHeader);
    }
    let head = format!("{body}{id}04");
    Ok(format!("{head}{:04X}", checksum(&head)))
}
/// Ordered fields and the original input, with optional profile-specific CRC verification.
#[derive(Debug)]
pub struct Qr<'a> {
    pub payload: &'a str,
    pub tags: Vec<Tag<'a>>,
}
pub fn parse(payload: &str) -> Result<Qr<'_>> {
    Ok(Qr {
        payload,
        tags: decode(payload)?,
    })
}
impl<'a> Qr<'a> {
    pub fn get(&self, id: &str) -> Option<&'a str> {
        self.tags.iter().find(|t| t.id == id).map(|t| t.value)
    }
}
/// Require one final CRC field with the supplied ID. Accept either hex case.
pub fn parse_checked<'a>(payload: &'a str, crc_id: &str) -> Result<Qr<'a>> {
    let qr = parse(payload)?;
    let (last, rest) = qr.tags.split_last().ok_or(Error::InvalidChecksum)?;
    if last.id != crc_id
        || rest.iter().any(|t| t.id == crc_id)
        || last.value.len() != 4
        || !last.value.bytes().all(|b| b.is_ascii_hexdigit())
        || u16::from_str_radix(last.value, 16).ok() != Some(checksum(&payload[..payload.len() - 4]))
    {
        return Err(Error::InvalidChecksum);
    }
    Ok(qr)
}
