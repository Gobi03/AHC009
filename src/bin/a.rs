#[allow(unused_imports)]
use proconio::marker::{Chars, Isize1, Usize1};
use proconio::{fastout, input};

#[allow(unused_imports)]
use std::cmp::*;
#[allow(unused_imports)]
use std::collections::*;

#[allow(unused_imports)]
use rand::rngs::ThreadRng;
#[allow(unused_imports)]
use rand::seq::SliceRandom;
#[allow(unused_imports)]
use rand::{thread_rng, Rng};
#[allow(unused_imports)]
use std::io::Write;
use std::time::SystemTime;

#[allow(dead_code)]
const SIDE: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}
impl Direction {
    fn to_delta(&self) -> Coord {
        match *self {
            Self::Left => Coord::new((-1, 0)),
            Self::Right => Coord::new((1, 0)),
            Self::Up => Coord::new((0, -1)),
            Self::Down => Coord::new((0, 1)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Coord {
    x: isize,
    y: isize,
}
#[allow(dead_code)]
impl Coord {
    fn new(p: (isize, isize)) -> Self {
        Coord { x: p.0, y: p.1 }
    }
    fn from_usize_pair(p: (usize, usize)) -> Self {
        Coord {
            x: p.0 as isize,
            y: p.1 as isize,
        }
    }

    fn in_field(&self) -> bool {
        (0 <= self.x && self.x < SIDE as isize) && (0 <= self.y && self.y < SIDE as isize)
    }

    // ペアへの変換
    fn to_pair(&self) -> (isize, isize) {
        (self.x, self.y)
    }

    // マンハッタン距離
    fn distance(&self, that: &Self) -> isize {
        (self.x - that.x).abs() + (self.y - that.y).abs()
    }

    fn mk_4dir(&self) -> Vec<Self> {
        let delta = [(-1, 0), (1, 0), (0, -1), (0, 1)];

        delta
            .iter()
            .map(|&p| self.plus(&Coord::new(p)))
            .filter(|&pos| pos.in_field())
            .collect()
    }

    fn com_to_delta(com: char) -> Self {
        match com {
            'U' => Coord::new((0, -1)),
            'D' => Coord::new((0, 1)),
            'L' => Coord::new((-1, 0)),
            'R' => Coord::new((1, 0)),
            _ => unreachable!(),
        }
    }

    // 四則演算
    fn plus(&self, that: &Self) -> Self {
        Coord::new((self.x + that.x, self.y + that.y))
    }
    fn minus(&self, that: &Self) -> Self {
        Coord::new((self.x - that.x, self.y - that.y))
    }

    fn access_matrix<'a, T>(&'a self, mat: &'a Vec<Vec<T>>) -> &'a T {
        &mat[self.y as usize][self.x as usize]
    }

    fn set_matrix<T>(&self, mat: &mut Vec<Vec<T>>, e: T) {
        mat[self.y as usize][self.x as usize] = e;
    }
}

#[allow(dead_code)]
struct Input {
    start: Coord,
    goal: Coord,
    p: f64,
    yoko: Vec<Vec<bool>>,
    tate: Vec<Vec<bool>>,
}
impl Input {
    fn new(start: Coord, goal: Coord, p: f64, hs: Vec<Vec<char>>, vs: Vec<Vec<char>>) -> Self {
        let yoko = hs
            .into_iter()
            .map(|h| h.into_iter().map(|a| a == '1').collect())
            .collect();
        let tate = vs
            .into_iter()
            .map(|v| v.into_iter().map(|a| a == '1').collect())
            .collect();

        Self {
            start,
            goal,
            p,
            yoko,
            tate,
        }
    }

    fn can_move(&self, pos: Coord, to_dir: Direction) -> bool {
        use Direction::*;

        let to = pos.plus(&to_dir.to_delta());

        if !to.in_field() {
            return false;
        }
        match to_dir {
            Left => self.yoko[pos.y as usize][pos.x as usize - 1],
            Right => self.yoko[pos.y as usize][pos.x as usize],
            Up => self.tate[pos.y as usize - 1][pos.x as usize],
            Down => self.tate[pos.y as usize][pos.x as usize],
        }
    }
}

// -> (スコア, , 移動位置の期待値のテーブル)
pub fn compute_score(input: &Input, out: &[char]) -> (i64, String, Vec<Vec<f64>>) {
    // 移動位置の期待値のテーブル
    let mut crt = mat![0.0; N; N];
    crt[input.s.0][input.s.1] = 1.0;

    let mut sum = 0.0; // スコア計算用のE[S]の値
    let mut goal = 0.0;

    // ターン数でループ
    for t in 0..out.len() {
        if t >= L {
            return (0, "too long output".to_owned(), crt);
        }

        // dは移動コマンドに相当
        if let Some(d) = DIR.iter().position(|&c| c == out[t]) {
            let mut next = mat![0.0; N; N];
            // セルごとに移動後の期待値を算出
            for i in 0..N {
                for j in 0..N {
                    if crt[i][j] > 0.0 {
                        if input.can_move(i, j, d) {
                            let i2 = i + DIJ[d].0;
                            let j2 = j + DIJ[d].1;
                            next[i2][j2] += crt[i][j] * (1.0 - input.p);
                            next[i][j] += crt[i][j] * input.p;
                        } else {
                            next[i][j] += crt[i][j];
                        }
                    }
                }
            }
            crt = next;
            sum += crt[input.t.0][input.t.1] * (2 * L - t) as f64;
            goal += crt[input.t.0][input.t.1];
            // ゴールすると固定されるので、除外
            crt[input.t.0][input.t.1] = 0.0;
        } else {
            return (0, format!("illegal char: {}", out[t]), crt);
        }
    }

    // goalは不動なので、ループ終了後に処理
    crt[input.t.0][input.t.1] = goal;

    (
        (1e8 * sum / (2 * L) as f64).round() as i64, // スコア
        String::new(),
        crt, // 移動位置の期待値のテーブル
    )
}

#[fastout]
fn main() {
    let system_time = SystemTime::now();
    let mut _rng = thread_rng();

    input! {
        si: usize,
        sj: usize,
        gi: usize,
        gj: usize,

        p: f64,

        h: [Chars; SIDE],
        v: [Chars; SIDE - 1],
    }

    let sp = Coord::from_usize_pair((sj, si));
    let gp = Coord::from_usize_pair((gj, gi));

    let input = Input::new(sp, gp, p, h, v);

    let mut ans = "".to_string();
    for _ in 0..20 {
        ans = format!("{}{}", ans, "DRDRURDRDL");
    }

    println!("{}", ans);

    eprintln!("{}ms", system_time.elapsed().unwrap().as_millis());
}
