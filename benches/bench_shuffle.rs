#![allow(unused)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mines::{cell::Cell, location::Loc, mmap::MineMap};
use rand::{seq::SliceRandom, thread_rng, Rng};
use smallvec::SmallVec;
use std::fmt::Write;

const WIDTH: usize = 255;
const HEIGHT: usize = 255;
const MAX_LEN: usize = WIDTH * HEIGHT;
const COUNT_MINES_TS: usize = MAX_LEN / 8;
// 表示无效下标。减1是为了后续增减操作不发生溢出。
const M: usize = usize::MAX - 1;
const INVALID_AROUND: [usize; 8] = [M, M, M, M, M, M, M, M];

fn ts_fill(cri: &mut Criterion) {
    let mut v = vec![123; MAX_LEN];
    cri.bench_function("fill", |b| {
        b.iter(|| {
            v.fill(black_box(0));
            v[..COUNT_MINES_TS].fill(black_box(9));
        })
    });
}

fn ts_random_shuffle(cri: &mut Criterion) {
    let mut rng = thread_rng();
    let mut v = vec![123; MAX_LEN];
    v.fill(0);
    v[..COUNT_MINES_TS].fill(9);

    cri.bench_function("rand_shuffle", |b| b.iter(|| v.shuffle(&mut rng)));
}

fn ts_bmp_warn_by_index(cri: &mut Criterion) {
    let mut rng = thread_rng();
    let mut map = vec![123u8; MAX_LEN];
    map.fill(0);
    map[..COUNT_MINES_TS].fill(9);
    map.shuffle(&mut rng);

    cri.bench_function("bmp_by_index", |b| b.iter(|| bmp_warm(&mut map)));
}

fn ts_uncover(cri: &mut Criterion) {
    let mut rng = thread_rng();
    let mut map = vec![0; MAX_LEN];
    map[..COUNT_MINES_TS].fill(9);
    map.shuffle(&mut rng);
    ignore(&mut map, 200, 200);
    bmp_warm(&mut map);

    cri.bench_function("uncover", |b| {
        b.iter(|| {
            // 找个空白点
            let a = loop {
                let t = rng.gen_range(0..MAX_LEN);
                if map[t] < 1 {
                    break t;
                }
            };
            let _ = black_box(uncover_empty_region(a, &map));
        })
    });
}

criterion_group!(
    benches, // ts_fill,
    // ts_random_shuffle,
    // ts_bmp_warn_by_index,
    ts_uncover,
);
// criterion_group!(benches, ts_emp_area);
criterion_main!(benches);

fn ignore(map: &mut Vec<u8>, x: usize, y: usize) {
    let mut rng = thread_rng();
    let around = get_around_index_by_loc(x, y);
    for &a in &around {
        match map.get_mut(a) {
            Some(c @ 9) => *c = 0,
            _ => continue,
        }
        loop {
            let r = rng.gen_range(0..MAX_LEN);
            if around.iter().any(|&o| o == r) {
                continue;
            }
            if let Some(c @ 0) = map.get_mut(r) {
                *c = 9;
                break;
            }
        }
    }
}

fn bmp_warm(map: &mut Vec<u8>) {
    for i in 0..MAX_LEN {
        if map[i] > 8 {
            for a in get_around_index(i) {
                if a < MAX_LEN {
                    map[a] += 1;
                }
            }
        }
    }
}

#[inline]
pub fn get_idx(x: usize, y: usize) -> Option<usize> {
    if x < WIDTH && y < HEIGHT {
        Some(y * WIDTH + x)
    } else {
        None
    }
}

/// 下标转坐标
fn idx_to_loc(i: usize) -> Option<(usize, usize)> {
    let s = WIDTH * HEIGHT;
    if i >= s {
        return None;
    }
    Some(if i == 0 {
        (0, 0)
    } else if i == WIDTH {
        (0, 1)
    } else if i < WIDTH {
        (i, 0)
    } else if i == s - 1 {
        (WIDTH - 1, HEIGHT - 1)
    } else {
        let x = i % WIDTH;
        (x, (i - x) / WIDTH)
    })
}

