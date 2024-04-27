use crate::{cell::Cell, location::Loc};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use smallvec::SmallVec;

// const MAX_LEN: usize = 255 * 255;
// const MAX_LEN_STAT: usize = MAX_LEN / 8;

// fn is_tag(map: &Vec<u8>, f: usize) -> bool {
//     if f == 0 {
//         map[0] & 0x80 > 0
//     } else if f < MAX_LEN {
//         map[f / 8] & (0x80 >> (f % 8)) > 0
//     } else {
//         false
//     }
// }

// fn set_tag(map: &mut Vec<u8>, f: usize) {
//     if f == 0 {
//         map[0] |= 1;
//     } else if f < MAX_LEN {
//         map[f / 8] |= 1 << (f % 8);
//     }
// }

/// 基于长宽和二维坐标换算得到下标
/// * 地雷、数量等信息存储在简单数组中
#[inline]
pub fn get_idx(x: usize, y: usize, w: usize, h: usize) -> Option<usize> {
    if x < w && y < h {
        Some(y * w + x)
    } else {
        None
    }
}

/// 基于长宽和二维坐标检查周围单位，收集并返回需要自增的位置
/// # Return
/// - 需要自增的位置是有效下标
/// - 不自增的位置用大于地图最大长度的值表示
#[allow(non_snake_case)]
pub fn get_bmp_idx(x: usize, y: usize, w: usize, h: usize) -> Option<[usize; 9]> {
    // “不自增”标记值--用较大数值表示；减1是为了方便后续处理
    const M: usize = usize::MAX - 1;
    // 获取中间单位的下标
    let i = get_idx(x, y, w, h)?;
    if i == 0 {
        // - - -
        // - + E
        // - S D
        return Some([M, M, M, M, M, 1, M, w, w + 1]);
    }
    let lim = w * h;
    if i >= lim {
        return None;
    }
    // 周围单位的下标
    let (iN, iW, iE, iS) = (
        if y == 0 { M } else { i - w },
        if x == 0 { M } else { i - 1 },
        if x == w - 1 { M } else { i + 1 },
        if y == h - 1 { M } else { i + w },
    );
    // 中心单位在末尾的时候直接返回
    if i == lim - 1 {
        // A N -
        // W + -
        // - - -
        return Some([iN - 1, iN, M, iW, M, M, M, M, M]);
    }
    // 初始化下标集合
    // A=N-1  N=i-w  B=N+1
    // W=i-1         E=i+1
    // C=S-1  S=i+w  D=S+1
    let mut ls = [
        if iN == 0 { 0 } else { iN - 1 },
        iN,
        iN + 1,
        iW,
        M,
        iE,
        iS - 1,
        iS,
        iS + 1,
    ];
    let (A, N, B) = (0, 1, 2);
    let (W, E) = (3, 5);
    let (C, S, D) = (6, 7, 8);
    if x == 0 {
        // N B
        //   E
        // S D
        ls[W] = M;
        ls[A] = M;
        ls[C] = M;
    } else if x == w - 1 {
        // A N
        // W
        // C S
        ls[E] = M;
        ls[B] = M;
        ls[D] = M;
    }
    if y == 0 {
        // W   E
        // C S D
        ls[N] = M;
        ls[A] = M;
        ls[B] = M;
    } else if y == h - 1 {
        // A N B
        // W   E
        ls[S] = M;
        ls[C] = M;
        ls[D] = M;
    }
    Some(ls)
}

pub struct MineMap {
    // u8::MAX ** 2 < u16::MAX
    pub count: u16,
    pub width: u8,
    pub height: u8,
    pub map: Vec<u8>,
    // stat: Vec<u8>,
}
impl MineMap {
    pub fn new(count: u16, width: u8, height: u8) -> Self {
        let cap = width as usize * height as usize;
        let cap = if count == 0 || cap < count as usize {
            0
        } else {
            cap
        };
        Self {
            count,
            width,
            height,
            map: vec![0; cap],
            // stat: vec![0; MAX_LEN_STAT],
        }
    }

    #[inline]
    fn get_idx(&self, x: usize, y: usize) -> Option<usize> {
        get_idx(x, y, self.width as usize, self.height as usize)
    }

