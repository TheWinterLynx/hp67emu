//! Legend matrix in the same logical coordinate system as the 35-key matrix.
//! Row centers use the free space between key rows, not independent y nudges.
#[derive(Clone, Copy, Debug)]
pub enum Mark {
    Top(u8),
    Text(&'static str, bool),
    Pair(&'static str, &'static str, f32),
    Xbar,
    Exchange(&'static str, &'static str),
    Conversion(&'static str, &'static str),
    Compare(char, bool),
    TextPower(&'static str, &'static str, f32),
    RadicalPower,
    Inverse(&'static str),
    UnaryStack,
}
#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub row: usize,
    pub x: f32,
    pub width: f32,
    pub mark: Mark,
}
pub const ROW_Y: [f32; 9] = [
    162.0, 220.0, 271.5, 324.0, 375.0, 426.0, 476.0, 527.0, 576.5,
];
const fn c(row: usize, x: f32, width: f32, mark: Mark) -> Cell {
    Cell {
        row,
        x,
        width,
        mark,
    }
}
use Mark::*;
pub const CELLS: &[Cell] = &[
    c(0, 54.5, 46., Top(0)),
    c(0, 110.5, 46., Top(1)),
    c(0, 166.0, 46., Top(2)),
    c(0, 221.5, 46., Top(3)),
    c(0, 277.0, 46., Top(4)),
    c(1, 54.5, 46., Text("a", false)),
    c(1, 110.5, 46., Text("b", false)),
    c(1, 166.0, 46., Text("c", false)),
    c(1, 221.5, 46., Text("d", false)),
    c(1, 277.0, 46., Text("e", false)),
    c(2, 54.5, 46., Xbar),
    c(2, 110.5, 46., Pair("GSB", "f", 6.5)),
    c(2, 166.0, 48., Pair("FIX", "SCI", 7.5)),
    c(2, 221.5, 46., Text("RND", false)),
    c(2, 277.0, 46., Pair("LBL", "f", 6.5)),
    c(3, 166.0, 48., Pair("DSZ", "(i)", 6.0)),
    c(3, 221.5, 48., Pair("ISZ", "(i)", 6.0)),
    c(4, 53.5, 58., Text("W/DATA", false)),
    c(4, 118.5, 58., Text("MERGE", true)),
    c(4, 166.0, 34., Exchange("P", "S")),
    c(4, 221.5, 48., Text("CL REG", false)),
    c(4, 277.0, 48., Text("CL PRGM", false)),
    c(5, 53.5, 58., Compare('=', false)),
    c(5, 118.5, 62., TextPower("LN", "e", 13.5)),
    c(5, 195.5, 66., TextPower("LOG", "10", 12.25)),
    c(5, 274.0, 66., RadicalPower),
    c(6, 53.5, 58., Compare('=', true)),
    c(6, 118.5, 62., Inverse("SIN")),
    c(6, 195.5, 66., Inverse("COS")),
    c(6, 274.0, 66., Inverse("TAN")),
    c(7, 53.5, 58., Compare('<', true)),
    c(7, 118.5, 62., Conversion("R", "P")),
    c(7, 195.5, 66., Conversion("D", "R")),
    c(7, 274.0, 66., Conversion("H", "H.MS")),
    c(8, 53.5, 58., Compare('>', false)),
    c(8, 118.5, 62., Pair("%", "%CH", 11.0)),
    c(8, 195.5, 66., Pair("INT", "FRAC", 11.5)),
    c(8, 274.0, 66., UnaryStack),
];
// Spacing is expressed in visible-ink units, not font side bearings.
pub const TERM_GAP: f32 = 1.15;
pub const COMPARISON_GAP: f32 = 9.0;
// HP-67 exchange mark: two opposed filled arrowheads only, with no shafts.
// 0.72 makes each tip-to-base triangle meet cleanly at the visual center.
pub const EXCHANGE_WIDTH: f32 = 0.72;
pub const EXCHANGE_GAP: f32 = 0.11;
pub const LETTER_TRACKING: f32 = 0.070;
pub const POWER_GAP: f32 = 0.18;
pub const RADICAL_PAIR_GAP: f32 = 12.25;
pub const INVERSE_GAP: f32 = 1.4;

#[derive(Clone, Copy)]
pub enum ExchangeForm {
    Heads,
    Conversion,
}
impl ExchangeForm {
    pub fn width(self, size: f32) -> f32 {
        size * match self {
            Self::Heads => EXCHANGE_WIDTH,
            Self::Conversion => 1.1,
        }
    }
    pub fn gap(self, size: f32) -> f32 {
        size * match self {
            Self::Heads => EXCHANGE_GAP,
            Self::Conversion => 0.32,
        }
    }
}
