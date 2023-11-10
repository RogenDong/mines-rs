use crate::{cell::Cell, location::Loc};
use rand::seq::SliceRandom;
use rand::thread_rng;

const MAX_LEN: usize = 255 * 255;
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
//
// fn set_tag(map: &mut Vec<u8>, f: usize) {
//     if f == 0 {
//         map[0] |= 1;
//     } else if f < MAX_LEN {
//         map[f / 8] |= 1 << (f % 8);
//     }
// }

#[inline]
pub fn get_idx(x: usize, y: usize, w: usize, h: usize) -> Option<usize> {
    if x < w && y < h {
        Some(y * w + x)
    } else {
        None
    }
}

#[allow(non_snake_case)]
pub fn get_bmp_idx(x: usize, y: usize, w: usize, h: usize) -> Option<[usize; 9]> {
    const M: usize = usize::MAX - 1;
    let mut i = get_idx(x, y, w, h)?;
    if i == 0 {
        return Some([M, M, M, M, M, 1, M, w, h + 1]);
    }
    let lim = w * h;
    if i >= lim {
        i = M;
    }
    let (iN, iW, iE, iS) = (
        if i < w { M } else { i - w },
        i - 1,
        i + 1,
        if i >= lim { M } else { i + w },
    );
    if i == MAX_LEN - 1 {
        return Some([iN - 1, iN, M, iW, M, M, M, M, M]);
    }
    //  A N B | A=N-1 N=i-w B=N+1
    //  W   E | W=i-1       E=i+1
    //  C S D | C=S-1 S=i+w D=S+1
    const N: usize = 1;
    const W: usize = 3;
    const E: usize = 5;
    const S: usize = 7;
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
    if x == 0 {
        ls[W] = M;
        ls[N - 1] = M;
        ls[S - 1] = M;
    } else if x == w - 1 {
        ls[E] = M;
        ls[N + 1] = M;
        ls[S + 1] = M;
    }
    if y == 0 {
        ls[N] = M;
        ls[N - 1] = M;
        ls[N + 1] = M;
    } else if y == h - 1 {
        ls[S] = M;
        ls[S - 1] = M;
        ls[S + 1] = M;
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

    // 刷新地雷
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
        loop {
            self.map.shuffle(&mut rng);
            if let Some(c) = ignore.and_then(|l| self.get_by_loc(l)) {
                if !c.is_mine() {
                    break;
                }
            } else {
                break;
            }
        }
        for y in 0..h {
            for x in 0..w {
                if let Some(i) = get_idx(x, y, w, h) {
                    if self.map[i] > 8 {
                        if let Some(ls) = get_bmp_idx(x, y, w, h) {
                            for ii in ls {
                                if ii < map_size {
                                    *self.map.get_mut(ii).unwrap() += 1;
                                }
                            }
                        } // get around
                    } // found mine
                } // get index
            } // loop column
        } // loop row
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
            let mut c = Cell(*v);
            if !c.is_open() {
                c.switch_open();
                *v = c.0;
                let _ = v; // drop *mut
                           // self.set_tag(x, y);
            }
        }
    }

    pub fn open_by_loc(&mut self, Loc(x, y): Loc) {
        self.open(x as usize, y as usize)
    }

    pub fn switch_flag(&mut self, x: usize, y: usize) {
        if let Some(v) = self.get_mut(x, y) {
            let mut c = Cell(*v);
            c.switch_flag();
            *v = c.0;
            let _ = v; // drop *mut
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
