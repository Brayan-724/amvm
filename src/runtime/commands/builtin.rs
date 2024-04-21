use std::io::Write;

use crate::runtime::{expr, AmvmResult};
use crate::{AmvmScope, CommandExpression, Value, ValueObject};

pub fn eval(scope: &mut AmvmScope, name: &String, args: &Vec<CommandExpression>) -> AmvmResult {
    let mut args_evaluated = vec![];

    for a in args.iter().map(|v| expr::eval(scope, v)) {
        args_evaluated.push(a?);
    }

    match name as &str {
        ".vm.create" => {
            let ctx = scope.create_sub(vec![]);
            let ctx = Box::new(ctx);
            let ctx = Box::into_raw(ctx) as *mut u32;

            scope
                .context
                .read()
                .unwrap()
                .set_prev_value(Value::Object(ValueObject::Native(ctx)));
        }
        ".vm.eval" => {
            let ctx = args_evaluated
                .get(0)
                .expect("Should use `.vm.eval $ctx ...`");
            let ctx = ctx.as_ref().expect("Out buffer should be a variable");
            let ctx = ctx.read();
            let ctx = ctx.as_object().expect("Should be object");
            let ctx = ctx
                .to_native_mutable::<AmvmScope>()
                .expect("Should be object");

            let code = args_evaluated
                .get(1)
                .expect("Should use `.vm.eval $ctx CODE`");
            let code = code.as_value();

            // let Value::U8(ctx) = ctx.deref_mut() else {
            //     return Err(crate::runtime::AmvmPropagate::Err(
            //         crate::runtime::AmvmError::Other("Context should be mutable u8"),
            //     ));
            // };

            let Value::String(code) = code.as_ref() else {
                return Err(crate::runtime::AmvmPropagate::Err(
                    crate::runtime::AmvmError::Other("Code should be String"),
                ));
            };

            let parsed = crate::aml3::from_str(&code)
                .map_err(|e| {
                    eprintln!("Error evaluating:\n{e}");
                    std::process::exit(1);
                })
                .unwrap();
            let p = crate::runtime::scope::eval(ctx, &parsed, true).unwrap();

            scope.context.read().unwrap().set_prev_value(p)
        }

        ".io.stdout.flush" => {
            std::io::stdout().flush().expect("Cannot write to stdout");
        }
        ".io.stdin.read_line" => {
            let out = args_evaluated
                .get(0)
                .expect("Should use `.io.stdin.read_line $out_buffer`");
            let out = out.as_ref().expect("Out buffer should be a variable");

            let mut out_buff = String::new();
            std::io::stdin()
                .read_line(&mut out_buff)
                .expect("Cannot read stdin");

            let out_buff = out_buff.strip_suffix('\n').unwrap_or(&out_buff);

            *out.write().expect("Should be mutable") = Value::String(out_buff.to_owned());
        }
        _ => unimplemented!("{name} {args_evaluated:#?}"),
    }

    Ok(Value::Null)
}
