use std::fmt::Display;

use rand::Rng;

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
#[derive(Clone, Copy, PartialEq, Eq)]
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
    pub fn set_flag(&mut self) {
        self.0 |= M_FLAGGED
    }
    pub fn is_flagged(self) -> bool {
        (self.0 & M_FLAGGED) > 0
    }

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

    pub fn set_safe(&mut self) -> Result<(), MineError> {
        if !self.is_mine() {
            return Err(MineError {
                kind: ErrorKind::GuessError,
                message: "当前标记存在地雷，不可设为【安全】".to_string(),
            });
        }
        self.0 |= M_GUESS_SAFE;
        Ok(())
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
    pub fn bump_warn(&mut self, add: bool) {
        let w = self.get_warn();
        if add {
            if w < 15 {
                self.0 += 1
            }
        } else if w > 0 {
            self.0 -= 1;
        }
    }
    #[inline]
    pub fn get_warn(self) -> u8 {
        (self.0 | 0xf0) ^ 0xf0
    }
}

pub struct MineMap {
    pub count: u8,
    pub width: u8,
    pub height: u8,
    map: Box<[[Mark; 256]; 256]>,
}
impl MineMap {
    pub fn from(count: u8, width: u8, height: u8) -> Self {
        Self {
            count,
            width,
            height,
            map: Box::new([[Mark(0); 256]; 256]),
        }
    }

    /// 预览刷新的地雷
    fn preview_shuffle(&self) -> Vec<Position> {
        let mut mine_map = Vec::with_capacity(self.count as usize);
        let mut rng = rand::thread_rng();
        let mut t = 0;
        while t < self.count {
            let y = rng.gen_range(0..self.height);
            let x = rng.gen_range(0..self.width);
            let xy = Position(x, y);
            if !mine_map.contains(&xy) {
                mine_map.push(xy);
                t += 1;
            }
        }
        mine_map
    }

    // 刷新地雷
    pub fn shuffle(&mut self) {
        let limit = Position(self.width, self.height);
        let ls_pv_mine = self.preview_shuffle();
        let mut map = [[Mark(0); 256]; 256];
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = Position(x, y);
                map[y as usize][x as usize] = if ls_pv_mine.contains(&pos) {
                    Mark(9)
                } else {
                    let a = pos.get_around(limit);
                    let a = a.iter().filter(|a| ls_pv_mine.contains(a)).count();
                    Mark(a as u8)
                }
            }
        } // for line
        self.map = Box::new(map);
    }

    pub fn format_str(&self) -> String {
        let (w, h) = (self.width as usize, self.height as usize);
        let mut buf = String::with_capacity(w * 2 * h + h);
        use std::fmt::Write;
        for y in 0..self.height {
            for x in 0..self.width {
                let m = self.get(x, y);
                if m.is_mine() {
                    buf.push_str(" +");
                    continue;
                }
                match m.get_warn() {
                    0 => buf.push_str("  "),
                    w => write!(buf, " {w}").unwrap(),
                }
            }
            buf.push('\n');
        }
        buf
    }

    pub fn get(&self, x: u8, y: u8) -> &Mark {
        &self.map[y as usize][x as usize]
    }

    pub fn get_by_pos(&self, Position(x, y): Position) -> &Mark {
        self.get(x, y)
    }

    fn get_mut(&mut self, Position(x, y): Position) -> &mut Mark {
        &mut self.map[y as usize][x as usize]
    }

    /// 获取所有标记了猜测的位置
    pub fn get_all_guess(&self) -> Vec<Position> {
        let mut ls = Vec::with_capacity(self.width as usize * self.height as usize / 2);
        for y in 0..self.height {
            for x in 0..self.width {
                let m = self.get(x, y);
                if m.is_guess() {
                    ls.push(Position(x, y));
                }
            }
        }
        ls
    }

    /// 更新所有猜测
    pub fn update_guess(&mut self) -> Result<(), MineError> {
        todo!()
    }

    /// 全图随机抽一个点，设置地雷
    fn try_random_one(&mut self) -> Result<Position, MineError> {
        let mut rng = rand::thread_rng();
        let p = loop {
            let y = rng.gen_range(0..self.height);
            let x = rng.gen_range(0..self.width);
            let m = self.get(x, y);
            if !m.is_open() && !m.is_mine() {
                break Position(x, y);
            }
        };
        self.get_mut(p).set_mine();
        for ap in p.get_around(Position(self.width, self.height)) {
            self.get_mut(ap).bump_warn(true);
        }
        Ok(p)
    }

    // 在指定队列中抽一个点
    // fn try_random_in(&mut self, ls: Vec<Position>) -> Result<Position, MineError> {
    //     todo!()
    // }

    /// 移动地雷（随机）
    pub fn move_mine_randomly(&mut self, pos: Position) -> Result<Position, MineError> {
        let mut ls_candidate = Vec::with_capacity(8);
        let limit = Position(self.width, self.height);
        // remove mine
        '_p1: {
            let around = pos.get_around(limit);
            // count around mines
            let mut cam = 0;
            for ap in around {
                let am = self.get_mut(ap);
                if am.is_mine() {
                    cam += 1;
                    continue;
                }
                // cache around that not open
                if !am.is_open() {
                    ls_candidate.push(ap);
                }
                // update around warn
                am.bump_warn(false);
            }
            self.get_mut(pos).set_warn(cam);
        }
        let len = ls_candidate.len();
        // 1. select from all position
        if len < 1 {
            return self.try_random_one();
        }
        // 2. select from around
        let p2 = if len > 1 {
            ls_candidate[rand::thread_rng().gen_range(0..len)]
        } else {
            ls_candidate[0]
        };
        // set mine
        self.get_mut(p2).set_mine();
        for ap in p2.get_around(limit) {
            let am = self.get_mut(ap);
            if am.is_mine() {
                continue;
            }
            am.bump_warn(true);
        }
        Ok(p2)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position(pub u8, pub u8);
impl Position {
    /// 获取周围3~8格
    pub fn get_around(self, Self(mx, my): Self) -> Vec<Self> {
        use std::ops::RangeInclusive;
        #[inline]
        fn limit(v: u8, max: u8) -> RangeInclusive<u8> {
            let mut mx = v + 1;
            let mut mi = 0;
            if v > 0 {
                mi = v - 1;
            } else if v == (max - 1) {
                mx = v;
            }
            mi..=mx
        }
        let Self(x, y) = self;
        let mut ls = Vec::with_capacity(8);
        for ty in limit(y, my) {
            for tx in limit(x, mx) {
                if tx == x && ty == y {
                    continue;
                }
                ls.push(Self(tx, ty));
            }
        }
        ls
    }
}
impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Position(x, y) = self;
        write!(f, "(x:{x}, y:{y})")
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    GenError,
    GuessError,
    MoveError,
    InvalidPosition,
}

#[derive(Debug)]
pub struct MineError {
    pub kind: ErrorKind,
    pub message: String,
}

#[cfg(test)]
mod tss;
