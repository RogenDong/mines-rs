use std::fmt::{Debug, Display, Formatter, Result};

// 位标识：是否打开
const BIT_REVEAL: u8 = 0x80;
// 位标识：是否插旗
const BIT_FLAG: u8 = 0x40;
// 位标识：周围地雷数
const BIT_WARN: u8 = 0x1F;

/// # 分析每个单元格的状态
/// ### 位含义
/// - `1000 0000` 是否已打开
/// - `0100 0000` 是否已插旗
/// - `0001 1111` 周围地雷数
#[derive(Clone, Copy, Eq)]
pub struct Cell(pub u8);

impl Cell {
    #[inline]
    pub fn get_warn(&self) -> u8 {
        self.0 & BIT_WARN
    }

    #[inline]
    pub fn is_reveal(&self) -> bool {
        self.0 >= BIT_REVEAL
    }

    #[inline]
    pub fn is_flagged(&self) -> bool {
        self.0 & BIT_FLAG == BIT_FLAG
    }

    #[inline]
    pub fn is_mine(&self) -> bool {
        self.get_warn() > 8
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.get_warn() == 0
    }

    pub fn bmp(&mut self, add: bool) {
        let v = self.get_warn();
        if add {
            if v != BIT_WARN {
                self.0 += 1;
            }
        } else if v > 0 {
            self.0 -= 1;
        }
    }

    #[inline]
    pub fn reveal(&mut self) {
        self.0 = self.0 & BIT_WARN | BIT_REVEAL
    }

    #[inline]
    pub fn switch_flag(&mut self) {
        self.0 ^= BIT_FLAG
    }
}
impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        self.get_warn() == other.get_warn()
    }
}
impl Display for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Cell({:0>8b})", self.0)
    }
}
impl Debug for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Cell")
            .field("value", &format!("{:0>8b}", self.0))
            .field("flagged", &self.is_flagged())
            .field("reveal", &self.is_reveal())
            .field("warn", &self.get_warn())
            .finish()
    }
}
