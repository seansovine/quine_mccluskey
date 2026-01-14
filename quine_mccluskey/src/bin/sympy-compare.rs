//! Simplify randomly generated formulas with Sympy and compare with our results.

use std::{
    io::Write,
    process::{Command, Stdio},
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use logic_minimization::{
    Minterm,
    check::sop_to_minterms,
    format::{binary_strings_from_init_hex, string_for_sop_minterms},
    qm_simplify_init,
};

use rand::Rng;

const NUM_CASES: usize = 20;
const DEV_DEBUG: bool = false;

const TEMPLATE: &str = r#"
from sympy.logic.boolalg import to_dnf
from sympy.abc import A, B, C, D, E, F

result = to_dnf(
	{},
    simplify=True,
)

print(result)
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::rng();
    let mut failures = 0;

    for i in 0..NUM_CASES {
        let init: u64 = rng.random_range(0..=u64::MAX);
        let init_string = format!("{init:016X}");
        println!("{:02}: Testing INIT value {init_string} ... ", i + 1);

        let term_strings = binary_strings_from_init_hex(&init_string)?;
        let minterms = term_strings
            .iter()
            .map(|s| (&**s).into())
            .collect::<Vec<Minterm>>();
        let sop_string = string_for_sop_minterms(&minterms, true, Some(" "));
        let python_script = TEMPLATE.replace("{}", &sop_string);
        if DEV_DEBUG {
            println!("{python_script}");
        }

        let args: [&str; 2] = ["-c", &python_script];
        let child = Command::new("python")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn child.");

        let child_output = child
            .wait_with_output()
            .expect("Failed to read child stdout.");
        let sympy_result = String::from_utf8_lossy(&child_output.stdout);
        let err = String::from_utf8_lossy(&child_output.stderr);

        if !err.is_empty() {
            return Err(err.into());
        }
        if DEV_DEBUG {
            println!("Result: {sympy_result}");
        }

        // Parse into minterm vector and count terms.
        let sympy_minterms = sop_to_minterms(&sympy_result);
        println!("  Sympy result has {} minterms.", sympy_minterms.len());

        // Simplify with our code and count terms.
        let (_, rust_num_minterms, _) =
            qm_simplify_init(&init_string).expect("Init conversion failed.");
        println!("  Rust Q-M result has {rust_num_minterms} minterms.");

        if rust_num_minterms == sympy_minterms.len() {
            let mut stdout = StandardStream::stdout(ColorChoice::Always);
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            writeln!(&mut stdout, "  Test passed.")?;
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
        } else {
            let mut stdout = StandardStream::stdout(ColorChoice::Always);
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
            writeln!(&mut stdout, "  Test failed!")?;
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
            failures += 1;
        }
    }

    println!("\n========\n");
    println!("Results:\n");
    println!("- Passes: {}", NUM_CASES - failures);
    println!("- Failures: {}\n", failures);

    Ok(())
}
