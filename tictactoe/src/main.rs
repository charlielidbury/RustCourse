#![feature(iter_intersperse)]
use std::fmt;
use std::iter;

#[derive(PartialEq, Copy, Clone, Debug)]
enum Player {
    Player1,
    Player2,
}

impl Player {
    fn get_next(&self) -> Self {
        match self {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        }
    }

    fn get_last(&self) -> Self {
        match self {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Player::Player1 => write!(f, "X"),
            Player::Player2 => write!(f, "O")
        }
    }
}

#[derive(PartialEq, Debug)]
enum GameResult {
    PlayerWon(Player),
    Draw,
}

impl fmt::Display for GameResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            GameResult::PlayerWon(player) => write!(f, "{} won", player),
            GameResult::Draw => write!(f, "draw")
        }
    }
}

#[derive(Debug)]
enum GameError {
    NotLegalPosition,
    PositionOccupied,
}

trait Game {
    type Move: Copy;

    fn moves(&self) -> Vec<Self::Move> {
        iter::successors(
            self.next_move(None),
            |&mv| self.next_move(Some(mv))
        ).collect()
    }

    fn next_move(&self, last_move: Option<Self::Move>) -> Option<Self::Move>;

    // Returns whether or not a given move as just won the game.
    // PRE: self.execute_move(mv) has been run.
    fn winning_move(&self, mv: Self::Move) -> bool;

    fn winner(&self) -> Option<GameResult>;

    fn execute_move(&mut self, player_move: Self::Move) -> Result<(), GameError>;

    fn undo_move(&mut self, player_move: Self::Move) -> Result<(), GameError>;

    fn current_player(&self) -> Player;
}

#[derive(PartialEq, Copy, Clone, Debug)]
struct MNKBoard<const M: usize, const N: usize, const K: usize> {
    board: [[Option<Player>; M]; N], 
    current_player: Player
}

impl<const M: usize, const N: usize, const K: usize> MNKBoard<M, N, K> {
    fn new() -> Self {
        MNKBoard {
            board: [[None; M]; N], 
            current_player: Player::Player1
        }
    }
}

impl<const M: usize, const N: usize, const K: usize> fmt::Display for MNKBoard<M, N, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sep = iter::once('\n')
            .chain(iter::repeat('-').take(2 * M - 1))
            .chain(iter::once('\n'))
            .collect::<String>();
        
        let lines = self.board.iter()
            .map(|row| row.iter()
                .map(|&op| match op {
                    None => ' ',
                    Some(Player::Player1) => 'X',
                    Some(Player::Player2) => 'O',
                })
                .intersperse('|')
                .collect::<String>()
            )
            .intersperse(sep);

        for line in lines {
            write!(f, "{}", line)?;
        }

        write!(f, "\n")?;

        Ok(())
    }
}

// M: width, N: height
impl<const M: usize, const N: usize, const K: usize> Game for MNKBoard<M, N, K> {
    type Move = (usize, usize);

    fn next_move(&self, last_move: Option<Self::Move>) -> Option<Self::Move> {
        // First square to check.
        let start = match last_move {
            None => 0,
            Some((x, y)) => y * M + x + 1,
        };

        for i in start..(N * M) {
            let (x, y) = (i % M, i / M);
            if let None = self.board[y][x] {
                return Some((x, y));
            }
        }

        None
    }

    fn winning_move(&self, mv: Self::Move) -> bool {
        const directions: [(usize, usize); 3] = [
            (0, 1),
            (1, 1),
            (1, 0),
        ];

        let target = self.board[mv.1][mv.0];
        if target.is_none() {
            return false;
        }

        directions.iter().any(|&(dx, dy)| {
            println!("mv = {:?}, dx = {}, dy = {}", mv, dx, dy);
            // Count in forward direction
            // co-ords only increase
            let (mut x, mut y) = (mv.0, mv.1);
            let mut fwd = 0;
            loop {
                x += dx;
                y += dy;

                if x >= M || y >= N || self.board[y][x] != target {
                    break;
                }

                fwd += 1;
            }

            // Count in reverse direction
            // co-ords only decrease
            let (mut x, mut y) = (mv.0, mv.1);
            let mut rev = 0;
            loop {
                if x < dx || y < dy {
                    break;
                }

                x -= dx;
                y -= dy;

                if self.board[y][x] != target {
                    break;
                }

                rev += 1;
            }
            println!("fwd = {}, rev = {}", fwd, rev);

            // 
            fwd + rev >= K - 1
        })
    }