/// 4个拐角点的周围点的下标偏移量
#[inline]
fn corner(i: usize) -> Option<[usize; 8]> {
    const M: usize = usize::MAX - 1;
    // 周围一圈下标在集合中的顺序
    // D,N,A = 7,0,1
    // W,_,E = 6,_,2
    // C,S,B = 5,4,3
    Some(if i == 0 {
        // 左上=[_,_,E,B,S,..]
        [M, M, i + 1, (i + WIDTH + 1), (i + WIDTH), M, M, M]
    } else if i == WIDTH - 1 {
        // 右上=[..,S,C,W,_]
        [M, M, M, M, (i + WIDTH), (i + WIDTH - 1), M, M]
    } else if i == MAX_LEN - WIDTH {
        // 左下=[N,A,E,..]
        [(i - WIDTH), (i - WIDTH + 1), (i + 1), M, M, M, M, M]
    } else if i == MAX_LEN - 1 {
        // 右下=[N,..,W,D]
        [(i - WIDTH), M, M, M, M, M, (i - 1), (i - WIDTH - 1)]
    } else {
        return None;
    })
}

/// 周围点的下标偏移量
#[inline]
fn get_around(wi: usize, ni: usize, ei: usize, si: usize) -> [usize; 8] {
    // 周围一圈下标在集合中的顺序
    // D,N,A = 7,0,1
    // W,_,E = 6,_,2
    // C,S,B = 5,4,3
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

/// 基于长宽和二维坐标收集并返回周围单位的下标
/// # Return
/// - 需要自增的位置是有效下标
/// - 不自增的位置用大于地图最大长度的值表示
fn get_around_index(i: usize) -> [usize; 8] {
    if i >= MAX_LEN {
        return INVALID_AROUND;
    }
    if MAX_LEN == 4 {
        return corner(i).unwrap();
    }
    if let Some(ls) = corner(i) {
        return ls;
    }
    let (x, y) = idx_to_loc(i).unwrap();
    // 根据x,y是否贴边计算四周下标偏移
    get_around(
        if x > 0 { i - 1 } else { M },
        if y > 0 { i - WIDTH } else { M },
        if x < WIDTH - 1 { i + 1 } else { M },
        if y < HEIGHT - 1 { i + WIDTH } else { M },
    )
}

/// 基于长宽和二维坐标收集并返回周围单位的下标
/// # Return
/// - 需要自增的位置是有效下标
/// - 不自增的位置用大于地图最大长度的值表示
fn get_around_index_by_loc(x: usize, y: usize) -> [usize; 8] {
    let Some(i) = get_idx(x, y) else {
        return INVALID_AROUND;
    };
    if i >= MAX_LEN {
        return INVALID_AROUND;
    }
    if MAX_LEN == 4 {
        return corner(i).unwrap();
    }
    if let Some(ls) = corner(i) {
        return ls;
    }
    // 根据x,y是否贴边计算四周下标偏移
    get_around(
        if x > 0 { i - 1 } else { M },
        if y > 0 { i - WIDTH } else { M },
        if x < WIDTH - 1 { i + 1 } else { M },
        if y < HEIGHT - 1 { i + WIDTH } else { M },
    )
}

fn uncover_empty_region(s: usize, map: &Vec<u8>) -> Vec<usize> {
    // 结果集
    let mut result = Vec::with_capacity(MAX_LEN - 2);
    // 本轮待检查的下标集
    let mut current = Vec::with_capacity(MAX_LEN - 2);
    // 暂存下一轮数据
    let mut next = Vec::with_capacity(MAX_LEN - 2);
    // 已访问的下标
    let mut vis = Vec::with_capacity(MAX_LEN - 2);
    // 起点直接加入结果集、已访集
    vis.push(s);
    result.push(s);
    // 获取起点周围的下标，作为首轮待检查下标
    current.extend(get_around_index(s).into_iter().filter(|a| *a < MAX_LEN));

    // 层层递推检查下标，找到所有可连接的空白。
    loop {
        for &i in &current {
            if vis.contains(&i) {
                continue;
            }
            vis.push(i);
            let v = map[i];
            if v > 0 {
                // 遇到数字时该下标收集入结果集，不寻找其周围下标。
                if v < 9 {
                    result.push(i);
                }
                continue;
            }
            next.extend(
                get_around_index(i)
                    .into_iter()
                    .filter(|a| *a < MAX_LEN && !vis.contains(a)),
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
