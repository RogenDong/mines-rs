// 位标识：是否打开
const BIT_OPEN: u8 = 0x80;
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
    pub fn is_open_raw(v: u8) -> bool {
        v & BIT_OPEN > 0
    }

    #[inline]
    pub fn is_open(&self) -> bool {
        self.0 & BIT_OPEN > 0
    }

    #[inline]
    pub fn is_flag(&self) -> bool {
        self.0 & BIT_FLAG > 0
    }

    #[inline]
    pub fn is_mine(&self) -> bool {
        self.get_warn() > 8
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.get_warn() < 1
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
    pub fn switch_open_raw(v: &mut u8) {
        *v ^= BIT_OPEN
    }

    #[inline]
    pub fn switch_flag_raw(v: &mut u8) {
        *v ^= BIT_FLAG
    }

    #[inline]
    pub fn switch_open(&mut self) {
        self.0 ^= BIT_OPEN
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
