#![allow(clippy::upper_case_acronyms)]

#[cfg(test)]
mod test;

use hex;
use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use AddrMode::*;
use Instruction::*;

pub fn assemble(src: &str) -> Result<Vec<u8>, String> {
    let mut bin: Vec<u8> = vec![];

    for (line_num, line) in src.lines().enumerate() {
        match assemble_line(line) {
            Ok(mut bytes) => bin.append(&mut bytes),
            Err(err_str) => return Err(format!("Error at line {}: {}", line_num, err_str)),
        }
    }

    Ok(bin)
}

pub fn disassemble(bin: &[u8]) -> Result<String, String> {
    let mut disassembled = String::new();
    let mut num_bytes_processed = 0;

    while num_bytes_processed < bin.len() {
        match disassemble_ins(&bin[num_bytes_processed..]) {
            Ok((ins_str, ins_num_bytes)) => {
                disassembled = disassembled + &ins_str + "\n";
                num_bytes_processed += ins_num_bytes
            }
            Err(err_str) => {
                return Err(format!(
                    "Error at byte {}: {}",
                    num_bytes_processed, err_str
                ))
            }
        }
    }

    Ok(String::from(disassembled.trim_end()))
}

fn assemble_line(line: &str) -> Result<Vec<u8>, String> {
    let tokens: Vec<&str> = line.split(";").next().unwrap().split_whitespace().collect();

    if tokens.is_empty() {
        return Ok(vec![]);
    }

    if tokens.len() > 2 {
        return Err(String::from("Too many tokens in line."));
    }

    let ins = match Instruction::from_str(tokens[0]) {
        Ok(val) => val,
        Err(_) => return Err(format!("Invalid instruction: {}.", tokens[0])),
    };

    let addr_mode = if tokens.len() == 1 {
        IMP
    } else if is_branch_instruction(&ins) {
        REL
    } else {
        get_addr_mode(tokens[1])?
    };

    let mut result = vec![get_opcode(&ins, &addr_mode)?];

    if addr_mode != IMP {
        result.extend(get_args(tokens[1])?);
    }

    Ok(result)
}

fn is_branch_instruction(ins: &Instruction) -> bool {
    matches!(ins, BCC | BCS | BEQ | BMI | BNE | BPL | BVC | BVS)
}

fn get_addr_mode(args: &str) -> Result<AddrMode, String> {
    for (regex, addr_mode) in ARG_REGEXES.iter() {
        if regex.is_match(args) {
            return Ok(addr_mode.clone());
        }
    }

    Err(String::from("Invalid argument."))
}

fn get_opcode(ins: &Instruction, addr_mode: &AddrMode) -> Result<u8, String> {
    for (opcode, search_ins, search_addr_mode) in OPCODES.iter() {
        if ins == search_ins && addr_mode == search_addr_mode {
            return Ok(*opcode);
        }
    }

    Err(format!("Invalid addressing mode for instruction {}.", ins))
}

fn get_args(args: &str) -> Result<Vec<u8>, String> {
    let mut bytes: Vec<u8> = vec![];

    for byte_match in ARG_VAL_REGEX.find_iter(args) {
        bytes.extend(hex::decode(byte_match.as_str()).unwrap());
    }

    bytes.reverse();
    Ok(bytes)
}

fn disassemble_ins(bytes: &[u8]) -> Result<(String, usize), String> {
    if bytes.is_empty() {
        return Ok((String::new(), 0));
    }

    let (ins, addr_mode) = get_ins_info(bytes[0])?;
    let (fmt_string, num_arg_bytes) = get_disassemble_info(&ins, &addr_mode);

    if bytes.len() < num_arg_bytes + 1 {
        return Err(String::from("Not enough arguments."));
    }

    let arg_str = match num_arg_bytes {
        0 => fmt_string,
        1 => fmt_string.replace("@", &format!("{:02X}", bytes[1])),
        2 => fmt_string.replace("@", &format!("{:02X}{:02X}", bytes[2], bytes[1])),
        _ => panic!("Disassemble error."),
    };

    let ins_str = ins.to_string() + " " + &arg_str;

    Ok((String::from(ins_str.trim_end()), num_arg_bytes + 1))
}

fn get_ins_info(opcode: u8) -> Result<(Instruction, AddrMode), String> {
    for (search_opcode, ins, addr_mode) in OPCODES.iter() {
        if opcode == *search_opcode {
            return Ok((ins.clone(), addr_mode.clone()));
        }
    }

    Err(String::from("Invalid opcode."))
}

