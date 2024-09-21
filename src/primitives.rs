use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use strum_macros::EnumCount as EnumCountMacro;

#[derive(Debug, Clone, Default, PartialEq, Serialize, EnumCountMacro)]
#[repr(u32)]
pub enum Primitive {
    #[default]
    NOP = 0,
    Input,
    Output,
    Lut,
    Gate,
    Latch,
    Subckt,
    Module,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, EnumCountMacro)]
#[repr(u32)]
pub enum ParsedPrimitive {
    #[default]
    NOP = 0,
    Input  { name: String },
    Output { name: String },
    Lut    { inputs: Vec<String>, output: String, table: Vec<Vec<u8>> },
    Gate   { c: String, d: String, q: String, r: Option<String>, e: Option<String> },
    Latch  { input: String, output: String, control: String, init: LatchInit },
    Subckt { name: String, conns: IndexMap<String, String> },
    Module { name: String, inputs: Vec<String>, outputs: Vec<String>, elems: Vec<ParsedPrimitive> },
}

impl From<ParsedPrimitive> for Primitive {
    fn from(value: ParsedPrimitive) -> Self {
        match value {
            ParsedPrimitive::NOP           => Primitive::NOP,
            ParsedPrimitive::Input  { .. } => Primitive::Input,
            ParsedPrimitive::Output { .. } => Primitive::Output,
            ParsedPrimitive::Lut    { .. } => Primitive::Lut,
            ParsedPrimitive::Gate   { .. } => Primitive::Gate,
            ParsedPrimitive::Latch  { .. } => Primitive::Latch,
            ParsedPrimitive::Subckt { .. } => Primitive::Subckt,
            ParsedPrimitive::Module { .. } => Primitive::Module,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg(test)]
pub mod primitives_test {
    use super::*;

    #[test]
    pub fn test_equality() {
        assert_eq!(Primitive::NOP == Primitive::Input, false);
        assert_eq!(Primitive::Input == Primitive::Input, true);
    }
}
