#![allow(dead_code)]
extern crate rand;
extern crate num_cpus;

// see http://www.wikihow.com/Play-Kalaha for the description of the game

use rand::Rng;
use std::thread;
use std::sync::*;
use std::collections::*;

const LIMIT:usize = 500_000; // how many games should I play till I gather all the required data

type Player = [i8; 7]; // first 6 cells are regular cells, the last one is the super-cell

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
struct Game {
    p: [Player; 2],
    t: i8, // turn of the game; 0 <==> first player, 1 <==> second player
}

fn new_player() -> Player {
    [4,4,4,4,4,4,0] // the initial configuration in the classical game
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

    // state makes the analysis of the game at the moment
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

    // step makes the current player play turn s and returns the game's position
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

    // possible yields all the posible /moves/ for the position
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


fn find_outcome(g: &Game, cache: &mut HashMap<Game, State>, limit: &mut usize, known_wins: &HashSet<Game>, known_draws: &HashSet<Game>, khits: &mut usize) -> State {
    let gstate = g.state();
    if gstate != State::InProgress {
        return gstate;
    }
    if let Some(ans) = cache.get(&g) {
        return ans.clone();
    }
    if known_wins.contains(g) {
        *khits += 1;
        return State::Win(g.t);
    }    
    if known_draws.contains(g) {
        *khits += 1;
        return State::Draw;
    }    
    if *limit == 0 {
        return State::InProgress;        
    }
    *limit -= 1;

    let player = g.t;
    let mut best = State::Win(1 - player); // initialize the best with worst-case scenario -- winning of the other player

    for i in 0..6 {
        if let Some(ng) = g.step(i) {
            let outcome = find_outcome(&ng, cache, limit, known_wins, known_draws, khits);
            match outcome {
                State::InProgress => return State::InProgress, // we've reached the limit
                State::Draw => {
                    if best != State::Draw {
                        best = State::Draw;
                    }
                },
                State::Win(p) => {
                    if p == player { // that's the best outcome
                        best = outcome;
                        break; // no need to search further
                    } else {  // that's the worst outcome, nothing to do
                    }
                }
            }
        }
    }

    cache.insert(g.clone(), best.clone());
    best
}

fn get_knowledge(known_wins: &HashSet<Game>, known_draws: &HashSet<Game>) -> (Option<(Game, State)>, usize) {
    let mut rng = rand::thread_rng();
    let mut games = Vec::new();
    let mut g = Game::new();
    loop {
        //println!("-----------------------------");
        //println!("{:?}", g);
        //g.print();
        games.push(g);
        let state = g.state();
        if state != State::InProgress {
            // println!("{:?}", state);
            break;
        }
        
        let possibilities = next(&g);
        let n = possibilities.len();
        g = possibilities[rng.gen_range(0, n)].clone();
    }
    // println!("---  ---  ---  ---  ---  ---  ---  ---  ---  ---");
    let mut cache: HashMap<Game, State> = HashMap::with_capacity(LIMIT);
    let mut knowledge: Option<(Game, State)> = None;
    let mut khits = 0;
    loop {
        if let Some(last) = games.pop() {
            let mut limit = LIMIT;
            let outcome = find_outcome(&last, &mut cache, &mut limit, known_wins, known_draws, &mut khits);
            if limit == 0 {
                // println!("reached limit");
                break;
            }
            // last.print();
            // println!("Outcome: {:?}", outcome);
            // println!("Positions in cache: {}, steps taken: {}", cache.len(), LIMIT - limit);
            if outcome != State::InProgress {
                knowledge = Some((last.clone(), outcome.clone()));
            }
        }
        else {
            println!("---  ---  ---  ---  ---  ---  ---  ---  ---  ---");
            println!("GAME SOLVED");
            println!("---  ---  ---  ---  ---  ---  ---  ---  ---  ---");
            break;
        }
    }
    // println!("Carried out knowledge: {:?}, khits no.: {}", knowledge, khits);
    (knowledge, khits)
}

// learn generates 'knowledge' by playing random games and saving outcomes
fn learn() {
    // this is a parallel version of the algorithm

    let rw_known_wins = Arc::new(RwLock::new(HashSet::new()));
    let rw_known_draws = Arc::new(RwLock::new(HashSet::new()));

    let mut khits_total = 0;
    let cpu_cores = num_cpus::get();

    for _ in 0..10_000 {
        let mut results = VecDeque::new();
        
        // spawn the threads
        for _ in 0..cpu_cores {
            let l_known_wins = rw_known_wins.clone();
            let l_known_draws = rw_known_draws.clone();
            results.push_back(thread::spawn(move || {
                let known_wins = l_known_wins.read().unwrap();
                let known_draws = l_known_draws.read().unwrap();
                get_knowledge(&known_wins, &known_draws)
            }));
        }

        // get all the results
        while let Some(thread_handle) = results.pop_front() {
            let (knowledge, khits0) = thread_handle.join().expect("child thread should complete");
            khits_total += khits0;
            if let Some((game, outcome)) = knowledge {
                let mut known_wins = rw_known_wins.write().unwrap();
                let mut known_draws = rw_known_draws.write().unwrap();
                println!("Random fact: {:?} {:?}, total khits: {}, knowledge: {}", game, outcome, khits_total, known_wins.len() + known_draws.len());
                match outcome {
                    State::Draw => { known_draws.insert(game); },
                    State::Win(p) => {
                        if p == game.t {
                            known_wins.insert(game);
                        } else {
                            for g in next(&game).into_iter() {
                                known_wins.insert(g);
                            }
                        }
                    },
                    _ => panic!("unexpected outcome"),
                };
            }
        }
    }
}

fn main() {
    // let g = Game::new();
    learn();    
    //experiment_with_outcomes();
}

// less-important main-like functions:

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

fn experiment_with_outcomes() {
    let g = Game { p: [[0, 0, 0, 1, 2, 1, 20], [0, 0, 2, 1, 1, 0, 20]], t: 0 };
    let mut cache: HashMap<Game, State> = HashMap::with_capacity(2_000_000);
    let mut limit = 2_000_000;
    let mut khits = 0;
    let empty_knownledge: HashSet<Game> = HashSet::new();
    let outcome = find_outcome(&g, &mut cache, &mut limit, &empty_knownledge, &empty_knownledge, &mut khits);
    g.print();
    println!("Outcome: {:?}", outcome);
    println!("Positions in cache: {}", cache.len());
}

