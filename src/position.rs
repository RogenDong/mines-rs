use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position(pub u8, pub u8);
impl Position {
    #[inline]
    fn limit(v: u8, max: u8) -> std::ops::RangeInclusive<u8> {
        let mut mx = v + 1;
        let mut mi = 0;
        if v > 0 {
            mi = v - 1;
        } else if v >= (max - 1) {
            mx = v;
        }
        mi..=mx
    }
    /// 获取周围一周的坐标（距离1单位）
    /// ## Arguments
    /// x 和 y 的最大可用值
    /// ## Returns
    /// 最少3个，最多8个有效坐标
    pub fn get_around(self, Self(mx, my): Self) -> [Option<Position>; 8] {
        let Self(x, y) = self;
        let mut ls = [None; 8];
        if x >= mx || y >= my {
            return ls;
        }
        let mut t = 0;
        for ty in Self::limit(y, my) {
            for tx in Self::limit(x, mx) {
                if tx == x && ty == y {
                    continue;
                }
                ls[t] = Some(Self(tx, ty));
                t += 1;
            }
        }
        ls
    }

    /// 获取垂直和水平方向两端各延申1单位处的坐标（4个）
    /// ## Arguments
    /// x 和 y 的最大可用值
    /// ## Returns
    /// 最少2个，最多4个有效坐标
    pub fn get_nearby(self, Self(mx, my): Self) -> [Option<Position>; 4] {
        let Self(x, y) = self;
        let mut ls = [None; 4];
        let mut t = 0;
        for ty in Self::limit(y, my) {
            if ty != y {
                ls[t] = Some(Self(x, ty));
                t += 1;
            }
        }
        for tx in Self::limit(x, mx) {
            if tx != x {
                ls[t] = Some(Self(tx, y));
                t += 1;
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
