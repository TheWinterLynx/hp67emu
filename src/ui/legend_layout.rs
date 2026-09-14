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
    151.7, 212.5, 266.25, 320.5, 373.25, 425.25, 477.25, 528.75, 576.3,
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
    c(0, 58., 46., Top(0)),
    c(0, 112., 46., Top(1)),
    c(0, 166., 46., Top(2)),
    c(0, 220., 46., Top(3)),
    c(0, 274., 46., Top(4)),
    c(1, 58., 46., Text("a", false)),
    c(1, 112., 46., Text("b", false)),
    c(1, 166., 46., Text("c", false)),
    c(1, 220., 46., Text("d", false)),
    c(1, 274., 46., Text("e", false)),
    c(2, 58., 46., Xbar),
    c(2, 112., 46., Pair("GSB", "f", 6.5)),
    c(2, 166., 48., Pair("FIX", "SCI", 7.5)),
    c(2, 220., 46., Text("RND", false)),
    c(2, 274., 46., Pair("LBL", "f", 6.5)),
    c(3, 166., 48., Pair("DSZ", "(i)", 6.0)),
    c(3, 220., 48., Pair("ISZ", "(i)", 6.0)),
    c(4, 56., 58., Text("W/DATA", false)),
    c(4, 119., 58., Text("MERGE", true)),
    c(4, 166., 34., Exchange("P", "S")),
    c(4, 220., 48., Text("CL REG", false)),
    c(4, 274., 48., Text("CL PRGM", false)),
    c(5, 56., 58., Compare('=', false)),
    c(5, 119., 62., TextPower("LN", "e", 11.5)),
    c(5, 194., 66., TextPower("LOG", "10", 10.5)),
    c(5, 270., 66., RadicalPower),
    c(6, 56., 58., Compare('=', true)),
    c(6, 119., 62., Inverse("SIN")),
    c(6, 194., 66., Inverse("COS")),
    c(6, 270., 66., Inverse("TAN")),
    c(7, 56., 58., Compare('<', true)),
    c(7, 119., 62., Conversion("R", "P")),
    c(7, 194., 66., Conversion("D", "R")),
    c(7, 270., 66., Conversion("H", "H.MS")),
    c(8, 56., 58., Compare('>', false)),
    c(8, 119., 62., Pair("%", "%CH", 10.0)),
    c(8, 194., 66., Pair("INT", "FRAC", 10.5)),
    c(8, 270., 66., UnaryStack),
];
// Spacing is expressed in visible-ink units, not font side bearings.
pub const TERM_GAP: f32 = 1.15;
pub const COMPARISON_GAP: f32 = 9.0;
pub const EXCHANGE_WIDTH: f32 = 0.66;
pub const EXCHANGE_GAP: f32 = 0.12;
pub const LETTER_TRACKING: f32 = 0.070;
pub const POWER_GAP: f32 = 0.18;
pub const RADICAL_PAIR_GAP: f32 = 10.5;
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
