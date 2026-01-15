//! Code to format and display logical expressions.

use crate::Minterm;

// ----------------------------
// String formatting functions.

// Character for negation in formatted outupt.
const NEG_CHAR: char = '~';

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
