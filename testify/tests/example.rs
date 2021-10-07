pub struct MyStruct {
    x: u8,
    y: u8,
}

impl MyStruct {
    pub fn new() -> Self {
        MyStruct { x: 2, y: 2 }
    }

    pub fn do(&self) -> u8 {
    self.x + self.y
}
}
