use crate::{
    runtime::AmvmResult,
    tokens::{AmvmScope, AmvmType, AmvmTypeDefinition, Value},
};

pub fn eval(scope: &mut AmvmScope, name: &str, body: &Vec<(Box<str>, AmvmType)>) -> AmvmResult {
    let declaration = AmvmTypeDefinition::Struct {
        generics: vec![],
        fields: body.clone(),
    };

    scope
        .context
        .lock()
        .unwrap()
        .structs
        .insert(name.to_string(), declaration);

    Ok(Value::Null)
}
