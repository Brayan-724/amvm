use crate::runtime::AmvmResult;
use crate::tokens::AmvmType;
use crate::{AmvmScope, AmvmTypeDefinition, Value};

pub fn eval(
    scope: &mut AmvmScope,
    name: &Box<str>,
    body: &Vec<(Box<str>, AmvmType)>,
) -> AmvmResult {
    let declaration = AmvmTypeDefinition::Struct(body.clone());

    scope
        .context
        .write()
        .unwrap()
        .structs
        .insert(name.to_string(), declaration);

    Ok(Value::Null)
}
