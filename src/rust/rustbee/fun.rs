use std::collections::HashMap;

type FunCall = fn(Vec<Param>) -> Option<()>;

pub struct fun {
    fun : HashMap<String, FunCall>,
}