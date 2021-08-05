fn main() {}
pub fn hello(y: u8) {
    let mut x = 3;
    if y < x {
        x = y;
    } else {
        x = 3;
    }
}
pub fn test_reference(x: u8) {
    if x == 3 {
        let a = 4;
        let x = a;
    }
}
#[cfg(test)]
mod testify_tests {}
