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
const MAX_TURN: usize = 200;

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

    fn to_char(&self) -> char {
        use Direction::*;
        match self {
            Left => 'L',
            Right => 'R',
            Up => 'U',
            Down => 'D',
        }
    }

    fn list() -> Vec<Self> {
        use Direction::*;
        vec![Left, Up, Right, Down]
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
#[derive(Debug, Clone)]
struct Input {
    start: Coord,
    goal: Coord,
    p: f64,
    yoko: Vec<Vec<bool>>,
    tate: Vec<Vec<bool>>,

    dist_table: Vec<Vec<usize>>,
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

        let mut input = Self {
            start,
            goal,
            p,
            yoko,
            tate,
            dist_table: vec![],
        };

        let mut dist_table = mat![std::usize::MAX; SIDE; SIDE];
        let mut q = VecDeque::new();
        q.push_back((goal.clone(), 0));
        goal.set_matrix(&mut dist_table, 0);
        while !q.is_empty() {
            let (pos, dist) = q.pop_front().unwrap();
            for dir in Direction::list() {
                if input.can_move(&pos, &dir) {
                    let delta = pos.plus(&dir.to_delta());

                    if delta.in_field() && *delta.access_matrix(&dist_table) == std::usize::MAX {
                        delta.set_matrix(&mut dist_table, dist + 1);
                        q.push_back((delta, dist + 1));
                    }
                }
            }
        }

        input.dist_table = dist_table;
        input
    }

    fn can_move(&self, pos: &Coord, to_dir: &Direction) -> bool {
        use Direction::*;

        let to = pos.plus(&to_dir.to_delta());

        if !to.in_field() {
            return false;
        }
        match to_dir {
            Left => !self.yoko[pos.y as usize][pos.x as usize - 1],
            Right => !self.yoko[pos.y as usize][pos.x as usize],
            Up => !self.tate[pos.y as usize - 1][pos.x as usize],
            Down => !self.tate[pos.y as usize][pos.x as usize],
        }
    }
}

#[derive(Debug, Clone)]
struct State {
    crt: Vec<Vec<f64>>, // 移動位置の期待値のテーブル
    turn: usize,
    sum: f64,
    goal_expected: f64,
    ans: Vec<Direction>,
}
impl State {
    fn new(input: &Input) -> Self {
        let mut crt = mat![0.0; SIDE; SIDE];
        input.start.set_matrix(&mut crt, 1.0);

        Self {
            crt,
            turn: 1,
            sum: 0.0,
            goal_expected: 0.0,
            ans: Vec::with_capacity(MAX_TURN),
        }
    }

    // 移動位置の期待値のテーブル を更新
    fn update_crt(&mut self, dir: &Direction, input: &Input) {
        input.goal.set_matrix(&mut self.crt, 0.0);

        let mut next = mat![0.0; SIDE; SIDE];
        // セルごとに移動後の期待値を算出
        for y in 0..SIDE {
            for x in 0..SIDE {
                if self.crt[y][x] > 0.0 {
                    let pos = Coord::from_usize_pair((x, y));
                    if input.can_move(&pos, dir) {
                        let pos2 = pos.plus(&dir.to_delta());

                        // 移動先
                        next[pos2.y as usize][pos2.x as usize] += self.crt[y][x] * (1.0 - input.p);
                        // 移動失敗分
                        next[y][x] += self.crt[y][x] * input.p;
                    } else {
                        next[y][x] += self.crt[y][x];
                    }
                }
            }
        }

        self.sum += input.goal.access_matrix(&next) * (401 - self.turn) as f64;

        self.goal_expected += input.goal.access_matrix(&next);
        input.goal.set_matrix(&mut next, self.goal_expected);

        self.turn += 1;
        self.crt = next;
        self.ans.push(dir.clone());
    }

    fn compute_score(&self) -> i64 {
        (1e8 * self.sum / (2 * MAX_TURN) as f64).round() as i64
    }

    fn print_ans(&self) {
        println!(
            "{}",
            self.ans.iter().map(|dir| dir.to_char()).collect::<String>()
        );
    }
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
    // eprintln!("{:?}", input);

    let mut st = State::new(&input);

    let dir_list = Direction::list();

    for turn in st.turn..=MAX_TURN {
        // 所在の期待値を係数にして、ゴールへのマンハッタン距離の増減を合算する。
        // 最も良い方向へ向かう

        let mut evals = vec![0.0; 4];

        let mut targets = vec![];
        for y in 0..SIDE {
            for x in 0..SIDE {
                let pos = Coord::from_usize_pair((x, y));
                if st.crt[y][x] > 0.0 && pos != input.goal {
                    targets.push(pos);
                }
            }
        }

        targets.sort_by(|a, b| {
            b.access_matrix(&st.crt)
                .partial_cmp(a.access_matrix(&st.crt))
                .unwrap()
        });
        // let pos = targets[0];
        //eprintln!("{:?}", pos);
        for pos in targets {
            for di in 0..4 {
                let dir = dir_list[di];
                let eval_p = if input.can_move(&pos, &dir) {
                    let pos2 = pos.plus(&dir.to_delta());
                    if *pos2.access_matrix(&input.dist_table)
                        < *pos.access_matrix(&input.dist_table)
                    {
                        *pos.access_matrix(&st.crt) * 10000000.0
                    } else {
                        -pos.access_matrix(&st.crt) * 10000000.0
                    }
                } else {
                    0.0
                };

                evals[di] += eval_p;
            }
        }

        // eprintln!("{:?}", evals);

        let mut com = dir_list[0];
        let mut eval_p = evals[0];
        for i in 1..4 {
            if eval_p < evals[i] {
                eval_p = evals[i];
                com = dir_list[i];
            }
        }

        st.update_crt(&com, &input);
    }

    // eprintln!("{:?}", st);
    // eprintln!("{}", st.compute_score());

    st.print_ans();

    eprintln!("{}ms", system_time.elapsed().unwrap().as_millis());
}

#[macro_export]
macro_rules! mat {
	($($e:expr),*) => { Vec::from(vec![$($e),*]) };
	($($e:expr,)*) => { Vec::from(vec![$($e),*]) };
	($e:expr; $d:expr) => { Vec::from(vec![$e; $d]) };
	($e:expr; $d:expr $(; $ds:expr)+) => { Vec::from(vec![mat![$e $(; $ds)*]; $d]) };
}
