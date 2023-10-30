

use crate::{cell::Cell, location::Loc};
use rand::seq::SliceRandom;
use rand::thread_rng;

// const MAX_LEN: usize = 255 * 255;
// const MAX_LEN_STAT: usize = MAX_LEN / 8;
//
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
        let cap = if count == 0 || cap < count as usize { 0 } else { cap };
        Self {
            count,
            width,
            height,
            map: vec![0; cap],
            // stat: vec![0; MAX_LEN_STAT],
        }
    }

    #[inline]
    fn get_index(&self, x: u8, y: u8) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y as usize) * (self.width as usize) + (x as usize))
        } else {
            None
        }
    }

    #[allow(dead_code)]
    #[inline]
    fn get_index_by_loc(&self, Loc(x, y): Loc) -> Option<usize> {
        self.get_index(x, y)
    }

    #[inline]
    pub fn get(&self, x: u8, y: u8) -> Option<Cell> {
        Some(Cell(*self.map.get(self.get_index(x, y)?)?))
    }

    #[inline]
    pub fn get_by_loc(&self, Loc(x, y): Loc) -> Option<Cell> {
        self.get(x, y)
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
                let (xu8, yu8) = (x as u8, y as u8);
                let Some(c) = self.get(xu8, yu8) else {
                    panic!("invalid location !!! ({x}, {y}) !!!")
                };
                if !c.is_mine() {
                    continue;
                }
                let l = Loc(xu8, yu8);
                for al in l.get_around() {
                    if let Some(av) = self.get_mut_by_loc(al) {
                        *av += 1;
                    }
                }
            }
        }
    }

    pub fn format_str(&self) -> String {
        let (w, h) = (self.width as usize, self.height as usize);
        let mut buf = String::with_capacity(w * 2 * h + h);
        use std::fmt::Write;
        for y in 0..self.height {
            for x in 0..self.width {
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
    fn get_mut(&mut self, x: u8, y: u8) -> Option<&mut u8> {
        if x < self.width && y < self.height {
            Some(self.map.get_mut(y as usize * self.width as usize + x as usize)?)
        } else {
            None
        }
    }

    // private
    #[inline]
    fn get_mut_by_loc(&mut self, Loc(x, y): Loc) -> Option<&mut u8> {
        self.get_mut(x, y)
    }

    // fn set_tag(&mut self, x: u8, y: u8) {
    //     if let Some(f) = self.get_index(x, y) {
    //         set_tag(&mut self.stat, f);
    //     }
    // }

    pub fn open(&mut self, x: u8, y: u8) {
        // let mut b = false;
        if let Some(v) = self.get_mut(x, y) {
            let mut c = Cell(*v);
            if !c.is_open() {
                c.switch_open();
                *v = c.0;
                // b = true;
            }
        }
        // if b {
        //     self.set_tag(x, y);
        // }
    }

    pub fn open_by_loc(&mut self, Loc(x, y): Loc) {
        self.open(x, y)
    }

    pub fn switch_flag(&mut self, x: u8, y: u8) {
        // let mut b = false;
        if let Some(v) = self.get_mut(x, y) {
            let mut c = Cell(*v);
            c.switch_flag();
            *v = c.0;
            // b = true;
        }
        // if b {
        //     self.set_tag(x, y);
        // }
    }

    pub fn switch_flag_by_loc(&mut self, Loc(x, y): Loc) {
        self.switch_flag(x, y)
    }

    // pub fn get_nearby_empty_area(&self, x: u8, y: u8) -> Vec<Loc> {
    //     if x >= self.width || y >= self.height {
    //         return Vec::with_capacity(0);
    //     }
    //     let mut stat = self.stat.clone();
    //     let (w, h) = (self.width as usize, self.height as usize);
    //     let mut all = Vec::with_capacity(w * h);
    //     let tmp = (w + h) * 2 - 4;
    //     let mut next = Vec::with_capacity(tmp);
    //     let mut current = Vec::with_capacity(tmp);
    //
    //     let start_pos = Loc(x, y);
    //     let start_idx = self.get_index_by_loc(start_pos).unwrap();
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
    //     all
    // }
}
