fn main() {}

struct SomeStruct {
    a: u8,
    b: u8
}

impl SomeStruct {
    pub fn hello(&mut self, y: u8) {
        if y < self.a {
            self.b = y;
        } else {
            self.a = 3;
        }
    }

    pub fn test_reference(&mut self, x: u8) {
        if x == 3 {
            self.a = 4;
        }
    }
}

#[cfg(test)]
mod testify_tests {}
