//! 字段值解析工具函数

/// 解析十六进制字符串
pub fn parse_hex_string(
    hex_str: &str,
) -> Result<Vec<u8>, String> {
    let hex_str = hex_str.trim();

    // 移除0x前缀
    let hex_str = if hex_str.starts_with("0x")
        || hex_str.starts_with("0X")
    {
        &hex_str[2..]
    } else {
        hex_str
    };

    if hex_str.is_empty() {
        return Ok(vec![0]);
    }

    // 确保长度为偶数
    let hex_str = if hex_str.len() % 2 == 1 {
        format!("0{}", hex_str)
    } else {
        hex_str.to_string()
    };

    let mut bytes = Vec::new();
    for chunk in hex_str.as_bytes().chunks(2) {
        let hex_pair = std::str::from_utf8(chunk)
            .map_err(|_| "Invalid UTF-8 in hex string")?;
        let byte = u8::from_str_radix(hex_pair, 16)
            .map_err(|_| {
                format!("Invalid hex value: {}", hex_pair)
            })?;
        bytes.push(byte);
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_string() {
        assert_eq!(
            parse_hex_string("0xFF").unwrap(),
            vec![0xFF]
        );
        assert_eq!(
            parse_hex_string("FF").unwrap(),
            vec![0xFF]
        );
        assert_eq!(
            parse_hex_string("1234").unwrap(),
            vec![0x12, 0x34]
        );
        assert_eq!(
            parse_hex_string("F").unwrap(),
            vec![0x0F]
        );
    }
}
