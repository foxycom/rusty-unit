use std::borrow::Borrow;
fn main() {}
struct SomeStruct {
    a: u8,
    b: u8,
}
impl SomeStruct {
    pub fn new(a: u8, b: u8) -> SomeStruct {
        SomeStruct { a, b }
    }
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
struct Rectangle {
    width: u8,
    height: u8,
}
impl Rectangle {
    pub fn new(width: u8, height: u8) -> Self {
        Rectangle { width, height }
    }
    pub fn width(&self) -> u8 {
        self.width
    }
}
struct AreaCalculator {}
impl AreaCalculator {
    pub fn new() -> Self {
        AreaCalculator {}
    }
    pub fn area_by_value(&self, rect: Rectangle) -> f64 {
        rect.height as f64 * rect.width as f64
    }
    pub fn area_by_ref(&self, rect: &Rectangle) -> f64 {
        rect.height as f64 * rect.width as f64
    }
}
#[cfg(test)]
mod testify_tests {
    use super::*;
    #[test]
    fn testify_201() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(201u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_202() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(202u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_203() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(203u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_204() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(204u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_205() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(205u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_206() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(206u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_207() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(207u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_208() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(208u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_209() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(209u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_210() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(210u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_211() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(211u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_212() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(212u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_213() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(213u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_214() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(214u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_215() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(215u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_216() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(216u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_217() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(217u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_218() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(218u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_219() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(219u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
    #[test]
    fn testify_220() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(220u64));
        LOGGER.with(|l| l.borrow_mut().wait());
    }
}