    // #[inline]
    // fn get_index_by_loc(&self, Loc(x, y): Loc) -> Option<usize> {
    //     self.get_idx(x as usize, y as usize)
    // }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<Cell> {
        Some(Cell(*self.map.get(self.get_idx(x, y)?)?))
    }

    #[inline]
    pub fn get_by_loc(&self, Loc(x, y): Loc) -> Option<Cell> {
        self.get(x as usize, y as usize)
    }

    /// 设置安全区
    fn ignore_area(&mut self, area: SmallVec<[Loc; 8]>) {
        let mut rng = thread_rng();
        for &Loc(a, b) in area.iter() {
            match self.get_mut(a as usize, b as usize) {
                Some(c @ 9) => *c = 0,
                _ => continue,
            }
            loop {
                let x = rng.gen_range(0..self.width);
                let y = rng.gen_range(0..self.height);
                if area.iter().any(|&Loc(a, b)| a == x && b == y) {
                    continue;
                }
                if let Some(c @ 0) = self.get_mut(x as usize, y as usize) {
                    *c = 9;
                    break;
                }
            }
        }
    }

    /// 刷新地雷
    pub fn shuffle(&mut self, ignore: Option<Loc>) {
        let (w, h, c) = (
            self.width as usize,
            self.height as usize,
            self.count as usize,
        );
        let map_size = w * h;
        if c == 0 || map_size < c {
            return;
        }
        // self.stat.fill(0);
        self.map.fill(0);
        self.map[..c].fill(9);
        // 用洗牌算法布置地雷
        let mut rng = thread_rng();
        self.map.shuffle(&mut rng);
        // 设置安全区
        if let Some(c) = ignore {
            self.ignore_area(c.get_around());
        }
        // 设置地雷数值
        for y in 0..h {
            for x in 0..w {
                if self.get(x, y).map_or(0, |c| c.0) < 9 {
                    continue;
                }
                // get around
                let Some(ls) = get_bmp_idx(x, y, w, h) else { continue };
                for i in ls {
                    if i < map_size {
                        self.map[i] += 1;
                    }
                }
            }
        }
    }

    pub fn format_str(&self) -> String {
        let (w, h) = (self.width as usize, self.height as usize);
        let mut buf = String::with_capacity(w * 2 * h + h);
        use std::fmt::Write;
        for y in 0..h {
            for x in 0..w {
                if let Some(c) = self.get(x, y) {
                    if c.is_mine() {
                        buf.push_str(" +");
                        continue;
                    }
                    match c.get_warn() {
                        0 => buf.push_str("  "),
                        w => write!(buf, " {w}").unwrap(),
                    }
                }
            }
            buf.push('\n');
        }
        buf
    }

    // private
    #[inline]
    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut u8> {
        let w = self.width as usize;
        if x < w && y < self.height as usize {
            Some(self.map.get_mut(y * w + x)?)
        } else {
            None
        }
    }

    // fn set_tag(&mut self, x: usize, y: usize) {
    //     if let Some(f) = self.get_idx(x, y) {
    //         set_tag(&mut self.stat, f);
    //     }
    // }

    pub fn open(&mut self, x: usize, y: usize) {
        if let Some(v) = self.get_mut(x, y) {
            // let mut c = Cell(*v);
            if !Cell::is_open_raw(*v) {
                Cell::switch_open_raw(v);
                // self.set_tag(x, y);
            }
        }
    }

    pub fn open_by_loc(&mut self, Loc(x, y): Loc) {
        self.open(x as usize, y as usize)
    }

    pub fn switch_flag(&mut self, x: usize, y: usize) {
        if let Some(v) = self.get_mut(x, y) {
            Cell::switch_flag_raw(v);
            // self.set_tag(x, y);
        }
    }

    pub fn switch_flag_by_loc(&mut self, Loc(x, y): Loc) {
        self.switch_flag(x as usize, y as usize)
    }

    // pub fn get_nearby_empty_area(&self, x: usize, y: usize) -> Result<Vec<Loc>, ()> {
    //     let (w, h) = (self.width as usize, self.height as usize);
    //     if x >= w || y >= h {
    //         return Err(());
    //     }
    //     let start_pos = Loc(x as u8, y as u8);
    //     let start_idx = self.get_index_by_loc(start_pos).ok_or_else(|| ())?;
    //
    //     let mut stat = self.stat.clone();
    //     let cap = (w + h) * 2 - 4;
    //     let mut all = Vec::with_capacity(w * h);
    //     let mut next = Vec::with_capacity(cap);
    //     let mut current = Vec::with_capacity(cap);
    //
    //     // tag start location
    //     if !is_tag(&stat, start_idx) {
    //         set_tag(&mut stat, start_idx);
    //         all.push(start_pos);
    //     }
    //     current.push(start_pos);
    //
    //     // 递归扩散，遍历所有相邻单位 TODO 死循环，待修
    //     while !current.is_empty() {
    //         for p in &current {
    //             for a in p.get_around() {
    //                 if let Some(f) = self.get_index_by_loc(a) {
    //                     if is_tag(&stat, f) {
    //                         continue;
    //                     }
    //                     all.push(a);
    //                     next.push(a);
    //                     set_tag(&mut stat, f);
    //                 }
    //             }
    //         }
    //         current.clear();
    //         current.append(&mut next);
    //     }
    //     Ok(all)
    // }
}
