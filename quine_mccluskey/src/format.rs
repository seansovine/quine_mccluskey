// Character for negation in formatted outupt.

use std::error::Error;

use crate::Minterm;

const NEG_CHAR: char = '~';

// ---------------------
// Conversion functions.

const DEV_DEBUG: bool = false;

/// Convert a hex "init" string to a list of binary term strings.
pub fn binary_strings_from_init_hex(hex_str: &str) -> Result<Vec<String>, Box<dyn Error>> {
    const HEX_LEN: usize = 16;
    if hex_str.len() > HEX_LEN {
        return Err(String::from("Hex string contains more than 16 hex chars.").into());
    }
    let zero_pad: String = std::iter::repeat_n('0', HEX_LEN - hex_str.len()).collect();
    let hex_str = format!("{zero_pad}{hex_str}");
    let num: u64 =
        u64::from_str_radix(&hex_str, 16).expect("String is not a valid 64-bit hex string.");

    if DEV_DEBUG {
        println!("As binary: {num:064b}");
    }
    let mut strings = vec![];
    for i in 0..64 {
        let mask: u64 = 1 << i;
        if mask & num > 0 {
            if DEV_DEBUG {
                println!("Term {i:02}: {i:06b}");
            }
            strings.push(format!("{i:06b}"));
        }
    }
    Ok(strings)
}

// Sort minterms nicely for canonical display.

pub fn display_sort_minterms(minterms: &mut [Minterm]) {
    if minterms.is_empty() {
        return;
    }
    assert!(minterms.first().unwrap().values.len() == 6);
    minterms.sort_by_key(|m| {
        let mut tuple = [2_u8; 6];
        for (i, val) in m.values.iter().rev().enumerate() {
            match val {
                b'1' => tuple[i] = 0,
                b'0' => tuple[i] = 1,
                b'x' => tuple[i] = 2,
                _ => unreachable!(),
            }
        }
        tuple
    });
}

// ----------------------------
// String formatting functions.

const EQN_VARS: &[char] = &['A', 'B', 'C', 'D', 'E', 'F'];

pub fn string_for_minterm(minterm: &Minterm) -> String {
    let mut term_string = String::new();
    for (i, c) in minterm.values.iter().rev().enumerate() {
        let var = match c {
            b'x' => continue,
            b'0' => format!("{NEG_CHAR}{}", EQN_VARS[i]),
            b'1' => format!("{}", EQN_VARS[i]),
            _ => unreachable!(),
        };
        if term_string.is_empty() {
            term_string = var.to_string();
        } else {
            term_string = format!("{term_string} & {var}");
        }
    }
    if term_string.is_empty() {
        "True".into()
    } else {
        term_string
    }
}

/// Get a string representation for the SOP with minterm set `minterms`.
pub fn string_for_sop_minterms(
    minterms: &[Minterm],
    omit_trivial: bool,
    separator: Option<&str>,
) -> String {
    if minterms.is_empty() {
        return "False".into();
    }

    let separator = separator.unwrap_or(" ");
    let mut expr_string = String::new();
    for minterm in minterms.iter() {
        let term_string = string_for_minterm(minterm);
        if term_string == "True" && omit_trivial {
            continue;
        }
        if expr_string.is_empty() {
            expr_string = format!("({term_string})");
        } else {
            expr_string = format!("{expr_string}{separator}| ({term_string})");
        }
    }

    if expr_string.is_empty() {
        "True".into()
    } else {
        expr_string
    }
}
