use smallvec::SmallVec;

#[derive(Debug, Clone, Copy, Eq)]
pub struct Loc(pub u8, pub u8);
impl Loc {
    pub fn new(x: u8, y: u8) -> Self {
        Loc(x, y)
    }

    fn edge(v: u8) -> std::ops::RangeInclusive<u8> {
        let mut mx = u8::MAX;
        let mut mi = 0;
        if v > 0 {
            mi = v - 1;
        }
        if v < u8::MAX {
            mx = v + 1;
        }
        mi..=mx
    }
    
    /// 获取周围一周的坐标（距离1单位）
    /// ## Returns
    /// 最少3个，最多8个有效坐标
    pub fn get_around(&self) -> SmallVec<[Loc; 8]> {
        let &Loc(x, y) = self;
        let mut ls = SmallVec::new();
        for ty in Self::edge(y) {
            for tx in Self::edge(x) {
                if tx != x || ty != y {
                    ls.push(Self(tx, ty));
                }
            }
        }
        ls
    }
}

impl PartialEq for Loc {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

use std::fmt::{Display, Formatter, Result};
impl Display for Loc {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "(x:{}, y:{})", self.0, self.1)
    }
}
