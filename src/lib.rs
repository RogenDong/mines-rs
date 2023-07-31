pub mod mark;
pub mod position;

#[cfg(test)]
mod tss;

use mark::Mark;
use position::Position;
use rand::Rng;

#[derive(Clone, Copy)]
enum SlotState {
    Sth,
    Empty,
    Tagged,
}

pub struct MineMap {
    pub count: u8,
    pub width: u8,
    pub height: u8,
    map: Box<[[Mark; 256]; 256]>,
    state_map: Box<[[SlotState; 256]; 256]>,
}
impl MineMap {
    fn clean_up(&mut self) {
        self.map = Box::new([[Mark(0); 256]; 256]);
        self.state_map = Box::new([[SlotState::Empty; 256]; 256]);
    }

    pub fn from(count: u8, width: u8, height: u8) -> Self {
        Self {
            count,
            width,
            height,
            map: Box::new([[Mark(0); 256]; 256]),
            state_map: Box::new([[SlotState::Empty; 256]; 256]),
        }
    }

    pub fn get(&self, x: u8, y: u8) -> &Mark {
        &self.map[y as usize][x as usize]
    }

    pub fn get_by_pos(&self, Position(x, y): Position) -> &Mark {
        self.get(x, y)
    }

    // fn get_mut(&mut self, x: u8, y: u8) -> &mut Mark {
    //     &mut self.map[y as usize][x as usize]
    // }

    #[inline]
    fn get_mut_by_pos(&mut self, Position(x, y): Position) -> &mut Mark {
        &mut self.map[y as usize][x as usize]
    }

    #[inline]
    fn set_slot_state(&mut self, Position(x, y): Position, st: SlotState) {
        self.state_map[y as usize][x as usize] = st;
    }

    #[inline]
    fn get_slot_state(&self, Position(x, y): Position) -> SlotState {
        self.state_map[y as usize][x as usize]
    }

    /// 预览刷新的地雷
    fn preview_shuffle(&self) -> Vec<Position> {
        let (w, h) = (self.width as usize, self.height as usize);
        if self.count < 1 || w * h < self.count as usize {
            return Vec::with_capacity(0);
        }
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
        let ls_pv_mine = self.preview_shuffle();
        if ls_pv_mine.is_empty() {
            return;
        }
        self.clean_up();
        let limit = Position(self.width, self.height);
        for p in ls_pv_mine {
            self.get_mut_by_pos(p).set_mine();
            self.set_slot_state(p, SlotState::Sth);
            for a in p.get_around(limit) {
                self.get_mut_by_pos(a).bump_warn(true);
                self.set_slot_state(a, SlotState::Sth);
            } // for around mine
        } // for mines
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
    pub fn update_guess(&mut self) {
        todo!()
    }

    fn set_mine(&mut self, p: Position) {
        for p in p.get_around(Position(self.width, self.height)) {
            let m = self.get_mut_by_pos(p);
            if !m.is_mine() {
                m.bump_warn(true);
                self.set_slot_state(p, SlotState::Sth);
            }
        }
        self.get_mut_by_pos(p).set_mine();
        self.set_slot_state(p, SlotState::Sth);
    }

    /// 全图随机抽一个点，设置地雷
    fn try_random_one(&mut self) -> Position {
        let mut rng = rand::thread_rng();
        let p = loop {
            let y = rng.gen_range(0..self.height);
            let x = rng.gen_range(0..self.width);
            let m = self.get(x, y);
            if !m.is_open() && !m.is_mine() {
                break Position(x, y);
            }
        };
        self.set_mine(p);
        p
    }

    // 在指定队列中抽一个点
    // TODO: 完善基础 API 后添加【猜测】相关 API
    // fn try_random_in(&mut self, ls: Vec<Position>) {
    //     todo!()
    // }

    /// 移动地雷（随机）
    /// ### Arguments
    /// `p` 原位置  
    /// ### Return
    /// 移动后位置
    pub fn move_mine_randomly(&mut self, x: u8, y: u8) -> Position {
        let p = Position(x, y);
        let mut ls_around = Vec::with_capacity(8);
        // count around mines
        let mut w = 0;
        for ap in p.get_around(Position(self.width, self.height)) {
            let am = self.get_mut_by_pos(ap);
            if am.is_mine() {
                w += 1;
                continue;
            }
            // cache around that not open
            if !am.is_open() {
                ls_around.push(ap);
            }
            // update around warn
            let bw = am.bump_warn(false);
            if bw < 1 {
                self.set_slot_state(ap, SlotState::Empty);
            }
        }
        // remove mine
        self.get_mut_by_pos(p).set_warn(w);
        if w < 1 {
            self.set_slot_state(p, SlotState::Empty);
        }

        let len = ls_around.len();
        // 1. select from all position
        if len < 1 {
            return self.try_random_one();
        }

        // 2. select from around
        let p = ls_around[rand::thread_rng().gen_range(0..len)];
        self.set_mine(p);
        p
    }

    /// 获取指定位置周围所有彼此接触的空位的坐标
    pub fn get_nearby_empty_slots(&mut self, x: u8, y: u8) -> Vec<Position> {
        if x >= self.width || y >= self.height {
            return Vec::with_capacity(0);
        }
        let (w, h) = (self.width as usize, self.height as usize);
        let mut all = Vec::with_capacity(w * h);
        let mut next = Vec::with_capacity((w + h) * 2 - 4);
        let mut current = Vec::with_capacity((w + h) * 2 - 4);
        
        let start_pos = Position(x, y);
        // tag start position
        if let SlotState::Empty = self.get_slot_state(start_pos) {
            self.set_slot_state(start_pos, SlotState::Tagged);
            all.push(start_pos);
        }
        current.push(start_pos);
        let lim = Position(self.width, self.height);
        // traverse around, collect empty slots.
        loop {
            for p in current.iter() {
                for a in p.get_around(lim) {
                    let SlotState::Empty = self.get_slot_state(a) else {continue};
                    self.set_slot_state(a, SlotState::Tagged);
                    next.push(a);
                    all.push(a);
                }
            }
            if next.is_empty() {
                break;
            }
            current.clear();
            current.append(&mut next);
            next.clear();
        }
        // reset state
        if let SlotState::Tagged = self.get_slot_state(start_pos) {
            self.set_slot_state(start_pos, SlotState::Empty);
        }
        for p in all.iter() {
            if let SlotState::Tagged = self.get_slot_state(*p) {
                self.set_slot_state(*p, SlotState::Empty)
            }
        }
        all
    }
}
