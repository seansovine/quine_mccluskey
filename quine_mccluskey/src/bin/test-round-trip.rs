//! Test correctness of QM by round-tripping from init
//! string to minimized sum-of-products and back to init.

use std::time::Instant;

use logic_minimization::{check::sop_string_to_init, qm_simplify_init};
use rand::Rng;

const NUM_CASES: usize = 50;

fn main() {
    let mut rng = rand::rng();
    for i in 0..NUM_CASES {
        let init: u64 = rng.random_range(0..=u64::MAX);
        let init_string = format!("{init:016X}");
        print!("{:02}: Testing INIT value {init_string} ... ", i + 1);
        let (sop_string, time_millis) = timed_qm(&init_string);
        let return_init = sop_string_to_init(&sop_string);
        match init_string == return_init {
            true => println!("PASSED."),
            false => println!("FAILED. Round trip INIT was: {return_init}."),
        }
        println!("    QM time: {time_millis} ms")
    }
}

fn timed_qm(init_str: &str) -> (String, u128) {
    let start_time = Instant::now();
    let (sop_string, _time) = qm_simplify_init(init_str).expect("Init conversion failed.");
    let elapsed = start_time.elapsed().as_millis();
    (sop_string, elapsed)
}
