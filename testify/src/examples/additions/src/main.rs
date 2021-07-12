fn main() {
    println!("Hello, world!");
    conditional_add_u64(2, 3);
}

pub fn conditional_add_u64(a: u64, b: u64) -> u64 {
    if a > b {
        b
    } else {
        if b > 0 {
            a
        } else {
            a + b
        }
    }
}