    fn winner(&self) -> Option<GameResult> {
        for y in 0..N {
            for x in 0..M {
                if let Some(player) = self.board[y][x] {
                    if self.winning_move((x, y)) {
                        return Some(GameResult::PlayerWon(player));
                    }
                }
            }
        }

        None
    }

    fn execute_move(&mut self, (x, y): Self::Move) -> Result<(), GameError> {
        // Make move
        self.board[y][x] = Some(self.current_player);

        // Switch players
        self.current_player = self.current_player.get_next();

        Ok(())
    }

    fn undo_move(&mut self, (x, y): Self::Move) -> Result<(), GameError> {
        // Make move
        self.board[y][x] = None;

        // Switch players
        self.current_player = self.current_player.get_last();

        Ok(())
    }

    fn current_player(&self) -> Player {
        self.current_player
    }
}

fn bar() -> [i32; 3] {
    [1, 2, 3]
}

fn foo() {
    let x = bar();

    println!("{:?}", x);
}

trait Agent<G: Game> {
    fn next_move(
        &mut self,
        game: &G
    ) -> G::Move;
}

struct Minimax;

impl<G: Game> Agent<G> for Minimax {
    fn next_move(
        &mut self,
        game: &G
    ) -> G::Move {
        // Picks first vaild move.
        game.next_move(None).unwrap()
    }
}

struct CliAgent;

impl<const M: usize, const N: usize, const K: usize> Agent<MNKBoard<M, N, K>> for CliAgent {
    fn next_move(
        &mut self,
        game: &MNKBoard<M, N, K>
    ) -> <MNKBoard<M, N, K> as Game>::Move {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();

        if let Ok(pos) = parse_input(line.as_str()) {
            return pos;
        } else {
            println!("Not valid input");
            return self.next_move(game);
        }
    }
}

fn parse_input(s: &str) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let mut ms = s.matches(char::is_numeric);
    Ok((
        ms.next().ok_or("no item1")?.parse::<usize>()?,
        ms.next().ok_or("no item2")?.parse::<usize>()?,
    ))
}

fn play_game<G: Game + fmt::Display, P1: Agent<G>, P2: Agent<G>>(
    game: &mut G, p1: &mut P1, p2: &mut P2
) {
    // Game main loop
    let result = loop {
        if let Some(w) = game.winner() {
            break w;
        }

        // Print current state
        println!("{}", game);

        // Selects which agent to ask for a move
        let agent: &mut dyn Agent<G> = match game.current_player() {
            Player::Player1 => p1,
            Player::Player2 => p2,
        };

        // Makes next move.
        let mv = agent.next_move(game);
        game.execute_move(mv).unwrap();
    };

    // Conclusion.
    println!("{}", game);
    println!("Game over. Result: {}", result);
}

fn ai_vs_cli() {
    let mut b = MNKBoard::<3, 3, 3>::new();
    let mut ai = Minimax;
    let mut player = CliAgent;

    play_game(&mut b, &mut player, &mut ai);
}

fn main() {
    let mut acc = 0;

    for i in 0..100000 {
        acc += i;
    }

    println!("acc = {}", acc);

    // ai_vs_cli();
}

mod test {
    use crate::{GameResult::*, Player::*, *};

    #[test]
    fn three_by_three_test_1() {
        let board = MNKBoard::<3, 3, 3>{
            board: [[Some(Player1), None, None],
            [Some(Player1), None, None],
            [Some(Player1), None, None]],
            current_player: Player1
        };

        assert_eq!(board.winner(), Some(PlayerWon(Player1)));
        assert_eq!(
            board.moves(),
            vec![(1, 0), (2, 0), (1, 1), (2, 1), (1, 2), (2, 2)]
        );
    }

