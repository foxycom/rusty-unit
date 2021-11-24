use syn::{Expr, FnArg, Lit, Type};
use crate::chromosome::Primitive;
use crate::chromosome::{Int, UInt, Float};
use crate::types::{FloatT, IntT, Param, PrimT, UintT};

#[derive(Debug, Default, Clone)]
pub struct TestIdGenerator {
    id: u64,
}

impl TestIdGenerator {
    pub fn new() -> TestIdGenerator {
        TestIdGenerator {
            id: Default::default(),
        }
    }

    pub fn next_id(&mut self) -> u64 {
        self.id += 1;
        self.id
    }

    pub fn reset(&mut self) {
        self.id = Default::default()
    }
}

pub fn generate_random_prim(prim: &PrimT, param: &Param) -> Primitive {
    match prim {
        PrimT::Int(int) => generate_random_int(int, param),
        PrimT::Uint(uint) => generate_random_uint(uint, param),
        PrimT::Float(float) => generate_random_float(float, param),
        PrimT::Str => generate_random_string(param),
        PrimT::Bool => generate_random_bool(param),
        PrimT::Char => generate_random_char(param)
    }
}

pub fn generate_random_int(int: &IntT, param: &Param) -> Primitive {
    let int = match int {
        IntT::Isize => Int::Isize(fastrand::isize(..)),
        IntT::I8 => Int::I8(fastrand::i8(..)),
        IntT::I16 => Int::I16(fastrand::i16(..)),
        IntT::I32 => Int::I32(fastrand::i32(..)),
        IntT::I64 => Int::I64(fastrand::i64(..)),
        IntT::I128 => Int::I128(fastrand::i128(..))
    };

    Primitive::Int(param.clone(), int)
}

pub fn generate_random_uint(uint: &UintT, param: &Param) -> Primitive {
    let uint = match uint {
        UintT::Usize => UInt::Usize(fastrand::usize(..)),
        UintT::U8 => UInt::U8(fastrand::u8(..)),
        UintT::U16 => UInt::U16(fastrand::u16(..)),
        UintT::U32 => UInt::U32(fastrand::u32(..)),
        UintT::U64 => UInt::U64(fastrand::u64(..)),
        UintT::U128 => UInt::U128(fastrand::u128(..))
    };

    Primitive::UInt(param.clone(), uint)
}

pub fn generate_random_float(float: &FloatT, param: &Param) -> Primitive {
    let float = match float {
        FloatT::F32 => Float::F32(fastrand::f32()),
        FloatT::F64 => Float::F64(fastrand::f64())
    };

    Primitive::Float(param.clone(), float)
}

pub fn generate_random_string(param: &Param) -> Primitive {
    let len = fastrand::usize(..);
    let str = if len == 0 {
        "".to_string()
    } else {
        (0..len).map(|_| fastrand::alphanumeric()).collect::<String>()
    };

    Primitive::Str(param.clone(), str)
}

pub fn generate_random_bool(param: &Param) -> Primitive {
    Primitive::Bool(param.clone(), fastrand::bool())
}

pub fn generate_random_char(param: &Param) -> Primitive {
    Primitive::Char(param.clone(), fastrand::alphanumeric())
}