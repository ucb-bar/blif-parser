use indexmap::IndexMap;
use std::fmt::Debug;

#[derive(PartialEq, Debug, Clone)]
pub enum Primitives {
    Input  { name: String },
    Output { name: String },
    Lut    { inputs: Vec<String>, output: String, table: Vec<Vec<u8>> },
    Gate   { c: String, d: String, q: String, r: Option<String>, e: Option<String> },
    Latch  { input: String, output: String, control: String, init: LatchInit },
    Subckt { name: String, conns: IndexMap<String, String> },
    Module { name: String, inputs: Vec<String>, outputs: Vec<String>, elems: Vec<Primitives> },
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum LatchInit {
    /// Defined in Yosys spec
    ZER0 = 0,
    ONE = 1,
    DONTCARE = 2,
    UNKNOWN = 3,
}

impl LatchInit {
    pub fn to_enum(i: &str) -> LatchInit {
        match i {
            "0" => LatchInit::ZER0,
            "1" => LatchInit::ONE,
            "2" => LatchInit::DONTCARE,
            _ => LatchInit::UNKNOWN,
        }
    }
}
