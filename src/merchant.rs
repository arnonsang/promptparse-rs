//! Inspect the payment accounts advertised by an EMV merchant-presented payload.
use crate::{
    decode,
    generate::{ANY, BILL},
    parse_checked, Error, Result,
};
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Scheme {
    Visa,
    PromptPayAnyId,
    PromptPayBillPayment,
    PromptPayCrossBorderBillPayment,
    Other,
}
#[derive(Debug, PartialEq, Eq)]
pub struct MerchantAccount<'a> {
    pub tag: &'a str,
    pub scheme: Scheme,
    pub operator_id: Option<&'a str>,
    pub value: &'a str,
}
/// Return all account fields in wire order. Primitive account values are opaque.
pub fn accounts(payload: &str) -> Result<Vec<MerchantAccount<'_>>> {
    let qr = parse_checked(payload, "63")?;
    if qr.tags.first().map(|t| (t.id, t.value)) != Some(("00", "01")) {
        return Err(Error::InvalidField("merchant payload format"));
    }
    let mut accounts = Vec::new();
    for t in qr
        .tags
        .into_iter()
        .filter(|t| ("02"..="51").contains(&t.id))
    {
        let operator_id = if t.id >= "26" {
            let fields = decode(t.value)?;
            let mut ids = fields.iter().filter(|c| c.id == "00");
            let id = ids
                .next()
                .ok_or(Error::InvalidField("merchant operator ID"))?
                .value;
            if ids.next().is_some() {
                return Err(Error::InvalidField("duplicate operator ID"));
            }
            Some(id)
        } else {
            None
        };
        let scheme = match (t.id, operator_id) {
            ("02" | "03", _) => Scheme::Visa,
            ("29", Some(ANY)) => Scheme::PromptPayAnyId,
            ("30", Some(BILL)) => Scheme::PromptPayBillPayment,
            ("30", Some("A000000677012006")) => Scheme::PromptPayCrossBorderBillPayment,
            _ => Scheme::Other,
        };
        accounts.push(MerchantAccount {
            tag: t.id,
            value: t.value,
            operator_id,
            scheme,
        });
    }
    Ok(accounts)
}
