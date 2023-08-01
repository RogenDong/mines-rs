/// 标记-猜-安全
const M_GUESS_SAFE: u8 = 0x10;
/// 标记-猜-可疑
const M_GUESS_SUSP: u8 = 0x20;
/// 标记-猜-确认地雷
const M_GUESS_MINE: u8 = 0x30;

#[derive(PartialEq, Eq)]
pub enum MineGuess {
    Mine,
    Safe,
    Suspicious,
}

/// 标记-插旗
const M_FLAGGED: u8 = 0x40;
/// 标记-打开
const M_OPENED: u8 = 0x80;

/// 状态
/// 每一位的含义：
/// ``` txt
///      ↓↓↓↓-warn-附近地雷数（8<?<16为地雷）
/// 0000 0000
/// |||└ 未知/安全(0/1) 0x30 = 地雷
/// ||└ 未知/可疑(0/1)  0x30 = 地雷
/// |└ 是否标记(0/1)
/// └ 是否打开(0/1)
/// ```
#[derive(Clone, Copy, Eq)]
pub struct Mark(pub u8);
impl Mark {
    /// 根据选项构造一个标记
    pub fn from(around: u8, guess: MineGuess, flagged: bool, opened: bool) -> Self {
        let mut v = (around | 0xf0) ^ 0xf0;
        if opened {
            v |= M_OPENED;
        }
        if flagged {
            v |= M_FLAGGED;
        }
        use MineGuess::*;
        v |= match guess {
            Mine => M_GUESS_MINE,
            Safe => M_GUESS_SAFE,
            Suspicious => M_GUESS_SUSP,
        };
        Self(v)
    }
    pub fn is_empty(self) -> bool {
        self.0 == 0 || (self.0 | 0xf0) == 0xf0
    }

    /// 切换插旗
    /// **不提供参数指定“是/否”，而是切换**
    pub fn set_flag(&mut self) {
        self.0 ^= M_FLAGGED
    }
    pub fn is_flagged(self) -> bool {
        (self.0 & M_FLAGGED) > 0
    }

    /// 打开此单元
    /// **只能开，不能关**
    pub fn set_open(&mut self) {
        self.0 |= M_OPENED
    }
    pub fn is_open(self) -> bool {
        self.0 >= M_OPENED
    }

    pub fn set_mine(&mut self) {
        self.0 = ((self.0 | 0xf) ^ 0xf) | 9
    }
    pub fn is_mine(self) -> bool {
        ((self.0 | 0xf0) ^ 0xf0) > 8
    }
    
    pub fn set_safe(&mut self) {
        if !self.is_mine() {
            self.0 |= M_GUESS_SAFE;
        }
    }
    pub fn set_suspicious(&mut self) {
        self.0 |= M_GUESS_SUSP
    }

    pub fn is_guess(self) -> bool {
        (self.0 & M_GUESS_MINE) > 0
    }
    pub fn guess_mine(self) -> bool {
        (self.0 & M_GUESS_MINE) == M_GUESS_MINE
    }
    pub fn guess_safe(self) -> bool {
        (self.0 & M_GUESS_MINE) == M_GUESS_SAFE
    }
    pub fn guess_suspicious(self) -> bool {
        (self.0 & M_GUESS_MINE) == M_GUESS_SUSP
    }

    pub fn set_warn(&mut self, warn: u8) {
        let mut w = warn;
        if w > 15 {
            w = (w | 0xf0) ^ 0xf0;
        }
        self.0 = ((self.0 | 0xf) ^ 0xf) | w
    }
    /// 增减 warn 值
    /// ### Arguments
    /// `add` true-增加；false-减少
    /// ### Return
    /// 增减后的值
    pub fn bump_warn(&mut self, add: bool) -> u8 {
        let w = self.get_warn();
        if add {
            if w < 15 {
                self.0 += 1
            }
        } else if w > 0 {
            self.0 -= 1;
        }
        self.get_warn()
    }
    #[inline]
    pub fn get_warn(self) -> u8 {
        (self.0 | 0xf0) ^ 0xf0
    }
}
impl PartialEq for Mark {
    fn eq(&self, other: &Self) -> bool {
        (self.0 | 0xf0) == (other.0 | 0xf0)
    }
}
