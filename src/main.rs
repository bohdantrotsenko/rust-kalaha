#![allow(dead_code)]
extern crate rand;
extern crate num_cpus;
extern crate byteorder;

// see http://www.wikihow.com/Play-Kalaha for the description of the game

use std::io::*;
use std::fs::File;
//use std::io::prelude::*;
use rand::Rng;
use std::thread;
use std::sync::*;
use std::collections::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

const LIMIT:usize = 150_000; // how many games should I play till I gather all the required data
const LCYCLES:usize = 10_000; // how many cycles to do before exiting
const WINTERVAL:usize = 100; // how often persist the knowledge to disk

type Player = [i8; 7]; // first 6 cells are regular cells, the last one is the super-cell

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
struct Game {
    p: [Player; 2],
    t: i8, // turn of the game; 0 <==> first player, 1 <==> second player
}

fn send_i8(t: &mut u64, b: i8) {
    assert!(b >= 0, "b must be >= 0");
    for _ in 0..b {
        *t = *t << 1;
        *t = *t | 1;
    }
    *t = *t << 1;
}

fn read_i8(t: &mut u64) -> i8 {
    let mut res = 0;
    assert!(*t & 1 == 0);
    *t = *t >> 1;
    while *t & 1 == 1 {
        res += 1;
        *t = *t >> 1;
    }
    res
}

fn pack(g: &Game) -> u64 {
    let mut res = 0;
    for p in 0..2 {
        for i in 0..7 {
            send_i8(&mut res, g.p[p][i]);
        }
    }
    send_i8(&mut res, g.t);
    res
}

fn unpack(d: u64) -> Game {
    let mut dd = d;
    let mut res = Game::new();
    res.t = read_i8(&mut dd);
    for p in (0..2).rev() {
        for i in (0..7).rev() {
            res.p[p][i] = read_i8(&mut dd);
        }
    }
    res
}

#[test]
fn test_packing() {
    let start = Game::new();
    assert!(start == Game::new(), "I don't know comparison");
    let packed = pack(&start);
    let unpacked = unpack(packed);
    assert!(start == unpacked, "packing doesn't work on start position");
}

#[test]
fn test_packing_ex() {
    let bench = next(&Game::new());
    for game in bench.into_iter() {
        let packed = pack(&game);
        let unpacked = unpack(packed);
        assert!(game == unpacked, format!("packing doesn't work on {:?}", game));
    }
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

fn find_outcome_dfs(g: &Game, cache: &mut HashMap<Game, State>, limit: &mut usize, known_wins: &HashSet<u64>, known_draws: &HashSet<u64>, khits: &mut usize) -> State {
    let gstate = g.state();
    if gstate != State::InProgress {
        return gstate;
    }
    if let Some(ans) = cache.get(&g) {
        return ans.clone();
    }
    let g_packed = pack(g);
    if known_wins.contains(&g_packed) {
        *khits += 1;
        return State::Win(g.t);
    }
    if known_draws.contains(&g_packed) {
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
            let outcome = find_outcome_dfs(&ng, cache, limit, known_wins, known_draws, khits);
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

fn get_knowledge(known_wins: &HashSet<u64>, known_draws: &HashSet<u64>) -> (Option<(Game, State)>, usize) {
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
            let outcome = find_outcome_dfs(&last, &mut cache, &mut limit, known_wins, known_draws, &mut khits);
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
        }
    }
    // println!("Carried out knowledge: {:?}, khits no.: {}", knowledge, khits);
    (knowledge, khits)
}

fn read_file(target: &mut HashSet<u64>, filename: &str) {
    if let Ok(file) = File::open(filename) {
        let mut reader = BufReader::new(file);
        let mut counter: u64 = 0;
        while let Ok(packed) = reader.read_u64::<LittleEndian>() {
            target.insert(packed);
            counter += 1;
        }
        println!("read {} cached games from {}", counter, filename);
    } else {
        println!("using empty set for {}", filename);
    }
}

fn write_file(source: &HashSet<u64>, filename: &str) -> Result<()> {
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);
    for packed in source {
        writer.write_u64::<LittleEndian>(*packed)?;
    }
    println!("stored {} cached games in {}", source.len(), filename);
    Ok(())
}

// learn generates 'knowledge' by playing random games and saving outcomes
fn learn_parallel() -> Result<()> {
    // this is a parallel version of the algorithm

    let rw_known_wins = Arc::new(RwLock::new(HashSet::new()));
    let rw_known_draws = Arc::new(RwLock::new(HashSet::new()));
    read_file(&mut rw_known_wins.write().unwrap(), "wins.u64");
    read_file(&mut rw_known_draws.write().unwrap(), "draws.u64");
    let mut khits_total = 0;
    let cpu_cores = num_cpus::get();

    for learning_round in 0..LCYCLES {
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
                    State::Draw => { known_draws.insert(pack(&game)); },
                    State::Win(p) => {
                        if p == game.t {
                            if !known_wins.insert(pack(&game)) {
                                println!("got false knowledge");
                            }
                        } else {
                            for g in next(&game).into_iter() {
                                if !known_wins.insert(pack(&g)) {
                                    println!("got false knowledge");
                                }
                            }
                        }
                    },
                    _ => panic!("unexpected outcome"),
                };
            }
        }

        if learning_round % WINTERVAL == WINTERVAL - 1 {
            write_file(&rw_known_wins.read().unwrap(), "wins.u64")?;
            write_file(&rw_known_draws.read().unwrap(), "draws.u64")?;
        }
    }

    Ok(())
}

fn random_game_len(rng: &mut rand::ThreadRng) -> usize {
    let mut g = Game::new();
    let mut counter = 0;
    loop {
        let state = g.state();
        if state != State::InProgress {
            return counter;
        }

        let possibilities = next(&g);
        let n = possibilities.len();
        g = possibilities[rng.gen_range(0, n)].clone();
        counter += 1;
    }
}

fn see_random_game_len_distr() {
    let mut rng = rand::thread_rng();
    let n = 1_000_000;
    let mut hist: HashMap<usize, usize> = HashMap::new();
    for _ in 0..n {
        let l = random_game_len(&mut rng);
        let counter = hist.entry(l).or_insert(0);
        *counter += 1;
    }
    let mut histv: Vec<_> = hist.into_iter().collect();
    histv.sort_by(|a, b| b.0.cmp(&a.0).reverse());
    println!("{:?}", histv);
}

fn main() {
    //see_random_game_len_distr();
    //play_random_game();
    //experiment_with_outcomes();
    // let g = Game::new();
    match learn_parallel() {
        Ok(()) => println!("done learning."),
        Err(err) => println!("error {}", err),
    }
    //experiment_with_outcomes();
}

fn experiment_with_outcomes() {
    let g = Game { p: [[0, 0, 0, 1, 2, 1, 20], [0, 0, 2, 1, 1, 0, 20]], t: 0 };
    let mut cache: HashMap<Game, State> = HashMap::with_capacity(2_000_000);
    let mut limit = 2_000_000;
    let mut khits = 0;
    let empty_knownledge: HashSet<u64> = HashSet::new();
    let outcome = find_outcome_dfs(&g, &mut cache, &mut limit, &empty_knownledge, &empty_knownledge, &mut khits);
    g.print();
    println!("Outcome: {:?}", outcome);
    println!("Positions in cache: {}", cache.len());
}

