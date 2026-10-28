#[test]
fn workspace_smoke_test() {
    let parsed = "1".parse::<u8>().unwrap();
    let expected = "1".parse::<u8>().expect("valid u8");
    assert_eq!(parsed, expected);
}
