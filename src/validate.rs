//! Verify the CRC and extract fields for a known profile. A slip reference can be used for a bank inquiry.
use crate::{
    decode,
    generate::{ProxyType, ANY, BILL},
    parse_checked, Error, Qr, Result, Tag,
};

fn field<'a>(tags: &[Tag<'a>], id: &'static str) -> Result<&'a str> {
    let mut matches = tags.iter().filter(|t| t.id == id);
    let v = matches.next().ok_or(Error::InvalidField(id))?;
    if matches.next().is_some() {
        return Err(Error::InvalidField(id));
    }
    Ok(v.value)
}
fn template<'a>(qr: &Qr<'a>, id: &'static str) -> Result<Vec<Tag<'a>>> {
    decode(field(&qr.tags, id)?)
}
fn optional<'a>(tags: &[Tag<'a>], id: &'static str) -> Result<Option<&'a str>> {
    tags.iter()
        .find(|t| t.id == id)
        .map(|_| field(tags, id))
        .transpose()
}
fn account<'a>(qr: &Qr<'a>, id: &'static str, aid: &str) -> Result<Vec<Tag<'a>>> {
    let t = template(qr, id)?;
    if field(&qr.tags, "00")? != "01" || field(&t, "00")? != aid {
        return Err(Error::InvalidField("application ID"));
    }
    if field(&qr.tags, "53")? != "764" {
        return Err(Error::InvalidField("THB currency"));
    }
    Ok(t)
}
fn satang(qr: &Qr<'_>) -> Result<Option<u64>> {
    let Some(s) = optional(&qr.tags, "54")? else {
        return Ok(None);
    };
    let (whole, frac) = match s.split_once('.') {
        Some((w, f)) if !f.is_empty() => (w, f),
        Some(_) => return Err(Error::InvalidField("amount")),
        None => (s, ""),
    };
    if whole.is_empty()
        || frac.len() > 2
        || !whole
            .bytes()
            .chain(frac.bytes())
            .all(|b| b.is_ascii_digit())
    {
        return Err(Error::InvalidField("amount"));
    }
    let cents = frac.parse::<u64>().unwrap_or(0) * if frac.len() == 1 { 10 } else { 1 };
    whole
        .parse::<u64>()
        .ok()
        .and_then(|n| n.checked_mul(100))
        .and_then(|n| n.checked_add(cents))
        .map(Some)
        .ok_or(Error::InvalidField("amount"))
}
#[derive(Debug, PartialEq, Eq)]
pub struct AnyId {
    pub kind: ProxyType,
    pub target: String,
    pub satang: Option<u64>,
}
pub fn any_id(payload: &str) -> Result<AnyId> {
    let qr = parse_checked(payload, "63")?;
    let t = account(&qr, "29", ANY)?;
    let mut proxies = [
        ProxyType::Mobile,
        ProxyType::NationalId,
        ProxyType::EWallet,
        ProxyType::BankAccount,
    ]
    .into_iter()
    .filter(|k| t.iter().any(|v| v.id == k.id()));
    let kind = proxies.next().ok_or(Error::InvalidField("proxy"))?;
    if proxies.next().is_some() {
        return Err(Error::InvalidField("ambiguous proxy"));
    }
    let value = field(&t, kind.id())?;
    let target = match (kind, value.strip_prefix("0066")) {
        (ProxyType::Mobile, Some(n)) if n.len() == 9 && n.bytes().all(|b| b.is_ascii_digit()) => {
            format!("0{n}")
        }
        _ => value.to_owned(),
    };
    Ok(AnyId {
        kind,
        target,
        satang: satang(&qr)?,
    })
}
#[derive(Debug, PartialEq, Eq)]
pub struct BillPayment<'a> {
    pub biller_id: &'a str,
    pub ref1: &'a str,
    pub ref2: Option<&'a str>,
    pub ref3: Option<&'a str>,
    pub satang: Option<u64>,
}
pub fn bill_payment(payload: &str) -> Result<BillPayment<'_>> {
    let qr = parse_checked(payload, "63")?;
    let t = account(&qr, "30", BILL)?;
    let additional = optional(&qr.tags, "62")?
        .map(decode)
        .transpose()?
        .unwrap_or_default();
    Ok(BillPayment {
        biller_id: field(&t, "01")?,
        ref1: field(&t, "02")?,
        ref2: optional(&t, "03")?,
        ref3: optional(&additional, "07")?,
        satang: satang(&qr)?,
    })
}
#[derive(Debug, PartialEq, Eq)]
pub struct Slip {
    pub sending_bank: String,
    pub trans_ref: String,
}
/// Accept a bank slip with omitted leading checksum zeros if the repaired CRC matches.
pub fn slip_verify(payload: &str) -> Result<Slip> {
    let repaired;
    let mut data = payload;
    if let Some((body, crc)) = payload.rsplit_once("9104") {
        if (1..4).contains(&crc.len()) && crc.bytes().all(|b| b.is_ascii_hexdigit()) {
            repaired = format!("{body}9104{crc:0>4}");
            data = &repaired;
        }
    }
    let qr = parse_checked(data, "91")?;
    let t = template(&qr, "00")?;
    if field(&t, "00")? != "000001" {
        return Err(Error::InvalidField("slip type"));
    }
    Ok(Slip {
        sending_bank: field(&t, "01")?.into(),
        trans_ref: field(&t, "02")?.into(),
    })
}
#[derive(Debug, PartialEq, Eq)]
pub struct TrueMoneySlip<'a> {
    pub event_type: &'a str,
    pub transaction_id: &'a str,
    pub date: &'a str,
}
pub fn true_money_slip(payload: &str) -> Result<TrueMoneySlip<'_>> {
    let qr = parse_checked(payload, "91")?;
    let t = template(&qr, "00")?;
    if field(&t, "00")? != "01" || field(&t, "01")? != "01" {
        return Err(Error::InvalidField("TrueMoney slip type"));
    }
    Ok(TrueMoneySlip {
        event_type: field(&t, "02")?,
        transaction_id: field(&t, "03")?,
        date: field(&t, "04")?,
    })
}
#[derive(Debug, PartialEq, Eq)]
pub struct BcelOneProof<'a> {
    pub kind: &'a str,
    pub ticket: &'a str,
    pub fccref: &'a str,
}
pub fn bcel_one_proof(payload: &str) -> Result<BcelOneProof<'_>> {
    let qr = parse_checked(payload, "63")?;
    let t = template(&qr, "33")?;
    if field(&qr.tags, "00")? != "01"
        || field(&qr.tags, "01")? != "11"
        || !matches!(field(&t, "00")?, "BCEL" | "ONEPROOF")
    {
        return Err(Error::InvalidField("BCEL OneProof"));
    }
    Ok(BcelOneProof {
        kind: field(&t, "02")?,
        ticket: field(&t, "03")?,
        fccref: field(&t, "04")?,
    })
}
