pub struct DependencyStruct {
    pub value: u8
}

impl DependencyStruct {
    pub fn new(value: u8) -> Self {
        DependencyStruct { value }
    }
}


pub mod nested_mod {

    pub mod sub_mod {
        pub struct NestedStruct {

        }

        impl NestedStruct {
            pub fn new() -> Self {
                NestedStruct {}
            }

            pub fn nested_fn() {

            }
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let s = crate::dependency::DependencyStruct::new(2);

    }
}