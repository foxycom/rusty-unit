fn main() {
    println!("Hello, world!");
}
fn deep_in(a: u8, b: u8, c: u8, d: u8, e: u8) {
    let mut done = false;
    if a < 20 {
        if b > 50 {
            if c < a {
                if d < a {
                    if e == 0 {
                        done = true;
                    }
                }
            }
        }
    }
}
fn hard(a: u8, b: u8, c: u8, d: u8) {
    let mut done = false;
    if a == 1 {
        if b == 2 {
            if c == 4 {
                if d == 5 {
                    done = true;
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {}
