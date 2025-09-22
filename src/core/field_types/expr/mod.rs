//! 字段默认值表达式处理模块
//!
//! 支持的表达式类型：
//! - 字面量：`i32=100`、`hex=0xFF` 等
//! - 随机值：`rand(min,max)` 或 `rand()`（布尔类型）
//! - 循环值：`loop(值1,值2,值3,...)`
//! - 切换值：`switch(默认值, 条件1:值1, 条件2:值2, ...)`

use crate::app::error::types::Result;
use crate::core::field_types::types::FieldDataType;

// 导入子模块
pub mod evaluators;
pub mod parsers;
pub mod types;
pub mod utils;

// 重新导出主要类型和函数
pub use types::DefaultExpr;
pub use utils::parse_field_value_by_type;

use evaluators::*;
use parsers::*;

impl DefaultExpr {
    /// 解析默认表达式字符串
    pub fn parse(
        expr: &str,
        data_type: &FieldDataType,
    ) -> std::result::Result<Self, String> {
        let expr = expr.trim();

        // rand()
        if let Some(inner) = expr.strip_prefix("rand(") {
            let inner =
                inner.strip_suffix(')').ok_or_else(
                    || "rand(...) missing ')'".to_string(),
                )?;
            return parse_random_expr(inner, data_type);
        }

        // loop()
        if let Some(inner) = expr.strip_prefix("loop(") {
            let inner =
                inner.strip_suffix(')').ok_or_else(
                    || "loop(...) missing ')'".to_string(),
                )?;
            return parse_loop_expr(inner);
        }

        // switch()
        if let Some(inner) = expr.strip_prefix("switch(") {
            let inner = inner
                .strip_suffix(')')
                .ok_or_else(|| {
                    "switch(...) missing ')'".to_string()
                })?;
            return parse_switch_expr(inner);
        }

        // 字面量默认值
        Ok(parse_literal(expr))
    }

    /// 求值默认表达式，返回字节数组
    pub fn evaluate(
        &self,
        data_type: &FieldDataType,
        row_index: usize,
        total_packets: Option<u64>,
    ) -> Result<Vec<u8>> {
        match (self, data_type) {
            // 字面量求值
            (DefaultExpr::Literal(s), ty) => {
                evaluate_literal(s, ty)
            }

            // 随机值求值
            (DefaultExpr::RandInt { min, max }, ty) => {
                evaluate_rand_int(*min, *max, ty)
            }
            (DefaultExpr::RandUint { min, max }, ty) => {
                evaluate_rand_uint(*min, *max, ty)
            }
            (DefaultExpr::RandFloat { min, max }, ty) => {
                evaluate_rand_float(*min, *max, ty)
            }
            (DefaultExpr::RandBool, _) => {
                evaluate_rand_bool()
            }
            (
                DefaultExpr::RandHex {
                    min,
                    max,
                    byte_size,
                },
                ty,
            ) => evaluate_rand_hex(
                *min, *max, *byte_size, ty,
            ),

            // 循环值求值
            (DefaultExpr::Loop(items), ty) => {
                evaluate_loop(items, row_index, ty)
            }

            // 切换值求值
            (
                DefaultExpr::Switch {
                    default_value,
                    rules,
                },
                ty,
            ) => evaluate_switch(
                default_value,
                rules,
                row_index,
                total_packets,
                ty,
            ),
        }
    }
}
