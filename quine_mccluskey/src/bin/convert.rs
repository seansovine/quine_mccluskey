use logic_minimization::binary_strings_from_init_hex;

fn main() {
    const TEST_STR: &str = "0000F0F0000000FF";
    let strings = binary_strings_from_init_hex(TEST_STR);
    dbg!(&strings);
}
