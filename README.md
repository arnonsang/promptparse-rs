# promptparse-rs

A Rust port of [PromptParse](https://github.com/maythiwat/promptparse).

## Installation

```sh
cargo add promptparse-rs
```

## Example

```rust
use promptparse::{generate::{any_id, ProxyType}, validate};

let payload = any_id(ProxyType::Mobile, "0812223333", Some(3000))?;
let transfer = validate::any_id(&payload)?;
assert_eq!(transfer.target, "0812223333");
assert_eq!(transfer.satang, Some(3000)); // 30.00 baht
# Ok::<(), promptparse::Error>(())
```

### Build a bill payment with named fields

```rust
use promptparse::{parse_checked, BillPayment};

let bill = BillPayment {
    biller_id: "099999999999990",
    ref1: "INV123",
    ref2: None,
    ref3: None,
    satang: Some(12550),
};
let payload = bill.to_payload()?;
let qr = parse_checked(&payload, "63")?;
assert_eq!(qr.get_subtag("30", "02")?, Some("INV123"));
# Ok::<(), promptparse::Error>(())
```

## Supported formats

| Format | Rust API |
| --- | --- |
| PromptPay AnyID and domestic bill payment | `generate::{any_id, bill_payment}` and matching functions in `validate` |
| TrueMoney Wallet, including a personal message | `generate::true_money` |
| Bank and TrueMoney receipt mini QR | `generate::{slip_verify, true_money_slip}` and matching functions in `validate` |
| BCEL OneProof | `validate::bcel_one_proof` extracts the type, ticket and reference |
| BOT bill-payment barcode | `BotBarcode::parse`, `to_string` and `to_qr_tag30` |
| Multi-scheme merchant QR | `merchant::accounts` identifies Visa-reserved fields, PromptPay transfer and domestic or cross-border billing. Other accounts retain their raw values. |

Use integer satang for baht amounts. In generators, `None` or zero produces an open-amount code. Tag 29 bank-account proxy type is reserved in the Thai standard. Bill-payment reference 3 follows upstream's convention for tag `62.07`. The generators encode supplied identifiers. Your bank or acquirer assigns the identifiers and determines which profiles it accepts.

For custom fields, `decode` returns tags in wire order, `Tag::children` reads a nested template, and `encode` recalculates field lengths after edits. Remove the old checksum before applying `with_crc_tag`. Use `parse_checked(payload, "63")` for merchant QR or `"91"` for a slip. For bank slips, `validate::slip_verify_strict` requires all four checksum digits. The default `slip_verify` also accepts a valid checksum with omitted leading zeros. Receipt verification requires a bank inquiry using the extracted reference.

Visa support covers offline inspection, including combined Visa and PromptPay payloads issued by SCB. Obtain a payable card QR from the merchant's acquirer. The research notes include the published example, Thai cross-border requirements and links to the specifications. [Payment QR research](docs/standards.md)

## Credits and license

Thank you to [Maythiwat Chomchuen](https://github.com/maythiwat) and the PromptParse contributors for the original TypeScript library, payment templates, and compatibility tests. Thank you to Theerawat Patthawee for the earlier Rust port that inspired the named payment fields, nested tag lookup, and slip checksum options. [Earlier Rust port](https://github.com/ttwrpz/promptparse-rs) · [Original PromptParse repository](https://github.com/maythiwat/promptparse) · [MIT license and copyright notices](LICENSE.md)
