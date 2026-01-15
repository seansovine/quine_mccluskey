//! Code for converting between various representations of logical functions.
//! The problem is simpler here than in the general case, since we restrict to
//! functions with at most six variables.

use std::error::Error;

use crate::{Minterm, format::string_for_minterm};

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

// We will take a simpler approach here, since we are dealing
// with equations with a very specific form, e.g.:
//  (A & !F) | (B & C & D)
pub fn sop_string_to_init(sop_str: &str) -> String {
    let minterms = sop_to_minterms(sop_str);
    minterms_to_init(&minterms)
}

pub fn init_to_minterms(init_str: &str) -> Result<Vec<Minterm>, Box<dyn Error>> {
    let term_strings = binary_strings_from_init_hex(init_str)?;
    Ok(term_strings.iter().map(|s| (&**s).into()).collect())
}

pub fn sop_to_minterms(sop_str: &str) -> Vec<Minterm> {
    let products = sop_str.trim().split('|');
    let mut minterms = vec![];
    for product in products {
        let minterm = parse_product(product.trim());
        if DEV_DEBUG {
            println!(
                "Product term {product} was parsed as term {}.",
                string_for_minterm(&minterm)
            );
        }
        minterms.push(minterm);
    }
    minterms
}

const ALLOWED_VARS: &[char] = &['A', 'B', 'C', 'D', 'E', 'F', 'G'];

// Allows either `!` or '~' for negation.
fn parse_product(prod_str: &str) -> Minterm {
    let mut minterm = Minterm {
        values: vec![b'x', b'x', b'x', b'x', b'x', b'x'],
    };
    let mut prod_ref = prod_str;
    if !prod_str.starts_with('(') {
        assert!(prod_str.len() == 1 && ALLOWED_VARS.contains(&prod_str.chars().next().unwrap()));
    } else {
        // We require nontrivial product enclosed in parentheses.
        assert!(prod_str.starts_with('(') && prod_str.chars().nth_back(0) == Some(')'));
        prod_ref = &prod_str.trim()[1..prod_str.len() - 1];
    }
    let terms = prod_ref.split('&');
    for term in terms {
        let term = term.trim();
        if term.starts_with('!') || term.starts_with('~') {
            let var = term
                .chars()
                .nth(1)
                .expect("Expected variable to follow ! symbol.");
            let var_i = char_to_index(var);
            minterm.values[var_i] = b'0';
        } else {
            let var = term.chars().next().unwrap();
            let var_i = char_to_index(var);
            minterm.values[var_i] = b'1';
        }
    }
    minterm
}

fn char_to_index(ch: char) -> usize {
    match ch {
        'A' => 5,
        'B' => 4,
        'C' => 3,
        'D' => 2,
        'E' => 1,
        'F' => 0,
        _ => panic!("Unexpected variable character: {ch}."),
    }
}

pub fn minterms_to_init(minterms: &[Minterm]) -> String {
    let mut init_num: u64 = 0;
    for minterm in minterms {
        let init_terms = minterm_to_init_terms(minterm);
        for term in init_terms {
            let term_num = u8::from_str_radix(&term, 2).unwrap();
            init_num |= 2_u64.pow(term_num as u32);
        }
    }
    format!("{init_num:016X}")
}

fn minterm_to_init_terms(minterm: &Minterm) -> Vec<String> {
    let mut init_terms = vec![String::new()];
    for value in minterm.values.iter() {
        match value {
            b'0' => {
                for term in init_terms.iter_mut() {
                    *term += "0";
                }
            }
            b'1' => {
                for term in init_terms.iter_mut() {
                    *term += "1";
                }
            }
            b'x' => {
                let mut one_terms = init_terms.clone();
                one_terms.iter_mut().for_each(|term| *term += "1");
                for term in init_terms.iter_mut() {
                    *term += "0";
                }
                init_terms.extend(one_terms);
            }
            _ => unreachable!(),
        }
    }
    init_terms
}
