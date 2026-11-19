use crate::{generate, Error, Result};
use std::fmt;
/// BOT bill-payment barcode. Amounts use integer satang.
#[derive(Debug, PartialEq, Eq)]
pub struct BotBarcode<'a> {
    pub biller_id: &'a str,
    pub ref1: &'a str,
    pub ref2: Option<&'a str>,
    pub satang: Option<u64>,
}
impl<'a> BotBarcode<'a> {
    pub fn parse(payload: &'a str) -> Result<Self> {
        let mut parts = payload
            .strip_prefix('|')
            .ok_or(Error::InvalidField("barcode prefix"))?
            .split('\r');
        let biller_id = parts
            .next()
            .filter(|s| !s.is_empty())
            .ok_or(Error::InvalidField("biller ID"))?;
        let ref1 = parts
            .next()
            .filter(|s| !s.is_empty())
            .ok_or(Error::InvalidField("reference"))?;
        let ref2 = parts.next().ok_or(Error::InvalidField("barcode fields"))?;
        let amount = parts
            .next()
            .filter(|v| v.bytes().all(|b| b.is_ascii_digit()))
            .and_then(|v| v.parse::<u64>().ok())
            .ok_or(Error::InvalidField("satang"))?;
        if parts.next().is_some() {
            return Err(Error::InvalidField("barcode fields"));
        }
        Ok(Self {
            biller_id,
            ref1,
            ref2: (!ref2.is_empty()).then_some(ref2),
            satang: (amount != 0).then_some(amount),
        })
    }
    pub fn to_qr_tag30(&self) -> Result<String> {
        generate::bill_payment(self.biller_id, self.ref1, self.ref2, None, self.satang)
    }
}
impl fmt::Display for BotBarcode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "|{}\r{}\r{}\r{}",
            self.biller_id,
            self.ref1,
            self.ref2.unwrap_or_default(),
            self.satang.unwrap_or_default()
        )
    }
}
