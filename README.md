# promptparse-rs

A Rust port of [PromptParse](https://github.com/maythiwat/promptparse).

## Example

```rust
use promptparse::{generate::{any_id, ProxyType}, validate};

let payload = any_id(ProxyType::Mobile, "0812223333", Some(3000))?;
let transfer = validate::any_id(&payload)?;
assert_eq!(transfer.target, "0812223333");
assert_eq!(transfer.satang, Some(3000)); // 30.00 baht
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

For custom fields, `decode` returns tags in wire order, `Tag::children` reads a nested template, and `encode` recalculates field lengths after edits. Remove the old checksum before applying `with_crc_tag`. Use `parse_checked(payload, "63")` for merchant QR or `"91"` for a slip. Receipt verification requires a bank inquiry using the extracted reference.

Visa support covers offline inspection, including combined Visa and PromptPay payloads issued by SCB. Obtain a payable card QR from the merchant's acquirer. The research notes include the published example, Thai cross-border requirements and links to the specifications. [Payment QR research](docs/standards.md)

## Credits and license

Thank you to [Maythiwat Chomchuen](https://github.com/maythiwat) and the PromptParse contributors for the original TypeScript library, payment templates, and compatibility tests.  [Original PromptParse repository](https://github.com/maythiwat/promptparse) · [MIT license and copyright notices](LICENSE.md)
