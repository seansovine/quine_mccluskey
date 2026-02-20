//! Code to format and display logical expressions.

use crate::Minterm;

// ----------------------------
// String formatting functions.

// Character for negation in formatted output.
const NEG_CHAR: char = '~';

// Sort minterms nicely for canonical display.

pub fn display_sort_minterms(minterms: &mut [Minterm]) {
    if minterms.is_empty() {
        return;
    }
    assert!(minterms.first().unwrap().values.len() <= 6);
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

pub struct FormattedExpr {
    pub minterm_expr: String,
    pub dont_cares: String,
}

impl FormattedExpr {
    pub fn sop_string(&self) -> &str {
        &self.minterm_expr
    }
}

/// Get a string representation for the SOP with minterm set `minterms`.
pub fn string_for_sop_minterms(
    minterms: &[Minterm],
    omit_trivial: bool,
    separator: Option<&str>,
) -> FormattedExpr {
    let mut dont_cares = String::new();
    if minterms.is_empty() {
        return FormattedExpr {
            minterm_expr: "False".into(),
            dont_cares,
        };
    }

    let separator = separator.unwrap_or(" ");
    let mut expr_string = String::new();
    for minterm in minterms.iter() {
        let term_string = string_for_minterm(minterm);
        if minterm.dont_care {
            if dont_cares.is_empty() {
                dont_cares = format!("({term_string})");
            } else {
                dont_cares = format!("{dont_cares},{separator} ({term_string})");
            }
            continue;
        } else {
            if term_string == "True" && omit_trivial {
                continue;
            }
            if expr_string.is_empty() {
                expr_string = format!("({term_string})");
            } else {
                expr_string = format!("{expr_string}{separator}| ({term_string})");
            }
        }
    }

    if expr_string.is_empty() {
        FormattedExpr {
            minterm_expr: "True".into(),
            dont_cares,
        }
    } else {
        FormattedExpr {
            minterm_expr: expr_string,
            dont_cares,
        }
    }
}
