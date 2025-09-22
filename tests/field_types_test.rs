//! 字段类型系统测试
//!
//! 测试字段类型解析、值转换和消息生成功能

use param_sender::app::config::message_types::{
    MessageDefinition, MessageField, MessageRuntimeState,
};
use param_sender::core::field_types::{
    parse_field_value_by_type, FieldDataType,
};

#[test]
fn test_field_type_parsing() {
    // 测试基本类型解析
    let (data_type, _) =
        FieldDataType::parse_type_and_default("u32")
            .unwrap();
    assert_eq!(data_type, FieldDataType::U32);

    // 测试带默认值的类型解析
    let (data_type, expr) =
        FieldDataType::parse_type_and_default("hex=0x01")
            .unwrap();
    assert_eq!(data_type, FieldDataType::HexDynamic(1));
    assert!(expr.is_some());

    // 测试十六进制类型
    let (data_type, _) =
        FieldDataType::parse_type_and_default("hex_4")
            .unwrap();
    assert_eq!(data_type, FieldDataType::HexDynamic(4));
}

#[test]
#[allow(clippy::approx_constant)]
fn test_field_value_conversion() {
    // 测试整数转换
    let bytes = parse_field_value_by_type(
        &FieldDataType::U32,
        "1234",
    )
    .unwrap();
    assert_eq!(bytes.len(), 4);
    assert_eq!(
        u32::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3]
        ]),
        1234
    );

    // 测试浮点数转换
    let bytes = parse_field_value_by_type(
        &FieldDataType::F32,
        "3.14",
    )
    .unwrap();
    assert_eq!(bytes.len(), 4);
    let value = f32::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3],
    ]);
    assert!((value - 3.14).abs() < 0.001);

    // 测试十六进制转换
    let bytes = parse_field_value_by_type(
        &FieldDataType::HexDynamic(1),
        "0xFF",
    )
    .unwrap();
    assert_eq!(bytes, vec![0xFF]);
}

#[test]
fn test_error_handling() {
    // 测试无效类型
    assert!(FieldDataType::parse_type_and_default(
        "invalid_type"
    )
    .is_err());

    // 测试无效值转换
    assert!(parse_field_value_by_type(
        &FieldDataType::U32,
        "not_a_number"
    )
    .is_err());
    assert!(parse_field_value_by_type(
        &FieldDataType::F32,
        "invalid_float"
    )
    .is_err());
}

#[test]
fn test_boolean_conversion() {
    // 测试布尔值转换
    let bytes = parse_field_value_by_type(
        &FieldDataType::Bool,
        "true",
    )
    .unwrap();
    assert_eq!(bytes, vec![1]);

    let bytes = parse_field_value_by_type(
        &FieldDataType::Bool,
        "false",
    )
    .unwrap();
    assert_eq!(bytes, vec![0]);
}

#[test]
fn test_hex_validation() {
    // 测试有效的十六进制
    let bytes = parse_field_value_by_type(
        &FieldDataType::HexDynamic(2),
        "0x1234",
    )
    .unwrap();
    assert_eq!(bytes, vec![0x12, 0x34]);

    // 测试无前缀的十六进制
    let bytes = parse_field_value_by_type(
        &FieldDataType::HexDynamic(1),
        "FF",
    )
    .unwrap();
    assert_eq!(bytes, vec![0xFF]);
}

#[test]
fn test_message_packet_generation() {
    // 创建测试消息定义
    let message_def = MessageDefinition {
        name: "test_message".to_string(),
        interval: 1000,
        enabled: true,
        packet_count: 0,
        network: None,
        fields: vec![
            MessageField {
                name: "version".to_string(),
                field_type: "u8=1".to_string(),
                editable: false,
            },
            MessageField {
                name: "data".to_string(),
                field_type: "u32".to_string(),
                editable: true,
            },
        ],
    };

    // 创建运行时状态
    let runtime_state =
        MessageRuntimeState::from_definition(message_def);

    // 生成数据包
    let packet =
        runtime_state.generate_packet(0, None).unwrap();
    assert_eq!(packet.len(), 5); // u8 + u32 = 1 + 4 = 5 bytes
    assert_eq!(packet[0], 1); // version = 1
    assert_eq!(
        u32::from_le_bytes([
            packet[1], packet[2], packet[3], packet[4]
        ]),
        0
    ); // data = 0 (default)
}

#[test]
fn test_random_expression_parsing() {
    // 测试随机整数表达式
    let (data_type, expr) =
        FieldDataType::parse_type_and_default(
            "i32=rand(100,200)",
        )
        .unwrap();
    assert_eq!(data_type, FieldDataType::I32);
    assert!(expr.is_some());

    // 测试随机浮点表达式
    let (data_type, expr) =
        FieldDataType::parse_type_and_default(
            "f32=rand(0.0,1.0)",
        )
        .unwrap();
    assert_eq!(data_type, FieldDataType::F32);
    assert!(expr.is_some());

    // 测试随机布尔表达式
    let (data_type, expr) =
        FieldDataType::parse_type_and_default(
            "bool=rand()",
        )
        .unwrap();
    assert_eq!(data_type, FieldDataType::Bool);
    assert!(expr.is_some());
}