fn is_shift_instruction(ins: &Instruction) -> bool {
    matches!(ins, ASL | LSR | ROL | ROR)
}

fn get_disassemble_info(ins: &Instruction, addr_mode: &AddrMode) -> (String, usize) {
    for (search_addr_mode, fmt_string, num_arg_bytes) in DISASSEMBLE_INFO.iter() {
        if addr_mode == search_addr_mode {
            return (String::from(*fmt_string), *num_arg_bytes);
        }
    }

    if is_shift_instruction(ins) {
        return (String::from("A"), 0);
    }

    (String::from(""), 0)
}

#[derive(Display, EnumString, Clone, PartialEq)]
enum Instruction {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
}

#[derive(Display, EnumString, Clone, PartialEq)]
enum AddrMode {
    IMP,
    IMM,
    ZP,
    ZPX,
    ZPY,
    REL,
    ABS,
    ABSX,
    ABSY,
    IND,
    INDX,
    INDY,
}

static OPCODES: [(u8, Instruction, AddrMode); 151] = [
    (0x69, ADC, IMM),
    (0x65, ADC, ZP),
    (0x75, ADC, ZPX),
    (0x6D, ADC, ABS),
    (0x7D, ADC, ABSX),
    (0x79, ADC, ABSY),
    (0x61, ADC, INDX),
    (0x71, ADC, INDY),
    (0x29, AND, IMM),
    (0x25, AND, ZP),
    (0x35, AND, ZPX),
    (0x2D, AND, ABS),
    (0x3D, AND, ABSX),
    (0x39, AND, ABSY),
    (0x21, AND, INDX),
    (0x31, AND, INDY),
    (0x0A, ASL, IMP),
    (0x06, ASL, ZP),
    (0x16, ASL, ZPX),
    (0x0E, ASL, ABS),
    (0x1E, ASL, ABSX),
    (0x90, BCC, REL),
    (0xB0, BCS, REL),
    (0xF0, BEQ, REL),
    (0x24, BIT, ZP),
    (0x2C, BIT, ABS),
    (0x30, BMI, REL),
    (0xD0, BNE, REL),
    (0x10, BPL, REL),
    (0x00, BRK, IMP),
    (0x50, BVC, REL),
    (0x70, BVS, REL),
    (0x18, CLC, IMP),
    (0xD8, CLD, IMP),
    (0x58, CLI, IMP),
    (0xB8, CLV, IMP),
    (0xC9, CMP, IMM),
    (0xC5, CMP, ZP),
    (0xD5, CMP, ZPX),
    (0xCD, CMP, ABS),
    (0xDD, CMP, ABSX),
    (0xD9, CMP, ABSY),
    (0xC1, CMP, INDX),
    (0xD1, CMP, INDY),
    (0xE0, CPX, IMM),
    (0xE4, CPX, ZP),
    (0xEC, CPX, ABS),
    (0xC0, CPY, IMM),
    (0xC4, CPY, ZP),
    (0xCC, CPY, ABS),
    (0xC6, DEC, ZP),
    (0xD6, DEC, ZPX),
    (0xCE, DEC, ABS),
    (0xDE, DEC, ABSX),
    (0xCA, DEX, IMP),
    (0x88, DEY, IMP),
    (0x49, EOR, IMM),
    (0x45, EOR, ZP),
    (0x55, EOR, ZPX),
    (0x4D, EOR, ABS),
    (0x5D, EOR, ABSX),
    (0x59, EOR, ABSY),
    (0x41, EOR, INDX),
    (0x51, EOR, INDY),
    (0xE6, INC, ZP),
    (0xF6, INC, ZPX),
    (0xEE, INC, ABS),
    (0xFE, INC, ABSX),
    (0xE8, INX, IMP),
    (0xC8, INY, IMP),
    (0x4C, JMP, ABS),
    (0x6C, JMP, IND),
    (0x20, JSR, ABS),
    (0xA9, LDA, IMM),
    (0xA5, LDA, ZP),
    (0xB5, LDA, ZPX),
    (0xAD, LDA, ABS),
    (0xBD, LDA, ABSX),
    (0xB9, LDA, ABSY),
    (0xA1, LDA, INDX),
    (0xB1, LDA, INDY),
    (0xA2, LDX, IMM),
    (0xA6, LDX, ZP),
    (0xB6, LDX, ZPY),
    (0xAE, LDX, ABS),
    (0xBE, LDX, ABSY),
    (0xA0, LDY, IMM),
    (0xA4, LDY, ZP),
    (0xB4, LDY, ZPX),
    (0xAC, LDY, ABS),
    (0xBC, LDY, ABSX),
    (0x4A, LSR, IMP),
    (0x46, LSR, ZP),
    (0x56, LSR, ZPX),
    (0x4E, LSR, ABS),
    (0x5E, LSR, ABSX),
    (0xEA, NOP, IMP),
    (0x09, ORA, IMM),
    (0x05, ORA, ZP),
    (0x15, ORA, ZPX),
    (0x0D, ORA, ABS),
    (0x1D, ORA, ABSX),
    (0x19, ORA, ABSY),
    (0x01, ORA, INDX),
    (0x11, ORA, INDY),
    (0x48, PHA, IMP),
    (0x08, PHP, IMP),
    (0x68, PLA, IMP),
    (0x28, PLP, IMP),
    (0x2A, ROL, IMP),
    (0x26, ROL, ZP),
    (0x36, ROL, ZPX),
    (0x2E, ROL, ABS),
    (0x3E, ROL, ABSX),
    (0x6A, ROR, IMP),
    (0x66, ROR, ZP),
    (0x76, ROR, ZPX),
    (0x6E, ROR, ABS),
    (0x7E, ROR, ABSX),
    (0x40, RTI, IMP),
    (0x60, RTS, IMP),
    (0xE9, SBC, IMM),
    (0xE5, SBC, ZP),
    (0xF5, SBC, ZPX),
    (0xED, SBC, ABS),
    (0xFD, SBC, ABSX),
    (0xF9, SBC, ABSY),
    (0xE1, SBC, INDX),
    (0xF1, SBC, INDY),
    (0x38, SEC, IMP),
    (0xF8, SED, IMP),
    (0x78, SEI, IMP),
    (0x85, STA, ZP),
    (0x95, STA, ZPX),
    (0x8D, STA, ABS),
    (0x9D, STA, ABSX),
    (0x99, STA, ABSY),
    (0x81, STA, INDX),
    (0x91, STA, INDY),
    (0x86, STX, ZP),
    (0x96, STX, ZPY),
    (0x8E, STX, ABS),
    (0x84, STY, ZP),
    (0x94, STY, ZPX),
    (0x8C, STY, ABS),
    (0xAA, TAX, IMP),
    (0xA8, TAY, IMP),
    (0xBA, TSX, IMP),
    (0x8A, TXA, IMP),
    (0x9A, TXS, IMP),
    (0x98, TYA, IMP),
];

