//! Payload generators. All amounts are integer satang (100 satang = 1 baht).
use crate::{encode, with_crc_tag, Error, Result, Tag};
use std::fmt::Write;

pub(crate) const ANY: &str = "A000000677010111";
pub(crate) const BILL: &str = "A000000677010112";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyType {
    Mobile,
    NationalId,
    EWallet,
    BankAccount,
}
impl ProxyType {
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::Mobile => "01",
            Self::NationalId => "02",
            Self::EWallet => "03",
            Self::BankAccount => "04",
        }
    }
}
fn payment(id: &str, account: &str, satang: Option<u64>, extra: Option<Tag<'_>>) -> Result<String> {
    let amount = satang
        .filter(|n| *n != 0)
        .map(|n| format!("{}.{:02}", n / 100, n % 100));
    let mut fields = vec![
        Tag::new("00", "01"),
        Tag::new("01", if amount.is_some() { "12" } else { "11" }),
        Tag::new(id, account),
        Tag::new("53", "764"),
        Tag::new("58", "TH"),
    ];
    if let Some(v) = &amount {
        fields.push(Tag::new("54", v))
    }
    fields.extend(extra);
    with_crc_tag(&encode(&fields)?, "63")
}
pub fn any_id(kind: ProxyType, target: &str, satang: Option<u64>) -> Result<String> {
    let mobile;
    let value = if kind == ProxyType::Mobile {
        let local = target
            .strip_prefix("0066")
            .or_else(|| target.strip_prefix("+66"))
            .or_else(|| target.strip_prefix("66"))
            .or_else(|| target.strip_prefix('0'))
            .ok_or(Error::InvalidField("mobile"))?;
        if local.len() != 9 || !local.bytes().all(|b| b.is_ascii_digit()) {
            return Err(Error::InvalidField("mobile"));
        }
        mobile = format!("0066{local}");
        &mobile
    } else {
        target
    };
    payment(
        "29",
        &encode(&[Tag::new("00", ANY), Tag::new(kind.id(), value)])?,
        satang,
        None,
    )
}
pub fn bill_payment(
    biller: &str,
    reference: &str,
    ref2: Option<&str>,
    ref3: Option<&str>,
    satang: Option<u64>,
) -> Result<String> {
    let mut fields = vec![
        Tag::new("00", BILL),
        Tag::new("01", biller),
        Tag::new("02", reference),
    ];
    if let Some(v) = ref2 {
        fields.push(Tag::new("03", v))
    }
    let additional = ref3.map(|v| encode(&[Tag::new("07", v)])).transpose()?;
    payment(
        "30",
        &encode(&fields)?,
        satang,
        additional.as_deref().map(|v| Tag::new("62", v)),
    )
}
pub fn true_money(mobile: &str, satang: Option<u64>, message: Option<&str>) -> Result<String> {
    if mobile.len() != 10 || !mobile.starts_with('0') || !mobile.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(Error::InvalidField("mobile"));
    }
    let account = encode(&[
        Tag::new("00", ANY),
        Tag::new("03", &format!("14000{mobile}")),
    ])?;
    let mut hex = String::new();
    for unit in message.unwrap_or_default().encode_utf16() {
        write!(hex, "{unit:04X}").unwrap()
    }
    payment(
        "29",
        &account,
        satang,
        (!hex.is_empty()).then_some(Tag::new("81", &hex)),
    )
}
pub fn slip_verify(bank: &str, reference: &str) -> Result<String> {
    let info = encode(&[
        Tag::new("00", "000001"),
        Tag::new("01", bank),
        Tag::new("02", reference),
    ])?;
    with_crc_tag(
        &encode(&[Tag::new("00", &info), Tag::new("51", "TH")])?,
        "91",
    )
}
pub fn true_money_slip(event: &str, transaction: &str, date: &str) -> Result<String> {
    let info = encode(&[
        Tag::new("00", "01"),
        Tag::new("01", "01"),
        Tag::new("02", event),
        Tag::new("03", transaction),
        Tag::new("04", date),
    ])?;
    let mut result = with_crc_tag(&encode(&[Tag::new("00", &info)])?, "91")?;
    let pos = result.len() - 4;
    result[pos..].make_ascii_lowercase();
    Ok(result)
}
