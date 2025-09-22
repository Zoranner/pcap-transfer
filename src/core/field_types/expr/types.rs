//! 表达式类型定义
//!
//! 定义所有表达式相关的数据结构

/// 切换条件类型
#[derive(Debug, Clone, PartialEq)]
pub enum SwitchCondition {
    /// 绝对位置（从1开始计数）
    Absolute(usize),
    /// 相对位置（负数，从-1开始，表示倒数第几个）
    Relative(i32),
    /// 范围（start-end，包含边界，从1开始计数）
    Range(usize, usize),
}

/// 切换规则
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchRule {
    /// 切换条件
    pub condition: SwitchCondition,
    /// 对应的值
    pub value: String,
}

/// 默认值表达式枚举
#[derive(Debug, Clone, PartialEq)]
pub enum DefaultExpr {
    /// 直接字面量（按列的数据类型进行解析）
    Literal(String),
    /// 随机整数（根据具体的数据类型边界进行校验与截断）
    RandInt {
        /// 最小值
        min: i128,
        /// 最大值
        max: i128,
    },
    /// 随机无符号整数
    RandUint {
        /// 最小值
        min: u128,
        /// 最大值
        max: u128,
    },
    /// 随机浮点
    RandFloat {
        /// 最小值
        min: f64,
        /// 最大值
        max: f64,
    },
    /// 随机布尔
    RandBool,
    /// 随机十六进制，按 byte_size 生成小端序字节
    RandHex {
        /// 最小值
        min: u128,
        /// 最大值
        max: u128,
        /// 字节大小
        byte_size: usize,
    },
    /// 循环取值（按行索引轮询），值以原始字符串形式存储
    Loop(Vec<String>),
    /// 条件切换值（根据包索引和总包数切换不同的值）
    Switch {
        /// 默认值
        default_value: String,
        /// 切换规则列表
        rules: Vec<SwitchRule>,
    },
}
