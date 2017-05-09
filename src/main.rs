#![allow(dead_code)]
extern crate rand;
use rand::Rng;
use std::collections::*;

type Player = [i8; 7];

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct Game {
    p: [Player; 2],
    t: i8,
}

fn new_player() -> Player {
    [4,4,4,4,4,4,0]
}

#[derive(Debug, PartialEq, Clone)]
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

    fn possible(&self) -> Vec<usize> {
        let side = self.t as usize;                
        let mut res: Vec<usize> = Vec::new();
        for i in 0..6 {
            if self.p[side][i] != 0 {
                res.push(i);
            }
        }
        res
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

fn show_first_row() {
    let g = Game::new();
    g.print();
    println!("{:?}", g.state());
    for t in next(&g).into_iter() {
        t.print();
    }
}

fn play_random_game() {
    let mut g = Game::new();
    let mut rng = rand::thread_rng();
    loop {
        println!("-----------------------------");
        println!("{:?}", g);
        g.print();
        let state = g.state();
        if state != State::InProgress {
            println!("{:?}", state);
            return;
        }
        
        let possibilities = next(&g);
        let n = possibilities.len();
        g = possibilities[rng.gen_range(0, n)].clone();
    }
}

fn find_outcome(g: &Game, cache: &mut HashMap<Game, (State, i8)>) -> (State, i8) {
    let gstate = g.state();
    if gstate != State::InProgress {
        return (gstate, -1);
    }
    if let Some(ans) = cache.get(&g) {
        return ans.clone();
    }

    let player = g.t;
    let mut best = State::Win(1 - player); // initialize the best with worst-case scenario -- winning of the other player
    let mut step: i8 = -1;

    for i in 0..6 {
        if let Some(ng) = g.step(i) {
            let outcome = find_outcome(&ng, cache);
            match outcome.0 {
                State::InProgress => panic!(),
                State::Draw => {
                    if best != State::Draw {
                        best = State::Draw;
                        step = i as i8;
                    }
                },
                State::Win(p) => {
                    if p == player { // that's the best outcome
                        best = outcome.0;
                        step = i as i8;
                        break; // no need to search further
                    } else {
                        // that's the worst outcome
                        if best != State::Draw {
                            step = i as i8; // only update the step
                        }
                    }
                }
            }
        }
    }

    debug_assert!(step >= 0, "best case must always be found");

    let ans = (best, step);
    cache.insert(g.clone(), ans.clone());
    ans
}

fn experiment_with_outcomes() {
    let mut g = Game { p: [[3, 0, 3, 0, 8, 2, 12], [0, 1, 3, 3, 1, 1, 11]], t: 0 };
    let mut cache: HashMap<Game, (State, i8)> = HashMap::new();
    let mut outcome = find_outcome(&g, &mut cache);
    println!("Outcome: {:?}", outcome);
    println!("Positions in cache: {}", cache.len());

    loop {
        g.print();
        if g.state() == State::InProgress {
            g = g.step(outcome.1 as usize).unwrap();
            outcome = find_outcome(&g, &mut cache);
        } else {
            break;
        }
    }
}

fn main() {
    //play_random_game();
    experiment_with_outcomes();
}