    #[test]
    fn three_by_three_test_2() {
        let board = MNKBoard::<3, 3, 3> {
            board: [[None, Some(Player2), None],
            [None, Some(Player2), None],
            [None, Some(Player2), None]],
            current_player: Player1
        };

        assert_eq!(board.winner(), Some(PlayerWon(Player2)));
    }

    #[test]
    fn three_by_three_test_3() {
        let board = MNKBoard::<3, 3, 3> {
            board: [[None, Some(Player2), None],
            [None, Some(Player2), None],
            [None, Some(Player1), None]],
            current_player: Player1
        };

        assert_eq!(board.winner(), None);
    }

    #[test]
    fn three_by_three_test_4() {
        let board = MNKBoard::<3, 3, 2> {
            board: [[None, Some(Player2), None],
            [None, Some(Player2), None],
            [None, Some(Player1), None]],
            current_player: Player1
        };

        assert_eq!(board.winner(), Some(PlayerWon(Player2)));
    }

    #[test]
    fn three_by_three_test_5() {
        let board = MNKBoard::<3, 3, 3> {
            board: [[None, Some(Player2), None],
            [None, Some(Player1), None],
            [Some(Player2), Some(Player2), Some(Player2)]],
            current_player: Player1
        };

        assert_eq!(board.winner(), Some(PlayerWon(Player2)));
    }

    #[test]
    fn three_by_three_test_6() {
        let board = MNKBoard::<3, 3, 2> {
            board: [[None, Some(Player2), None],
            [None, Some(Player1), None],
            [Some(Player2), None, Some(Player1)]],
            current_player: Player1
        };

        assert_eq!(board.winner(), Some(PlayerWon(Player1)));
    }

    #[test]
    fn three_by_three_test_7() {
        let board = MNKBoard::<3, 3, 2> {
            board: [[None, Some(Player2), None],
            [Some(Player1), None, None],
            [Some(Player2), Some(Player1), None]],
            current_player: Player1
        };

        assert_eq!(board.winner(), Some(PlayerWon(Player1)));
        assert_eq!(
            board.moves(),
            vec![(0, 0), (2, 0), (1, 1), (2, 1), (2, 2)]
        );
    }

    #[test]
    fn three_by_three_test_8() {
        let board = MNKBoard::<3, 3, 2> {
            board: [[None, Some(Player2), None],
            [Some(Player1), None, Some(Player2)],
            [Some(Player2), None, None]],
            current_player: Player1
        };

        assert_eq!(board.winner(), Some(PlayerWon(Player2)));
    }

    #[test]
    fn three_by_three_test_9() {
        let board = MNKBoard::<3, 2, 2> {
            board: [[None, Some(Player2), None],
            [Some(Player1), None, Some(Player2)]],
            current_player: Player1
        };

        assert_eq!(board.winner(), Some(PlayerWon(Player2)));
    }

    #[test]
    fn three_by_three_test_10() {
        let board = MNKBoard::<2, 3, 2> {
            board: [[None, Some(Player2)],
            [Some(Player1), None],
            [Some(Player2), Some(Player1)]],
            current_player: Player1
        };

        assert_eq!(board.winner(), Some(PlayerWon(Player1)));
    }

    #[test]
    fn three_by_three_test_11() {
        let board = MNKBoard::<2, 3, 2> {
            board: [[None, Some(Player1)],
            [None, Some(Player2)],
            [Some(Player2), Some(Player1)]],
            current_player: Player1
        };

        assert_eq!(board.winner(), Some(PlayerWon(Player2)));
    }

    #[test]
    fn three_by_three_test_12() {
        let board = MNKBoard::<3, 2, 2> {
            board: [[None, Some(Player2), None],
            [Some(Player2), None, Some(Player1)]],
            current_player: Player1
        };

        assert_eq!(board.winner(), Some(PlayerWon(Player2)));
    }

    #[test]
    fn three_by_three_test_13() {
        let board = MNKBoard::<3, 3, 2> {
            board: [[None, None, None],
            [None, None, Some(Player1)],
            [None, Some(Player1), None]],
            current_player: Player1
        };

        assert_eq!(board.winner(), Some(PlayerWon(Player1)));
    }
}
