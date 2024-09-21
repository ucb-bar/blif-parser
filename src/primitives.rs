use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use strum_macros::EnumCount as EnumCountMacro;
use std::mem::discriminant;

#[derive(Debug, Clone, Default, Serialize, EnumCountMacro)]
#[repr(u32)]
pub enum Primitives {
    #[default]
    NOP = 0,
    Input  { name: String },
    Output { name: String },
    Lut    { inputs: Vec<String>, output: String, table: Vec<Vec<u8>> },
    Gate   { c: String, d: String, q: String, r: Option<String>, e: Option<String> },
    Latch  { input: String, output: String, control: String, init: LatchInit },
    Subckt { name: String, conns: IndexMap<String, String> },
    Module { name: String, inputs: Vec<String>, outputs: Vec<String>, elems: Vec<Primitives> },
}

impl PartialEq for Primitives {
    fn eq(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
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
        assert_eq!(Primitives::NOP == Primitives::Input { name: "hello".to_string() }, false);
        assert_eq!(Primitives::Input { name: "bye".to_string() } == Primitives::Input { name: "hello".to_string() }, true);
    }
}
