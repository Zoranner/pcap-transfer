//! 配置管理器测试
//!
//! 测试配置文件的加载、保存和错误处理

use param_sender::app::config::manager::{
    AppConfig, ConfigManager,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_manager_creation() {
    // 测试配置管理器创建
    let result = ConfigManager::new();
    assert!(
        result.is_ok(),
        "Config manager creation should succeed"
    );
}

#[test]
fn test_default_config_loading() {
    // 创建临时目录
    let _temp_dir = TempDir::new().unwrap();

    // 在非标准位置创建配置管理器应该失败，但不会panic
    let result = ConfigManager::new();

    // 即使创建成功，load应该能处理不存在的配置文件
    if let Ok(mut manager) = result {
        let load_result = manager.load();
        assert!(
            load_result.is_ok(),
            "Loading should succeed with default config"
        );
    }
}

#[test]
fn test_config_serialization() {
    // 测试配置序列化
    let config = AppConfig::default();
    let toml_str = toml::to_string_pretty(&config);
    assert!(
        toml_str.is_ok(),
        "Config serialization should work"
    );

    // 测试反序列化
    if let Ok(serialized) = toml_str {
        let deserialized: Result<AppConfig, _> =
            toml::from_str(&serialized);
        assert!(
            deserialized.is_ok(),
            "Config deserialization should work"
        );
    }
}

#[test]
fn test_invalid_config_handling() {
    // 测试无效TOML配置的处理
    let invalid_toml = r#"
        [invalid_section]
        invalid_key = "invalid_value"
        missing_bracket = 
    "#;

    let result: Result<AppConfig, _> =
        toml::from_str(invalid_toml);
    assert!(
        result.is_err(),
        "Invalid TOML should fail to parse"
    );
}

#[tokio::test]
async fn test_config_persistence() {
    // 这个测试需要实际的文件系统操作
    // 在实际项目中，可以使用临时目录进行测试

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // 创建一个简单的配置文件
    let config_content = r#"
[sender]
strategy = "sequential"

[sender.network]
address = "192.168.1.1"
port = 9090
network_type = "multicast"
interface = "eth0"

[[messages]]
name = "test_message"
interval = 2000
enabled = true
packet_count = 0

[[messages.fields]]
name = "test_field"
type = "u32"
editable = true
"#;

    fs::write(&config_path, config_content).unwrap();

    // 尝试解析配置
    let parsed_config: Result<AppConfig, _> =
        toml::from_str(config_content);
    assert!(
        parsed_config.is_ok(),
        "Valid config should parse successfully"
    );

    if let Ok(config) = parsed_config {
        assert_eq!(
            config.sender.network.address,
            "192.168.1.1"
        );
        assert_eq!(config.sender.network.port, 9090);
        assert_eq!(config.messages.len(), 1);
        assert_eq!(config.messages[0].name, "test_message");
        assert_eq!(config.messages[0].fields.len(), 1);
    }
}
