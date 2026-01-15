//! Test correctness of QM by round-tripping from init
//! string to minimized sum-of-products and back to init.

use std::time::Instant;

use rand::Rng;

use logic_minimization::{convert::sop_string_to_init, qm_simplify_init, qm_simplify_init_greedy};

const NUM_CASES: usize = 200;
const USE_GREEDY: bool = true;
const DEBUG_INITS: bool = false;

#[allow(unused)]
#[derive(Default)]
struct ExampleInfo {
    init: String,
    time_millis: u128,
    num_out_terms: u32,

    // TODO: Add API functions to populate these.
    num_in_terms: u32,
    num_prime_impls: u32,
}

fn main() {
    let mut rng = rand::rng();
    let mut failures = 0;
    let mut example_info = vec![];

    for i in 0..NUM_CASES {
        let init: u64 = rng.random_range(0..=u64::MAX);
        let init_string = format!("{init:016X}");

        print!("{:02}: Testing INIT value {init_string} ... ", i + 1);

        let (sop_string, num_minterms, time_millis) = timed_qm(&init_string, false);
        let return_init = sop_string_to_init(&sop_string);

        match init_string == return_init {
            true => println!("PASSED ({num_minterms} minterms)."),
            false => {
                println!("FAILED. Round trip INIT was: {return_init}.");
                failures += 1;
            }
        }
        if DEBUG_INITS {
            println!("    minimal SOP: {sop_string}");
        }
        println!("    QM time: {time_millis} ms");

        if time_millis >= 500 {
            example_info.push(ExampleInfo {
                init: init_string.clone(),
                time_millis,
                num_out_terms: num_minterms as u32,
                ..Default::default()
            });
        }

        if !USE_GREEDY {
            continue;
        }

        print!("--: Testing INIT value {init_string} using GREEDY approximation ... ");

        let (sop_string, num_minterms, time_millis) = timed_qm(&init_string, true);
        let return_init = sop_string_to_init(&sop_string);

        match init_string == return_init {
            true => println!("PASSED ({num_minterms} minterms)."),
            false => {
                println!("FAILED. Round trip INIT was: {return_init}.");
                failures += 1;
            }
        }
        if DEBUG_INITS {
            println!("    minimal SOP: {sop_string}");
        }
        println!("    QM time: {time_millis} ms");
    }

    println!("\n========\n");
    println!("Results:\n");
    println!("- Passes: {}", NUM_CASES - failures);
    println!("- Failures: {}\n", failures);

    if !example_info.is_empty() {
        println!("Long-running examples:");
        for example in example_info {
            println!("  INIT: 16'h{}", example.init);
            println!("  Time: {} ms\n", example.time_millis);
        }
        // IDEA: It could be interesting to plot # prime impls vs. time taken.
    }
}

fn timed_qm(init_str: &str, greedy: bool) -> (String, usize, u128) {
    let start_time = Instant::now();
    let (sop_string, num_minterms) = if greedy {
        qm_simplify_init_greedy(init_str).expect("Init conversion failed.")
    } else {
        let (sop_string, num_minterms, _time) =
            qm_simplify_init(init_str).expect("Init conversion failed.");
        (sop_string, num_minterms)
    };
    let elapsed = start_time.elapsed().as_millis();

    (sop_string, num_minterms, elapsed)
}
