use reflex::core::{uuid, ArgType, Arity, FunctionArity, Uid, Uuid};

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct ResolveLoaderResults;
impl ResolveLoaderResults {
    pub const UUID: Uuid = uuid!("dd4517b0-722d-431f-89fd-612c76f7e694");
    const ARITY: FunctionArity<2, 0> = FunctionArity {
        required: [ArgType::Strict, ArgType::Strict],
        optional: [],
        variadic: None,
    };
    pub fn arity(&self) -> Arity {
        Arity::from(&Self::ARITY)
    }
}
impl Uid for ResolveLoaderResults {
    fn uid(&self) -> Uuid {
        Self::UUID
    }
}
