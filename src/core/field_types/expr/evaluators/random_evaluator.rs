//! 随机表达式求值器

use crate::app::error::types::{AppError, Result};
use crate::core::field_types::types::FieldDataType;
use rand::Rng;

/// 求值随机整数表达式
pub fn evaluate_rand_int(
    min: i128,
    max: i128,
    data_type: &FieldDataType,
) -> Result<Vec<u8>> {
    let mut rng = rand::thread_rng();
    match data_type {
        FieldDataType::I8 => {
            let v = rng
                .gen_range((min as i64)..=(max as i64))
                as i8;
            Ok(v.to_le_bytes().to_vec())
        }
        FieldDataType::I16 => {
            let v = rng
                .gen_range((min as i64)..=(max as i64))
                as i16;
            Ok(v.to_le_bytes().to_vec())
        }
        FieldDataType::I32 => {
            let v = rng
                .gen_range((min as i64)..=(max as i64))
                as i32;
            Ok(v.to_le_bytes().to_vec())
        }
        FieldDataType::I64 => {
            let v = rng.gen_range(min..=max) as i64;
            Ok(v.to_le_bytes().to_vec())
        }
        _ => Err(AppError::validation(
            "rand_int",
            "Invalid data type for random integer",
        )),
    }
}

/// 求值随机无符号整数表达式
pub fn evaluate_rand_uint(
    min: u128,
    max: u128,
    data_type: &FieldDataType,
) -> Result<Vec<u8>> {
    let mut rng = rand::thread_rng();
    match data_type {
        FieldDataType::U8 => {
            let v = rng
                .gen_range((min as u64)..=(max as u64))
                as u8;
            Ok(v.to_le_bytes().to_vec())
        }
        FieldDataType::U16 => {
            let v = rng
                .gen_range((min as u64)..=(max as u64))
                as u16;
            Ok(v.to_le_bytes().to_vec())
        }
        FieldDataType::U32 => {
            let v = rng
                .gen_range((min as u64)..=(max as u64))
                as u32;
            Ok(v.to_le_bytes().to_vec())
        }
        FieldDataType::U64 => {
            let v = rng.gen_range(min..=max) as u64;
            Ok(v.to_le_bytes().to_vec())
        }
        _ => Err(AppError::validation(
            "rand_uint",
            "Invalid data type for random unsigned integer",
        )),
    }
}

/// 求值随机浮点表达式
pub fn evaluate_rand_float(
    min: f64,
    max: f64,
    data_type: &FieldDataType,
) -> Result<Vec<u8>> {
    let mut rng = rand::thread_rng();
    match data_type {
        FieldDataType::F32 => {
            let v = rng.gen_range(min..=max) as f32;
            Ok(v.to_le_bytes().to_vec())
        }
        FieldDataType::F64 => {
            let v = rng.gen_range(min..=max);
            Ok(v.to_le_bytes().to_vec())
        }
        _ => Err(AppError::validation(
            "rand_float",
            "Invalid data type for random float",
        )),
    }
}

/// 求值随机布尔表达式
pub fn evaluate_rand_bool() -> Result<Vec<u8>> {
    let mut rng = rand::thread_rng();
    let v: bool = rng.gen();
    Ok(vec![if v { 1 } else { 0 }])
}

/// 求值随机十六进制表达式
pub fn evaluate_rand_hex(
    min: u128,
    max: u128,
    byte_size: usize,
    data_type: &FieldDataType,
) -> Result<Vec<u8>> {
    let mut rng = rand::thread_rng();
    let v = if min == max {
        min
    } else {
        rng.gen_range(min..=max)
    };

    if let FieldDataType::HexDynamic(size) = data_type {
        if byte_size != *size {
            return Err(AppError::validation(
                "hex",
                "hex rand byte_size mismatch with type",
            ));
        }
        let mut bytes = vec![0u8; *size];
        for (i, byte) in
            bytes.iter_mut().enumerate().take(*size)
        {
            *byte = ((v >> (8 * i)) & 0xFF) as u8;
        }
        Ok(bytes)
    } else {
        Err(AppError::validation(
            "rand_hex",
            "Invalid data type for random hex",
        ))
    }
}