#[test]
fn test_loop_expression_parsing() {
    // 测试循环表达式
    let (data_type, expr) =
        FieldDataType::parse_type_and_default(
            "u8=loop(1,2,3)",
        )
        .unwrap();
    assert_eq!(data_type, FieldDataType::U8);
    assert!(expr.is_some());
}

#[test]
fn test_switch_expression_parsing() {
    use param_sender::core::field_types::expr::types::SwitchCondition;
    use param_sender::core::field_types::DefaultExpr;

    // 测试基本 switch 表达式
    let (data_type, expr) =
        FieldDataType::parse_type_and_default(
            "u8=switch(100, -2:200)",
        )
        .unwrap();
    assert_eq!(data_type, FieldDataType::U8);
    assert!(expr.is_some());

    if let Some(DefaultExpr::Switch {
        default_value,
        rules,
    }) = expr
    {
        assert_eq!(default_value, "100");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].value, "200");
        if let SwitchCondition::Relative(offset) =
            rules[0].condition
        {
            assert_eq!(offset, -2);
        } else {
            panic!("Expected Relative condition");
        }
    } else {
        panic!("Expected Switch expression");
    }
}

#[test]
fn test_switch_expression_evaluation() {
    // 测试 switch 表达式求值
    let (data_type, expr) =
        FieldDataType::parse_type_and_default(
            "u8=switch(100, -2:200, 3:150)",
        )
        .unwrap();

    if let Some(expr) = expr {
        // 测试前几个包（使用默认值）
        let result1 =
            expr.evaluate(&data_type, 0, Some(10)).unwrap();
        assert_eq!(result1, vec![100u8]);

        let result2 =
            expr.evaluate(&data_type, 1, Some(10)).unwrap();
        assert_eq!(result2, vec![100u8]);

        // 测试第3个包（索引2，应该使用值150）
        let result3 =
            expr.evaluate(&data_type, 2, Some(10)).unwrap();
        assert_eq!(result3, vec![150u8]);

        // 测试倒数第2个包（索引8，包编号9，应该使用值200）
        let result4 =
            expr.evaluate(&data_type, 8, Some(10)).unwrap();
        assert_eq!(result4, vec![200u8]);

        // 测试最后一个包（索引9，包编号10，应该使用默认值100，因为-2只匹配倒数第2个）
        let result5 =
            expr.evaluate(&data_type, 9, Some(10)).unwrap();
        assert_eq!(result5, vec![100u8]);
    } else {
        panic!("Expected expression");
    }
}

#[test]
fn test_switch_range_condition() {
    // 测试范围条件
    let (data_type, expr) =
        FieldDataType::parse_type_and_default(
            "u8=switch(100, 3-5:150)",
        )
        .unwrap();

    if let Some(expr) = expr {
        // 测试范围外的包
        let result1 =
            expr.evaluate(&data_type, 0, Some(10)).unwrap();
        assert_eq!(result1, vec![100u8]);

        let result2 =
            expr.evaluate(&data_type, 1, Some(10)).unwrap();
        assert_eq!(result2, vec![100u8]);

        // 测试范围内的包（第3-5个包，索引2-4）
        let result3 =
            expr.evaluate(&data_type, 2, Some(10)).unwrap();
        assert_eq!(result3, vec![150u8]);

        let result4 =
            expr.evaluate(&data_type, 3, Some(10)).unwrap();
        assert_eq!(result4, vec![150u8]);

        let result5 =
            expr.evaluate(&data_type, 4, Some(10)).unwrap();
        assert_eq!(result5, vec![150u8]);

        // 测试范围外的包
        let result6 =
            expr.evaluate(&data_type, 5, Some(10)).unwrap();
        assert_eq!(result6, vec![100u8]);
    } else {
        panic!("Expected expression");
    }
}

#[test]
fn test_switch_infinite_packets() {
    // 测试无限包模式（packet_count = 0）
    let (data_type, expr) =
        FieldDataType::parse_type_and_default(
            "u8=switch(100, 5:150, -2:200)",
        )
        .unwrap();

    if let Some(expr) = expr {
        // 无限模式下，只有绝对位置条件有效
        let result1 =
            expr.evaluate(&data_type, 0, Some(0)).unwrap();
        assert_eq!(result1, vec![100u8]); // 默认值

        let result2 =
            expr.evaluate(&data_type, 4, Some(0)).unwrap();
        assert_eq!(result2, vec![150u8]); // 第5个包，绝对位置匹配

        let result3 = expr
            .evaluate(&data_type, 100, Some(0))
            .unwrap();
        assert_eq!(result3, vec![100u8]); // 相对位置无效，使用默认值

        // 测试没有总包数信息的情况
        let result4 =
            expr.evaluate(&data_type, 4, None).unwrap();
        assert_eq!(result4, vec![150u8]); // 绝对位置仍然有效

        let result5 =
            expr.evaluate(&data_type, 100, None).unwrap();
        assert_eq!(result5, vec![100u8]); // 相对位置无效，使用默认值
    } else {
        panic!("Expected expression");
    }
}
