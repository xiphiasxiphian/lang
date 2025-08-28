use std::collections::HashMap;

use crate::frontend::parsers::types::Type;

const SPECIAL_CHAR: char = '$';
pub type UniqueId = String;

pub fn gen_id(id: String, num: usize) -> UniqueId
{
    format!("{id}{SPECIAL_CHAR}{num}")
}
