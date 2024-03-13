use crate::{AmvmScope, CommandExpression};

use super::AmvmResult;

mod addition;
mod binary_op;
mod cond;
mod property;
mod value;
mod var;

pub fn eval(scope: &mut AmvmScope, expr: &CommandExpression) -> AmvmResult {
    match expr {
        CommandExpression::Value(v) => value::eval(scope, v),
        CommandExpression::Var(v) => var::eval(scope, v),
        CommandExpression::Property(var, property) => property::eval(scope, var, property),

        CommandExpression::Addition(a, b) => addition::eval(scope, a, b),
        CommandExpression::Substraction(a, b) => {
            binary_op::eval(scope, binary_op::BinaryOpKind::Sub, a, b)
        }
        CommandExpression::BinaryCondition(kind, a, b) => cond::eval(scope, kind, a, b),
    }
}
