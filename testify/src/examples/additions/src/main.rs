use crate::dependency::DependencyStruct;

mod dependency;

fn main() {}
struct SomeStruct {
    a: u8,
    b: u8,
    dependency: dependency::DependencyStruct
}
impl SomeStruct {
    pub fn new(a: u8, b: u8) -> SomeStruct {
        SomeStruct { a, b, dependency: DependencyStruct{} }
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

}
