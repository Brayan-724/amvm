use std::io::Write;
use std::sync::Arc;

use expr::AmvmExprResult;
use variable::AmvmVariable;

use crate::runtime::variable;
use crate::tokens::VariableKind;
use crate::{
    runtime::{expr, AmvmPropagate, AmvmResult},
    tokens::{AmvmScope, CommandExpression, Value, ValueObject},
};

pub fn eval(scope: &mut AmvmScope, name: &str, args: &[CommandExpression]) -> AmvmResult {
    let mut args_evaluated = Vec::with_capacity(args.len());

    for arg in args {
        args_evaluated.push(expr::eval(scope, arg)?);
    }

    let result = call(scope, name, &args_evaluated)?;

    if let Some(result) = result {
        scope.context.lock().unwrap().push_prev(result);
    }

    Ok(Value::Null)
}

pub fn call(
    scope: &mut AmvmScope,
    name: &str,
    args: &[AmvmExprResult],
) -> Result<Option<AmvmExprResult>, AmvmPropagate> {
    match name {
        ".vm.create" => {
            let ctx = scope.create_sub(vec![]);
            let ctx = Box::new(ctx);
            let ctx = Box::into_raw(ctx) as *mut u32;

            return Ok(Some(Value::Object(ValueObject::Native(ctx)).into()));
        }
        ".vm.eval" => {
            let mut args_evaluated = args.iter();
            let ctx = args_evaluated
                .next()
                .expect("Should use `.vm.eval $ctx ...`");
            let ctx = ctx.as_var().expect("Out buffer should be a variable");
            let ctx = ctx.read();
            let ctx = ctx.as_object().expect("Should be object");
            let Some(ctx) = ctx.to_native_mutable::<AmvmScope>() else {
                drop(args_evaluated);
                return Err(AmvmPropagate::Err(scope.error("Should be object")));
            };

            let code = args_evaluated
                .next()
                .expect("Should use `.vm.eval $ctx CODE`");
            let code = code.as_value();

            let Value::String(code) = code.as_ref() else {
                drop(args_evaluated);
                return Err(crate::runtime::AmvmPropagate::Err(
                    scope.error("Code should be String"),
                ));
            };

            let parsed = crate::aml3::from_str(code)
                .map_err(|e| {
                    eprintln!("Error evaluating:\n{e}");
                    std::process::exit(1);
                })
                .unwrap();
            let p = crate::runtime::scope::eval(ctx, &parsed, true).unwrap();

            return Ok(Some(p.into()));
        }

        // IO //
        ".io.stdout.flush" => {
            std::io::stdout().flush().expect("Cannot write to stdout");
        }
        ".io.stdin.read_line" => {
            let mut args_evaluated = args.iter();
            let out = args_evaluated
                .next()
                .expect("Should use `.io.stdin.read_line $out_buffer`");
            let out = out.as_var().expect("Out buffer should be a variable");

            let mut out_buff = String::new();
            std::io::stdin()
                .read_line(&mut out_buff)
                .expect("Cannot read stdin");

            let out_buff = out_buff.strip_suffix('\n').unwrap_or(&out_buff);

            *out.write().expect("Should be mutable") = Value::String(out_buff.to_owned());
        }

        // OBJ //
        ".obj.mut_access" => {
            let mut args = args.iter();
            let variable = args.next().expect("Already checked");
            let field = args.next().expect("Already");
            let field = field.as_value();
            let variable = variable.as_ref();

            if !variable.is_mutable() {
                drop(args);
                return Err(AmvmPropagate::Err(
                    scope.error("Cannot borrow inmutable to mutable"),
                ));
            }

            let mut variable = variable.write().expect("Checked above");

            let Value::Object(variable) = &mut *variable else {
                drop(args);
                return Err(AmvmPropagate::Err(
                    scope.error("Cannot access to fields in non-object values"),
                ));
            };

            let res = match variable {
                ValueObject::Native(_) => todo!("Can't get properties of native object"),
                ValueObject::Instance(_, map) | ValueObject::PropertyMap(map) => match &*field {
                    Value::String(name) => {
                        let Some(v) = map.get(name) else {
                            drop(args);
                            return Err(AmvmPropagate::Err(scope.error("Property not found")));
                        };

                        AmvmExprResult::Variable(AmvmVariable::from_rw(
                            VariableKind::Mut,
                            Arc::clone(v),
                        ))
                    }
                    _ => {
                        drop(args);
                        return Err(AmvmPropagate::Err(
                            scope.error("Objects only can be accessed by a string"),
                        ));
                    }
                },
            };

            return Ok(Some(res));
        }

        // MEM //
        ".mem.replace" => {
            let mut args = args.iter();
            let variable = args.next().expect("Already checked");
            let value = args.next().expect("Already checked");
            let value = &*value.as_value();

            let variable = variable.as_ref();
            let mut variable = variable.write().expect("Should be mutable"); // TODO: Error propagation
            *variable = value.clone();
        }

        _ => unimplemented!("{name}"),
    }

    Ok(None)
}
