use pcap_transfer::core::csv::expr::{
    parse_cell_value_by_type, DefaultExpr,
};
use pcap_transfer::core::csv::types::CsvDataType;

#[test]
fn test_parse_rand_expr_integration() {
    let expr = DefaultExpr::parse(
        "rand(10,20)",
        &CsvDataType::I32,
    )
    .unwrap();
    assert!(matches!(
        expr,
        DefaultExpr::RandInt { min: 10, max: 20 }
    ));

    let expr = DefaultExpr::parse(
        "rand(1.5,2.5)",
        &CsvDataType::F32,
    )
    .unwrap();
    assert!(
        matches!(expr, DefaultExpr::RandFloat { min, max } if min == 1.5 && max == 2.5)
    );

    let expr =
        DefaultExpr::parse("rand()", &CsvDataType::Bool)
            .unwrap();
    assert!(matches!(expr, DefaultExpr::RandBool));
}

#[test]
fn test_parse_loop_expr_integration() {
    let expr = DefaultExpr::parse(
        "loop(11,22,33)",
        &CsvDataType::I32,
    )
    .unwrap();
    if let DefaultExpr::Loop(items) = expr {
        assert_eq!(items, vec!["11", "22", "33"]);
    } else {
        panic!("Expected Loop variant");
    }
}

#[test]
fn test_evaluate_loop_integration() {
    let expr = DefaultExpr::Loop(vec![
        "11".to_string(),
        "22".to_string(),
    ]);
    let bytes0 =
        expr.evaluate(&CsvDataType::I32, 0).unwrap();
    assert_eq!(bytes0, 11i32.to_le_bytes().to_vec());
    let bytes1 =
        expr.evaluate(&CsvDataType::I32, 1).unwrap();
    assert_eq!(bytes1, 22i32.to_le_bytes().to_vec());
    let bytes2 =
        expr.evaluate(&CsvDataType::I32, 2).unwrap();
    assert_eq!(bytes2, 11i32.to_le_bytes().to_vec());
}

#[test]
fn test_parse_cell_value_hex_integration() {
    let ty = CsvDataType::HexDynamic(2);
    let bytes =
        parse_cell_value_by_type(&ty, "0x1234").unwrap();
    assert_eq!(bytes, vec![0x12, 0x34]);
}
