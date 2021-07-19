fn main() {
    println!("Hello, world!");
    conditional_add_u8(2, 3);
}
pub fn conditional_add_u8(a: u8, b: u8) -> u8 {
    if a > b {
        b
    } else {
        if b < 0 {
            a
        } else {
            a + b
        }
    }
}
#[cfg(test)]
mod tests {}
