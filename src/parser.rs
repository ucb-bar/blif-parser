use crate::primitives::*;
use std::fs;
use indexmap::IndexMap;

type IResultStr<'a> = IResult<&'a str, &'a str>;

use nom::{
    bytes::complete::{is_not, tag, take_until},
    branch::alt,
    combinator::value,
    sequence::{pair, terminated},
    IResult,
};

fn take_until_or_end<'a>(tag: &'a str, istr: &'a str) -> IResultStr<'a> {
    let ret: IResult<&str, &str> = take_until(tag)(istr);
    match ret {
        Ok(x) => Ok(x),
        Err(_) => Ok(("", istr)),
    }
}

fn terminated_newline<'a>(istr: &'a str) -> IResultStr<'a> {
    let ret: IResult<&str, &str> = terminated(take_until("\n"), nom::character::complete::line_ending)(istr);
    match ret {
        Ok(x) => Ok(x),
        Err(_) => Ok(("", istr)),
    }
}

fn lut_table_parser<'a>(input: &'a str, table: &mut Vec<Vec<u8>>) -> IResultStr<'a> {
    let mut i = input;
    let mut li;
    let mut te;
    while i.len() > 0 {
        (i, li) = terminated_newline(i)?;

        let mut row: Vec<u8> = vec![];
        let (_, mut table_input) = take_until(" ")(li)?;
        while table_input.len() > 0 {
            (table_input, te) = nom::character::complete::one_of("01")(table_input)?;
            row.push(te.to_digit(10).unwrap() as u8);
        }
        table.push(row);
    }
    Ok(("", ""))
}

fn lut_body_parser<'a>(input: &'a str, luts: &mut Vec<ParsedPrimitive>) -> IResultStr<'a> {
    let (i, ioline) = terminated_newline(input)?;
    let mut io: Vec<&str> = ioline.split(' ').collect();

    let output = io.pop().unwrap_or("INVALID_OUTPUT").to_string();
    let inputs: Vec<String> = io.iter().map(|v| v.to_string()).collect();
    let (i, table) = take_until_or_end(".", i)?;

    let mut lut_table = vec![];
    let _ = lut_table_parser(table, &mut lut_table);

    luts.push(ParsedPrimitive::Lut {
        inputs: inputs,
        output: output,
        table: lut_table
    });

    Ok((i, ""))
}

fn subckt_parser<'a>(input: &'a str, subckts: &mut Vec<ParsedPrimitive>) -> IResultStr<'a> {
    let (i, sline) = terminated_newline(input)?;
    let conns_vec: Vec<&str> = sline.split(' ').collect();
    let name = conns_vec[0];

    let mut conns = IndexMap::new();
    conns_vec.iter().skip(1).for_each(|c| {
        let lr: Vec<&str> = c.split('=').collect();
        let lhs = lr[0];
        let rhs = lr[1];
        conns.insert(lhs.to_string(), rhs.to_string());
    });

    subckts.push(ParsedPrimitive::Subckt {
        name: name.to_string(),
        conns: conns,
    });

    Ok((i, ""))
}

// _SDFF_NP0_ : FF with reset C D Q R
// _DFFE_PN_  : FF with enables C D E Q
// _SDFFE_PP0N_ : FF with reset and enable C D E Q R
fn gate_parser<'a>(input: &'a str, gates: &mut Vec<ParsedPrimitive>) -> IResultStr<'a> {
    let (i, line) = terminated_newline(input)?;

    let signal_conns: Vec<&str> = line.split(' ').collect();
    let mut c = "".to_string();
    let mut d = "".to_string();
    let mut q = "".to_string();
    let mut r = None;
    let mut e = None;

    for sc in signal_conns.iter() {
        let x: Vec<&str> = sc.split('=').collect();
        if x.len() != 2 {
            continue;
        }
        match x[0] {
            "C" => {
                c = x[1].to_string();
            }
            "D" => {
                d = x[1].to_string();
            }
            "Q" => {
                q = x[1].to_string();
            }
            "R" => {
                r = Some(x[1].to_string());
            }
            "E" => {
                e = Some(x[1].to_string());
            }
            _ => {}
        }
    }

    gates.push(ParsedPrimitive::Gate { c: c, d: d, q: q, r: r, e: e });
    Ok((i, ""))
}

fn latch_parser<'a>(input: &'a str, latches: &mut Vec<ParsedPrimitive>) -> IResultStr<'a> {
    let (i, line) = terminated_newline(input)?;
    let latch_info: Vec<&str> = line.split(' ').collect();

    let mut input = "".to_string();
    let mut output = "".to_string();
    let mut control = "".to_string();
    let mut init = LatchInit::UNKNOWN;

    for (idx, li) in latch_info.iter().enumerate() {
        match idx {
            0 => {
                input = li.to_string();
            }
            1 => {
                output = li.to_string();
            }
            3 => {
                control = li.to_string();
            }
            4 => {
                init = LatchInit::to_enum(li);
            }
            _ => {}
        }
    }
    match init {
        LatchInit::ONE =>
            assert!(false, "Chisel RegInits changes the LUTs not the latch inputs"),
        _ =>
            ()
    }
    latches.push(ParsedPrimitive::Latch { input: input, output: output, control: control, init: init });
    Ok((i, ""))
}

