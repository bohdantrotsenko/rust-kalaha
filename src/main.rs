#![allow(dead_code)]

type Player = [i8; 7];

#[derive(Clone, Debug)]
struct Game {
    p: [Player; 2],
    t: i8,
}

fn new_player() -> Player {
    [4,4,4,4,4,4,0]
}

#[derive(Debug, PartialEq)]
enum State {
    InProgress,
    Draw,
    Win(i8),
}

impl Game {
    fn new() -> Game {
        Game{p: [new_player(), new_player()], t: 0}
    }

    fn print(&self) {
        print!("|{:2}|", self.p[1][6]);
        for i in 0..6 {
            print!(" [{:2}]", self.p[1][5 - i])
        }
        print!(" /--\\");
        if self.t == 1 {
            print!(" <-- turn");
        }
        println!();
        print!("\\--/ ");
        for i in 0..6 {
            print!("[{:2}] ", self.p[0][i])
        }
        print!("|{:2}|", self.p[0][6]);
        if self.t == 0 {
            print!(" <-- turn");
        }
        println!();
    }

    fn state(&self) -> State {
        let p0full: i8 = self.p[0].iter().sum();
        let p1full: i8 = self.p[1].iter().sum();
        let p0 = p0full - self.p[0][6];
        let p1 = p1full - self.p[1][6];
        if p0 != 0 && p1 != 0 {
            State::InProgress
        } else {
            if p0full < p1full {
                State::Win(1)
            } else if p0full > p1full {
                State::Win(0)
            } else {
                State::Draw
            }
        }
    }

    fn step(&self, s: usize) -> Option<Game> {
        if s >= 6 { panic!("wrong start index"); }
        let mut n: Game = self.clone();
        let mut leftover = n.p[n.t as usize][s];
        if leftover == 0 { return None; }

        let mut side: usize = n.t as usize;
        let mut pos: usize = s + 1;
        n.p[side][s] = 0;

        loop {
            n.p[side][pos] += 1;
            leftover -= 1;
            if leftover == 0 { break; }
            if (pos == 5 && side != self.t as usize) || (pos == 6) {
                // pos == 6 => we have reached the super bucket and have to switch sides
                // pos == 5 => we have reached opponents 0 bucket and have to return home
                side = 1 - side;
                pos = 0;
            } else {
                pos += 1;
            }
        }

        if pos < 6 { // if we have reached the super bucket, we don't give up turn
            // check for take-over
            if side == self.t as usize && n.p[side][pos] == 1 {
                let opp = 5 - pos;
                n.p[side][6] += n.p[1 - side][opp];
                n.p[1 - side][opp] = 0;
            }
            n.t = 1 - n.t; // switch the turn
        }
        Some(n)
    }
}

// next computes all the posible games for the next turn
fn next(g: &Game) -> Vec<Game> {
    let mut res: Vec<Game> = Vec::new();
    let turn = g.t;
    for i in 0..6 {
        if let Some(cand) = g.step(i) {
            if cand.t == turn && cand.state() == State::InProgress {
                for elem in next(&cand).into_iter() {
                    res.push(elem);
                }
            } else {
                res.push(cand);
            }
        }
    }
    res
}

fn main() {
    let mut g = Game::new();
    //g.p[0][0] = 6;
    g.print();
    println!("{:?}", g.state());
    //g.step(0).unwrap().print();
    for t in next(&g).into_iter() {
        t.print();
    }
}