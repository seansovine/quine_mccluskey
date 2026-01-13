//! Code for checking logical equivalence of equations.
//! The problem is simpler here, since we restrict to
//! at most six variables.

use crate::{Minterm, string_for_minterm};

const DEV_DEBUG: bool = false;

// We will take a simpler approach here, since we are dealing
// with equations with a very specific form, e.g.:
//  (A & !F) | (B & C & D)
pub fn sop_string_to_init(sop_str: &str) -> String {
    let minterms = sop_to_minterms(sop_str);
    minterms_to_init(&minterms)
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

// Allows either `!` or '~' for negation.
fn parse_product(prod_str: &str) -> Minterm {
    let mut minterm = Minterm {
        values: vec![b'x', b'x', b'x', b'x', b'x', b'x'],
    };
    assert!(prod_str.starts_with('(') && prod_str.chars().nth_back(0) == Some(')'));
    // We require product terms enclosed in parentheses.
    let prod_str = &prod_str.trim()[1..prod_str.len() - 1];
    let terms = prod_str.split('&');
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
    for (i, value) in minterm.values.iter().enumerate() {
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