lazy_static! {
    static ref ARG_REGEXES: [(Regex, AddrMode); 11] = [
        (Regex::new("^A$").unwrap(), IMP),
        (Regex::new(r"^\$[0-9a-fA-F]{4}$").unwrap(), ABS),
        (Regex::new(r"^\$[0-9a-fA-F]{2}$").unwrap(), ZP),
        (Regex::new(r"^\$[0-9a-fA-F]{2},X$").unwrap(), ZPX),
        (Regex::new(r"^\$[0-9a-fA-F]{2},Y$").unwrap(), ZPY),
        (Regex::new(r"^\$[0-9a-fA-F]{4},X$").unwrap(), ABSX),
        (Regex::new(r"^\$[0-9a-fA-F]{4},Y$").unwrap(), ABSY),
        (Regex::new(r"^#\$[0-9a-fA-F]{2}$").unwrap(), IMM),
        (Regex::new(r"^\(\$[0-9a-fA-F]{4}\)$").unwrap(), IND),
        (Regex::new(r"^\(\$[0-9a-fA-F]{2},X\)$").unwrap(), INDX),
        (Regex::new(r"^\(\$[0-9a-fA-F]{2}\),Y$").unwrap(), INDY),
    ];
    static ref ARG_VAL_REGEX: Regex = Regex::new(r"[0-9a-fA-F]{2}").unwrap();
}

static DISASSEMBLE_INFO: [(AddrMode, &str, usize); 11] = [
    (ABS, "$@", 2),
    (ZP, "$@", 1),
    (ZPX, "$@,X", 1),
    (ZPY, "$@,Y", 1),
    (ABSX, "$@,X", 2),
    (ABSY, "$@,Y", 2),
    (IMM, "#$@", 1),
    (IND, "($@)", 2),
    (INDX, "($@,X)", 1),
    (INDY, "($@),Y", 1),
    (REL, "$@", 1),
];