fn module_body_parser<'a>(input: &'a str, modules: &mut Vec<ParsedPrimitive>) -> IResultStr<'a> {
    const MODULE_END_UNIX: &'static str = "\n.end\n";
    const MODULE_END_WINDOWS: &'static str = "\r\n.end\r\n";

    // Get module body
    let (i, _) = tag(".model ")(input)?;
    let (i, name) = terminated(take_until("\n"), nom::character::complete::line_ending)(i)?;
    let (mut i, body) = terminated(
        alt((take_until(MODULE_END_WINDOWS), take_until(MODULE_END_UNIX))),
        nom::character::complete::line_ending,
    )(i)?;

    // Parse inputs
    let (bi, iline) = terminated(take_until("\n"), nom::character::complete::line_ending)(body)?;
    let inputs: Vec<String> = iline.split(' ').map(|v| v.to_string()).skip(1).collect();

    // Parse outputs
    let (bi, oline) = terminated(take_until("\n"), nom::character::complete::line_ending)(bi)?;
    let outputs: Vec<String> = oline.split(' ').map(|v| v.to_string()).skip(1).collect();

    let mut elems = vec![];
    let mut bi = bi;
    let mut tagstr;

    while bi.len() > 1 {
        (bi, tagstr) = terminated(take_until(" "), nom::character::complete::multispace0)(bi)?;
        if tagstr.eq(".names") {
            (bi, _) = lut_body_parser(bi, &mut elems)?;
        } else if tagstr.eq(".subckt") {
            (bi, _) = subckt_parser(bi, &mut elems)?;
        } else if tagstr.eq(".gate") {
            (bi, _) = gate_parser(bi, &mut elems)?;
        } else if tagstr.eq(".latch") {
            (bi, _) = latch_parser(bi, &mut elems)?;
        }
    }

    if i.len() > MODULE_END_UNIX.len() {
        // Advance to the next .end
        (i, _) = take_until(".")(i)?;
    } else {
        // End of file
        (i, _) = take_until("\n")(i)?;
    }

    modules.push(ParsedPrimitive::Module {
        name: name.to_string(),
        inputs: inputs,
        outputs: outputs,
        elems: elems
    });

    Ok((i, ""))
}

fn parse_modules_from_blif_str<'a>(input: &'a str, circuit: &mut Vec<ParsedPrimitive>) -> IResultStr<'a> {
    // remove comments; is_not will stop before \r or \n
    let (i, _) = value((), pair(tag("#"), is_not("\r\n")))(input)?;
    let (i, _) = take_until(".")(i)?;

    let mut i = i;
    while i.len() > 4 {
        (i, _) = module_body_parser(i, circuit)?;
        (i, _) = take_until_or_end("\n", i)?;
        (i, _) = take_until_or_end(".model", i)?;
    }

    Ok(("", ""))
}

pub fn parse_blif(input: &str) -> Result<Vec<ParsedPrimitive>, String> {
    let mut circuit = vec![];
    let res = parse_modules_from_blif_str(input, &mut circuit);
    match res {
        Ok(_) => {
            return Ok(circuit);
        }
        Err(e) => {
            return Err(format!("Error while parsing:\n{}", e).to_string());
        }
    }
}

pub fn parse_blif_file(input_file_path: &str) -> Result<Vec<ParsedPrimitive>, String> {
    let blif_file = fs::read_to_string(input_file_path);
    match blif_file {
        Ok(blif_str) => {
            return parse_blif(&blif_str);
        }
        Err(e) => {
            return Err(format!("Error while reading the file:\n{}", e).to_string());
        }
    }
}

#[cfg(test)]
pub mod parser_tests {
    use super::*;

    pub fn test_blif_parser(file_path: &str) -> bool {
        let res = parse_blif_file(&file_path);
        match res {
            Ok(_) => true,
            Err(err) => {
                println!("blif file parsing error:\n{}", err);
                false
            }
        }
    }

    #[test]
    pub fn test_adder_parse() {
        assert_eq!(test_blif_parser("./tests/Adder.lut.blif"), true);
    }

    #[test]
    pub fn test_adder_two_mod_parse() {
        assert_eq!(test_blif_parser("./tests/AdderTwoMod.blif"), true);
    }

    #[test]
    pub fn test_gcd_parse() {
        assert_eq!(test_blif_parser("./tests/GCD.lut.blif"), true);
    }

    #[test]
    pub fn test_const_parse() {
        assert_eq!(test_blif_parser("./tests/Const.lut.blif"), true);
    }

    #[test]
    pub fn test_digitaltop_parse() {
        assert_eq!(test_blif_parser("./tests/DigitalTop.lut.blif"), true);
    }
}
