
#![feature(no_coverage)]

#[cfg_attr(not(feature = "std"), no_std)]
#[allow(unused_imports)]
#[cfg(feature = "connect_four")]
pub mod connect_four;


#[cfg(feature = "gomoku")]
pub mod gomoku;

#[cfg(feature = "minesweeper")]
pub mod minesweeper;

#[cfg(feature = "reversi")]
pub mod reversi;

#[cfg(feature = "tictactoe")]
#[feature(no_coverage)]
pub mod tictactoe;

#[cfg(feature = "std")]
mod std_lib {
    pub(crate) use std::{
        boxed::Box,
        cmp::Ordering,
        collections::VecDeque,
        convert::Infallible,
        iter,
        ops::{Index, IndexMut},
        vec::Vec,
    };
}

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

#[cfg(not(feature = "std"))]
mod std_lib {
    pub(crate) use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
    pub(crate) use core::{
        cmp::Ordering,
        convert::Infallible,
        iter,
        ops::{Index, IndexMut},
    };
}

pub use ntest::timeout;
pub mod rusty_monitor;
