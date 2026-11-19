use promptparse::{generate::ProxyType, merchant::Scheme, *};

#[test]
fn upstream_payment_vectors() {
    let static_qr = "00020101021129370016A0000006770101110113006681222333353037645802TH63041DCF";
    assert_eq!(
        generate::any_id(ProxyType::Mobile, "0812223333", None).unwrap(),
        static_qr
    );
    assert_eq!(
        generate::any_id(ProxyType::Mobile, "0812223333", Some(3000)).unwrap(),
        "00020101021229370016A0000006770101110113006681222333353037645802TH540530.0063043CAD"
    );
    let bill = generate::bill_payment(
        "0112233445566",
        "CUSTOMER001",
        Some("INV001"),
        Some("SCB"),
        None,
    )
    .unwrap();
    assert_eq!(bill,"00020101021130620016A000000677010112011301122334455660211CUSTOMER0010306INV00153037645802TH62070703SCB6304780E");
    assert_eq!(validate::bill_payment(&bill).unwrap().ref3, Some("SCB"));
    assert!(validate::any_id(&bill).is_err());
    assert_eq!(generate::true_money("0801111111",Some(1005),Some("Hello World!")).unwrap(),"00020101021229390016A000000677010111031514000080111111153037645802TH540510.05814800480065006C006C006F00200057006F0072006C006400216304F5A2");
}
#[test]
fn upstream_receipt_vectors() {
    let s = generate::slip_verify("002", "0002123123121200011").unwrap();
    assert_eq!(
        s,
        "004000060000010103002021900021231231212000115102TH91049C30"
    );
    assert_eq!(validate::slip_verify(&s).unwrap().sending_bank, "002");
    let t = generate::true_money_slip("P2P", "123", "01012025").unwrap();
    assert_eq!(validate::true_money_slip(&t).unwrap().event_type, "P2P");
    assert!(validate::true_money_slip(&s).is_err());
}
#[test]
fn barcode_preserves_satang() {
    let raw = "|099400016550100\r123456789012\r670429\r364922";
    let b = BotBarcode::parse(raw).unwrap();
    assert_eq!(b.to_string(), raw);
    assert_eq!(b.to_qr_tag30().unwrap(),"00020101021230650016A00000067701011201150994000165501000212123456789012030667042953037645802TH54073649.2263044534");
    for bad in ["|b\rr\r\r1x", "|b\rr\r", "|b\rr\r\r1\rextra"] {
        assert!(BotBarcode::parse(bad).is_err())
    }
}
#[test]
fn unicode_and_malformed_lengths() {
    assert_eq!(checksum("ร้านทดสอบ"), 0x1EA0);
    let s="00020101021129370016A0000006770101110113006681222333353037645802TH5909ร้านทดสอบ6007กรุงเทพ63048A01";
    assert_eq!(parse_checked(s, "63").unwrap().get("59"), Some("ร้านทดสอบ"));
    assert_eq!(encode(&decode("5901😀").unwrap()).unwrap(), "5901😀");
    for bad in ["000411110104222202", "5900", "0104AB", "01x4test", "กxxx"] {
        assert!(decode(bad).is_err())
    }
    assert!(parse_checked(&s.replace("8A01", "ffff"), "63").is_err());
    assert!(parse_checked(&with_crc_tag("000201", "91").unwrap(), "63").is_err());
}
#[test]
fn scb_combined_merchant_sample() {
    // Siam Commercial Bank, QR 30 & QR CS example linked in docs/standards.md.
    let s="0002010102120216492984278344661530670016A00000067701011201151234567890123450210REFERENCE10310REFERENCE25204701153037645406100.005802TH5922TestMerchant15548661816007BANGKOK62610123201904191110032630000000523201904191110032630000000703SCB63043EA1";
    let a = merchant::accounts(s).unwrap();
    assert_eq!(a.len(), 2);
    assert_eq!(a[0].scheme, Scheme::Visa);
    assert_eq!(a[1].scheme, Scheme::PromptPayBillPayment);
    assert_eq!(validate::bill_payment(s).unwrap().satang, Some(10000));
}

#[test]
fn bcel_profile_requires_its_identifiers_and_reference_fields() {
    let body = "00020101021133290004BCEL0203P2P0305T12340401R";
    let proof = with_crc_tag(body, "63").unwrap();
    assert_eq!(validate::bcel_one_proof(&proof).unwrap().ticket, "T1234");
    let other = with_crc_tag(&body.replace("BCEL", "ELSE"), "63").unwrap();
    assert!(validate::bcel_one_proof(&other).is_err());
}
#[test]
fn merchant_operator_and_amount_edge_cases() {
    let qr = with_crc_tag("0002010303abc29100006OTHERA30200016A000000677012006", "63").unwrap();
    let accounts = merchant::accounts(&qr).unwrap();
    assert_eq!(
        accounts.iter().map(|a| a.scheme).collect::<Vec<_>>(),
        [
            Scheme::Visa,
            Scheme::Other,
            Scheme::PromptPayCrossBorderBillPayment
        ]
    );
    assert_eq!(accounts[1].operator_id, Some("OTHERA"));
    let qr = generate::any_id(ProxyType::NationalId, "1234567890123", Some(u64::MAX)).unwrap();
    assert_eq!(validate::any_id(&qr).unwrap().satang, Some(u64::MAX));
}
