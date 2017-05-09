type Player = [i8; 7];
struct Game {
    p: [Player; 2],
    t: i8,
}

fn new_player() -> Player {
    [4,4,4,4,4,4,0]
}

fn new_game() -> Game {
    Game{p: [new_player(), new_player()], t: 0}
}

fn print(g: &Game) {
    print!("|{:2}|", g.p[1][6]);
    for i in 0..6 {
        print!(" [{:2}]", g.p[1][5 - i])
    }
    print!(" /--\\");
    if g.t == 1 {
        print!(" <-- turn");
    }
    println!();
    print!("\\__/ ");
    for i in 0..6 {
        print!("[{:2}] ", g.p[0][i])
    }
    print!("|{:2}|", g.p[0][6]);
    if g.t == 0 {
        print!(" <-- turn");
    }
    println!();
}

fn main() {
    print(&new_game());
}
