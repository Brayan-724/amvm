use crate::tokens::{AmvmType, AmvmTypeDefinition};

pub fn amvm_iterator_type() -> AmvmTypeDefinition {
    #[allow(non_snake_case)]
    let T: Box<AmvmType> = Box::new(AmvmType::Named(Box::from("T")));

    AmvmTypeDefinition::Struct {
        generics: vec![("T".into(), None)],
        fields: vec![(
            "next".into(),
            AmvmType::Fun(vec![AmvmType::Named("Iterator".into())], T),
        )],
    }
}
