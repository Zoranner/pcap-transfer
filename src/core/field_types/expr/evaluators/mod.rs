//! 表达式求值器模块
//!
//! 提供各种表达式类型的求值功能

pub mod literal_evaluator;
pub mod loop_evaluator;
pub mod random_evaluator;
pub mod switch_evaluator;

// 重新导出主要的求值函数
pub use literal_evaluator::evaluate_literal;
pub use loop_evaluator::evaluate_loop;
pub use random_evaluator::{
    evaluate_rand_bool, evaluate_rand_float,
    evaluate_rand_hex, evaluate_rand_int,
    evaluate_rand_uint,
};
pub use switch_evaluator::evaluate_switch;
