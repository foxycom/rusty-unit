use std::cell::RefCell;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::rc::Rc;
use std::str::from_utf8;

thread_local! {
    static MONITOR: Rc<RefCell<Monitor>> = Rc::new(RefCell::new(Monitor::new()));
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    /// The `+` operator
    Add,
    /// The `-` operator (subtraction)
    Sub,
    /// The `*` operator (multiplication)
    Mul,
    /// The `/` operator (division)
    Div,
    /// The `%` operator (modulus)
    Rem,
    /// The `^` operator (bitwise xor)
    BitXor,
    /// The `&` operator (bitwise and)
    BitAnd,
    /// The `|` operator (bitwise or)
    BitOr,
    /// The `<<` operator (shift left)
    Shl,
    /// The `>>` operator (shift right)
    Shr,
    /// The `==` operator (equality)
    Eq,
    /// The `<` operator (less than)
    Lt,
    /// The `<=` operator (less than or equal to)
    Le,
    /// The `!=` operator (not equal to)
    Ne,
    /// The `>=` operator (greater than or equal to)
    Ge,
    /// The `>` operator (greater than)
    Gt,
    /// The `ptr.offset` operator
    Offset,
}

impl Into<u32> for BinaryOp {
    fn into(self) -> u32 {
        match self {
            BinaryOp::Add => 0,
            BinaryOp::Sub => 1,
            BinaryOp::Mul => 2,
            BinaryOp::Div => 3,
            BinaryOp::Rem => 4,
            BinaryOp::BitXor => 5,
            BinaryOp::BitAnd => 6,
            BinaryOp::BitOr => 7,
            BinaryOp::Shl => 8,
            BinaryOp::Shr => 9,
            BinaryOp::Eq => 10,
            BinaryOp::Lt => 11,
            BinaryOp::Le => 12,
            BinaryOp::Ne => 13,
            BinaryOp::Ge => 14,
            BinaryOp::Gt => 15,
            BinaryOp::Offset => 16,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    /// The `-` operator for negation
    Neg,
}

impl Into<u32> for UnaryOp {
    fn into(self) -> u32 {
        match self {
            UnaryOp::Not => 0,
            UnaryOp::Neg => 1,
        }
    }
}

struct Monitor {
    connection: TcpStream,
    test_id: u64,
}

const ROOT_BRANCH: &'static str = "{} root[{}, {}];";
const BRANCH: &'static str = "{} branch[{}, {}, {}];";

impl Monitor {
    pub fn set_test_id(&mut self, test_id: u64) {
        self.test_id = test_id;
    }

    pub fn trace_fn(&mut self, global_id: f64, id: f64) {
        let msg = format!("root[{}, {}];", global_id, id);
        self.send(&msg);
    }
    pub fn trace_branch(&mut self, global_id: u64, block: u64, dist: f64) {
        let msg = format!("branch[{} {} {}];", global_id, block, dist);
        self.send(&msg);
    }

    fn send(&mut self, msg: &str) {
        self.connection.write(msg.as_bytes()).unwrap();
    }

    fn new() -> Self {
        let connection = match TcpStream::connect("localhost:3333") {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Failed to connect: {}", e);
            }
        };

        Monitor {
            connection,
            test_id: 0,
        }
    }
}

pub fn trace_fn(global_id: f64, id: f64) {
    MONITOR.with(|m| m.borrow_mut().trace_fn(global_id, id));
}

pub fn trace_branch_enum(global_id: u64, block: u64, is_hit: bool) {
    let dist = if is_hit { 0.0 } else { 1.0 };
    MONITOR.with(|m| {
        m.borrow_mut()
            .trace_branch(global_id, block, dist)
    });
}

pub fn trace_branch_hit(global_id: u64, block: u64) {
    MONITOR.with(|m| {
        m.borrow_mut().trace_branch(global_id, block, 0.0);
    })
}

pub fn trace_branch_bool(
    global_id: u64,
    block: u64,
    left: f64,
    right: f64,
    op: BinaryOp,
    is_true_branch: bool,
) {
    let dist = distance(left, right, op, is_true_branch);
    MONITOR.with(|m| {
        m.borrow_mut()
            .trace_branch(global_id, block, dist)
    });
}

fn distance(left: f64, right: f64, op: BinaryOp, is_true_branch: bool) -> f64 {
    match op {
        // left <= right
        BinaryOp::Le => {
            if is_true_branch {
                // left <= right
                right - left + 1.0
            } else {
                // left > right
                left - right
            }
        }
        // left < right
        BinaryOp::Lt => {
            if is_true_branch {
                // left < right
                right - left
            } else {
                // left >= right
                left - right + 1.0
            }
        }
        // left > right
        BinaryOp::Gt => {
            if is_true_branch {
                // left > right
                left - right
            } else {
                // left <= right
                right - left + 1.0
            }
        }
        // left >= right
        BinaryOp::Ge => {
            if is_true_branch {
                // left >= right
                left - right + 1.0
            } else {
                // left < right
                right - left
            }
        }
        // left == right
        BinaryOp::Eq => {
            if is_true_branch {
                // left == right
                1.0
            } else {
                (left - right).abs()
            }
        }
        // left != right
        BinaryOp::Ne => {
            if is_true_branch {
                // left != right
                (left - right).abs()
            } else {
                // left == right
                1.0
            }
        }
        _ => todo!(),
    }
}

pub fn set_test_id(id: u64) {
    MONITOR.with(|m| m.borrow_mut().set_test_id(id));
}
