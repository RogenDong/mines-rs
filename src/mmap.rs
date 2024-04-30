use crate::{cell::Cell, location::Loc};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

// 表示无效下标。减1是为了后续增减操作不发生溢出。
const M: usize = usize::MAX - 1;
const INVALID_AROUND: [usize; 8] = [M, M, M, M, M, M, M, M];

/// 基于长宽和二维坐标换算得到下标
/// * 地雷、数量等信息存储在简单数组中
#[inline]
fn loc_to_idx(x: usize, y: usize, w: usize, h: usize) -> Option<usize> {
    if x < w && y < h {
        Some(y * w + x)
    } else {
        None
    }
}

// /// 下标转坐标
// fn idx_to_loc(i: usize, w: usize, h: usize) -> Option<(usize, usize)> {
//     let s = w * h;
//     if i >= s {
//         return None;
//     }
//     Some(if i == 0 {
//         (0, 0)
//     } else if i == w {
//         (0, 1)
//     } else if i < w {
//         (i, 0)
//     } else if i == s - 1 {
//         (w - 1, h - 1)
//     } else {
//         let x = i % w;
//         (x, (i - x) / w)
//     })
// }

/// 基于长宽和二维坐标收集并返回周围单位的下标
/// # Return
/// - 需要自增的位置是有效下标
/// - 不自增的位置用大于地图最大长度的值表示
fn get_around_index(i: usize, w: usize, h: usize) -> [usize; 8] {
    let size = w * h;
    if i >= size {
        return INVALID_AROUND;
    }
    // 周围一圈下标在集合中的顺序
    // D,N,A = 7,0,1
    // W,_,E = 6,_,2
    // C,S,B = 5,4,3
    if i == 0 {
        // 左上=[_,_,E,B,S,..]
        return [M, M, (i + 1), (i + w + 1), (i + w), M, M, M];
    } else if i == w - 1 {
        // 右上=[..,S,C,W,_]
        return [M, M, M, M, (i + w), (i + w - 1), (i - 1), M];
    } else if i == size - w {
        // 左下=[N,A,E,..]
        return [(i - w), (i - w + 1), (i + 1), M, M, M, M, M];
    } else if i == size - 1 {
        // 右下=[N,..,W,D]
        return [(i - w), M, M, M, M, M, (i - 1), (i - w - 1)];
    }
    let t = i % w;
    // 根据x,y是否贴边计算四周下标偏移
    let (wi, ni, ei, si) = (
        if t < 1 { M } else { i - 1 },
        if i < w { M } else { i - w },
        if t + 1 == w { M } else { i + 1 },
        if i >= size - w { M } else { i + w },
    );
    if wi == M {
        // 表示当前贴左边，偏移=[N,A,E,B,S,..]
        [ni, (ni + 1), ei, (si + 1), si, M, M, M]
    } else if ei == M {
        // 表示当前贴右边，偏移=[N,..,S,C,W,D]
        [ni, M, M, M, si, (si - 1), wi, (ni - 1)]
    } else {
        // A,D的偏移可以通过N+-1获得；B,C的偏移同理。
        [ni, (ni + 1), ei, (si + 1), si, (si - 1), wi, (ni - 1)]
    }
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
    /// 导入布局和状态
    /// # Argument
    /// - map `[宽width, 高height, 数据data..]`
    /// - hold_stat 是否保留状态
    pub fn by(mut map: Vec<u8>, hold_stat: bool) -> Result<Self, String> {
        if map.len() < 6 {
            return Err("输入数据太短！[宽, 高, 数据..]".to_string());
        }
        let mut count = 0;
        if hold_stat {
            count = map.iter().skip(2).filter(|&v| v & 0x1f > 8).count() as u16;
        } else {
            for v in &mut map[2..] {
                *v &= 0x1f;
                if *v > 8 {
                    count += 1;
                }
            }
        }
        Ok(Self {
            map: map[2..].into(),
            height: map[1],
            width: map[0],
            count,
        })
    }

    pub fn new(count: u16, width: u8, height: u8) -> Result<Self, String> {
        if width < 2 || height < 2 {
            return Err("请设置更大的区域！".to_string());
        }
        if count < 1 {
            return Err("请设置更多地雷！".to_string());
        }
        let cap = width as usize * height as usize;
        if count as usize >= cap {
            return Err("请减少地雷数量！".to_string());
        }
        Ok(Self {
            count,
            width,
            height,
            map: vec![0; cap],
            // stat: vec![0; MAX_LEN_STAT],
        })
    }

    #[inline]
    fn my_size(&self) -> (usize, usize, usize) {
        let h = self.height as usize;
        let w = self.width as usize;
        (w, h, w * h)
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<Cell> {
        Some(Cell(*self.map.get(loc_to_idx(
            x,
            y,
            self.width as usize,
            self.height as usize,
        )?)?))
    }

    #[inline]
    pub fn get_by_loc(&self, Loc(x, y): Loc) -> Option<Cell> {
        self.get(x as usize, y as usize)
    }

    /// 刷新地雷
    fn shuffle(&mut self) {
        let size = self.width as usize * self.height as usize;
        let c = self.count as usize;
        if c == 0 || size < c {
            return;
        }
        // self.stat.fill(0);
        self.map.fill(0);
        self.map[..c].fill(9);
        // 用洗牌算法布置地雷
        let mut rng = thread_rng();
        self.map.shuffle(&mut rng);
        // TODO: 在结束前对洗牌结果添加一些评判
    }

    /// 设置安全区
    fn ignore_area(&mut self, ignore: Option<Loc>) {
        let Some(c) = ignore else { return };
        let (x, y, (w, h, size)) = (c.0 as usize, c.1 as usize, self.my_size());
        let Some(c) = loc_to_idx(x, y, w, h) else {
            return;
        };
        self.map[c] = 0;
        let mut rng = thread_rng();
        let area = get_around_index(c, w, h);
        for &a in &area {
            if a >= size || self.map[a] != 9 {
                continue;
            }
            self.map[a] = 0;
            loop {
                let i = rng.gen_range(0..size);
                if !area.contains(&i) && self.map[i] == 0 {
                    self.map[i] = 9;
                    break;
                }
            }
        }
    }

    pub fn new_game(&mut self, ignore: Option<Loc>) {
        self.shuffle();
        // 设置安全区
        self.ignore_area(ignore);
        // 设置地雷警示数值
        let (w, h, size) = self.my_size();
        for i in 0..size {
            if self.map[i] < 9 {
                continue;
            }
            // get around
            for a in get_around_index(i, w, h) {
                if a < size {
                    self.map[a] += 1;
                }
            }
        }
    }

    /// 重置进度：清除开关、标记状态
    pub fn reset_progress(&mut self) {
        for v in self.map.iter_mut() {
            *v &= 0x1f;
        }
    }

    pub fn format_str(&self) -> String {
        let (w, h, size) = self.my_size();
        let mut buf = String::with_capacity(size * 2 + h);
        use std::fmt::Write;
        let mut ln = 0;
        for i in 0..size {
            match Cell(self.map[i]).get_warn() {
                0 => buf.push_str("  "),
                v @ 1..=8 => write!(buf, " {v}").unwrap(),
                _ => buf.push_str(" ·"),
            }
            if ln < w - 1 {
                ln += 1;
            } else {
                buf.push('\n');
                ln = 0;
            }
        }
        buf
    }

    /// 找到空白区域
    fn uncover_empty_region(&self, i: usize) -> Vec<usize> {
        let (w, h, size) = self.my_size();
        // 结果集
        let mut result = Vec::with_capacity(size - 2);
        // 本轮待检查的下标集
        let mut current = Vec::with_capacity(size - 2);
        // 暂存下一轮数据
        let mut next = Vec::with_capacity(size - 2);
        // 已访问的下标
        let mut vis = Vec::with_capacity(size - 2);
        // 起点直接加入结果集、已访集
        vis.push(i);
        result.push(i);
        // 获取起点周围的下标，作为首轮待检查下标
        current.extend(get_around_index(i, w, h).into_iter().filter(|a| *a < size));

        // 层层递推检查下标，找到所有可连接的空白。
        loop {
            for &i in &current {
                if vis.contains(&i) {
                    continue;
                }
                vis.push(i);
                let v = self.map[i];
                if v > 0 {
                    // 遇到数字时该下标收集入结果集，不寻找其周围下标。
                    if v < 9 {
                        result.push(i);
                    }
                    continue;
                }
                next.extend(
                    get_around_index(i, w, h)
                        .into_iter()
                        .filter(|a| *a < size && !vis.contains(a)),
                );
                result.push(i);
            }
            // next为空集则结束递推。
            if next.is_empty() {
                break;
            }
            // 暂存每层收集到的待检下标，本轮结束时next导入到current。
            current.clear();
            current.append(&mut next);
        }
        result
    }

    /// 打开一片区域
    fn open_region(&mut self, i: usize) {
        let region = self.uncover_empty_region(i);
        for i in region {
            self.map[i] |= crate::cell::BIT_OPEN;
        }
        // TODO: 打开后需要反馈
    }

    /// 打开周围一圈
    pub fn open_around(&mut self, x: usize, y: usize) {
        let (w, h) = (self.width as usize, self.height as usize);
        let Some(i) = loc_to_idx(x, y, w, h) else {
            return;
        };
        let c = Cell(self.map[i]);
        if c.is_flag() {
            return;
        }
        for a in get_around_index(i, w, h) {
            let mut c = Cell(self.map[a]);
            if c.is_empty() {
                return self.open_region(i);
            }
            if !c.is_flag() {
                c.open();
                self.map[a] = c.0;
            }
        }
    }

    pub fn open(&mut self, x: usize, y: usize) {
        let Some(i) = loc_to_idx(x, y, self.width as usize, self.height as usize) else {
            return;
        };
        let mut c = Cell(self.map[i]);
        if c.is_flag() {
            return;
        }
        if c.is_empty() {
            return self.open_region(i);
        }
        c.switch_open();
        self.map[i] = c.0;
    }

    pub fn open_by_loc(&mut self, Loc(x, y): Loc) {
        self.open(x as usize, y as usize)
    }

    pub fn switch_flag(&mut self, x: usize, y: usize) {
        let Some(i) = loc_to_idx(x, y, self.width as usize, self.height as usize) else {
            return;
        };
        let mut c = Cell(self.map[i]);
        c.switch_flag();
        self.map[i] = c.0;
    }

    pub fn switch_flag_by_loc(&mut self, Loc(x, y): Loc) {
        self.switch_flag(x as usize, y as usize)
    }

    /// 导出布局数据
    /// # Argument
    /// - `hold_stat` 是否保留状态
    /// # Returns
    /// - `[宽width, 高height, 数据data..]`
    pub fn export(&self, hold_stat: bool) -> Vec<u8> {
        let mut res = Vec::with_capacity(self.map.len() + 2);
        res.push(self.width);
        res.push(self.height);
        if hold_stat {
            res.extend(&self.map);
        } else {
            res.extend(self.map.iter().map(|&v| v & 0x1f));
        }
        res
    }
}
