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
    /// ### Returns
    /// 无效、越界坐标坐标需要自行过滤
    pub fn get_around(&self) -> SmallVec<[Self; 8]> {
        let &Self(x, y) = self;
        let mut ls = SmallVec::new();
        let edge_x = Self::edge(x);
        for ty in Self::edge(y) {
            for tx in edge_x {
                if tx != x || ty != y {
                    ls.push(Self(tx, ty));
                }
            }
        }
        ls
    }

    /// 获取上下左右4格的坐标
    /// ### Returns
    /// 无效、越界坐标坐标需要自行过滤
    pub fn get_nearby(&self) -> SmallVec<[Self; 4]> {
        let &Self(x, y) = self;
        let edge_y = Self::edge(y);
        let edge_x = Self::edge(x);
        let mut ls = SmallVec::new();
        ls.push(Self(x, *edge_y.start()));
        ls.push(Self(*edge_x.start(), y));
        ls.push(Self(x, *edge_y.end()));
        ls.push(Self(*edge_x.end(), y));
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
