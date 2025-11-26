use chess_engine::{get_best_move_core, Board, Color, Move, PieceType};
use rustyline::completion::{Completer, Pair};

use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};
use std::{thread, time};

#[derive(Clone)]
struct ChessHelper {
    commands: Vec<String>,
}

impl Completer for ChessHelper {
    type Candidate = Pair;

    fn complete(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Result<(usize, Vec<Pair>)> {
        if line.starts_with('/') {
            let matches: Vec<Pair> = self
                .commands
                .iter()
                .filter(|cmd| cmd.starts_with(line))
                .map(|cmd| Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone(),
                })
                .collect();
            Ok((0, matches))
        } else {
            Ok((0, Vec::new()))
        }
    }
}

impl Hinter for ChessHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for ChessHelper {}
impl Validator for ChessHelper {}
impl Helper for ChessHelper {}

use std::collections::VecDeque;

fn main() {
    println!("Welcome to Console Chess!");
    println!("You play as White. Enter moves as 'e2e4'.");

    // Setup initial board
    let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let mut user_color = Color::White;
    let mut history: Vec<String> = Vec::new();
    let mut autoplay = false;
    let mut recent_boards: VecDeque<Board> = VecDeque::with_capacity(2);

    // Rustyline setup
    let config = rustyline::Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();
    let mut rl =
        rustyline::Editor::<ChessHelper, rustyline::history::FileHistory>::with_config(config)
            .unwrap();

    let helper = ChessHelper {
        commands: vec![
            "/help".to_string(),
            "/save".to_string(),
            "/history".to_string(),
            "/new".to_string(),
            "/swap".to_string(),
            "/autoplay".to_string(),
            "/quit".to_string(),
        ],
    };
    rl.set_helper(Some(helper));

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        print_board(&board, history.len());

        if autoplay {
            println!("Autoplay: Swapping sides...");
            thread::sleep(time::Duration::from_secs(2));
            user_color = if user_color == Color::White {
                Color::Black
            } else {
                Color::White
            };
        }

        if board.turn == user_color {
            // User turn
            let readline = rl.readline("Enter move (or /help): ");
            match readline {
                Ok(line) => {
                    let input = line.trim();
                    rl.add_history_entry(input.to_string()).unwrap();

                    if input == "quit" || input == "/quit" {
                        break;
                    } else if input.starts_with('/') {
                        match input {
                            "/help" => {
                                println!("Commands:");
                                println!("  /save     - Print current FEN");
                                println!("  /history  - Show move history");
                                println!("  /new      - Start new game");
                                println!("  /swap     - Swap sides");
                                println!("  /autoplay - Auto-swap every 2s");
                                println!("  /quit     - Exit");
                            }
                            "/save" => {
                                let fen = board_to_fen(&board);
                                println!("Game FEN: {}", fen);
                            }
                            "/history" => {
                                println!("Move History:");
                                for (i, move_str) in history.iter().enumerate() {
                                    println!("{}. {}", i + 1, move_str);
                                }
                            }
                            "/new" => {
                                board = Board::from_fen(
                                    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                                );
                                user_color = Color::White;
                                history.clear();
                                autoplay = false;
                                println!("New game started.");
                            }
                            "/swap" => {
                                user_color = if user_color == Color::White {
                                    Color::Black
                                } else {
                                    Color::White
                                };
                                println!("Swapped sides. You are now {:?}.", user_color);
                            }
                            "/autoplay" => {
                                autoplay = true;
                                println!("Autoplay enabled. Press Ctrl-C to stop.");
                                continue;
                            }
                            _ => println!("Unknown command. Type /help for list."),
                        }
                        continue;
                    }

                    if let Some(m) = parse_move(input) {
                        let legal_moves = chess_engine::generate_moves(&board);
                        if legal_moves.contains(&m) {
                            board.make_move(&m);
                            history.push(input.to_string());
                        } else {
                            println!("Illegal move! Try again.");
                        }
                    } else {
                        println!("Invalid move format. Use 'e2e4'.");
                    }
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(rustyline::error::ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        } else {
            // Bot turn
            println!("Bot is thinking...");
            let mut excluded_moves = Vec::new();
            let mut best_move = None;

            // Try to find a non-repeating move
            for _ in 0..5 {
                // Try up to 5 times
                let fen = board_to_fen(&board);
                if let Some(m) = get_best_move_core(&fen, 3, &excluded_moves) {
                    // Check if this move leads to a repeated state
                    let mut test_board = board.clone();
                    test_board.make_move(&m);

                    // Simple repetition check: if we've seen this board state recently
                    let is_repetition = recent_boards.iter().any(|b| {
                        // Compare squares and turn. Ignore castling/ep for now as Board doesn't have them.
                        b.squares
                            .iter()
                            .zip(test_board.squares.iter())
                            .all(|(p1, p2)| p1 == p2)
                            && b.turn == test_board.turn
                    });

                    if is_repetition {
                        println!("Bot avoiding repetition...");
                        excluded_moves.push(m);
                        continue;
                    }

                    best_move = Some(m);
                    break;
                } else {
                    break; // No more moves
                }
            }

            if let Some(m) = best_move {
                let move_str = format!(
                    "{}{}{}{}",
                    (m.from_col as u8 + b'a') as char,
                    8 - m.from_row,
                    (m.to_col as u8 + b'a') as char,
                    8 - m.to_row
                );
                println!("Bot plays: {}", move_str);
                board.make_move(&m);
                history.push(move_str);

                // Update recent boards
                recent_boards.push_back(board.clone());
                if recent_boards.len() > 2 {
                    recent_boards.pop_front();
                }
            } else {
                println!("Bot has no valid moves (or all lead to repetition). Game Over.");
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}

fn print_board(board: &Board, history_len: usize) {
    let move_num = history_len / 2 + 1;
    let side = match board.turn {
        Color::White => "White",
        Color::Black => "Black",
    };
    println!("\nMove: {} | Side to play: {}", move_num, side);
    println!("  a b c d e f g h");
    for r in 0..8 {
        print!("{} ", 8 - r);
        for c in 0..8 {
            if let Some(p) = board.get_piece(r, c) {
                let symbol = match (p.color, p.piece_type) {
                    (Color::White, PieceType::Pawn) => "P",
                    (Color::White, PieceType::Knight) => "N",
                    (Color::White, PieceType::Bishop) => "B",
                    (Color::White, PieceType::Rook) => "R",
                    (Color::White, PieceType::Queen) => "Q",
                    (Color::White, PieceType::King) => "K",
                    (Color::Black, PieceType::Pawn) => "p",
                    (Color::Black, PieceType::Knight) => "n",
                    (Color::Black, PieceType::Bishop) => "b",
                    (Color::Black, PieceType::Rook) => "r",
                    (Color::Black, PieceType::Queen) => "q",
                    (Color::Black, PieceType::King) => "k",
                };
                print!("{} ", symbol);
            } else {
                print!(". ");
            }
        }
        println!("{}", 8 - r);
    }
    println!("  a b c d e f g h");
}

fn parse_move(input: &str) -> Option<Move> {
    if input.len() != 4 {
        return None;
    }
    let chars: Vec<char> = input.chars().collect();

    // Validate columns 'a'-'h'
    if chars[0] < 'a' || chars[0] > 'h' {
        return None;
    }
    if chars[2] < 'a' || chars[2] > 'h' {
        return None;
    }

    // Validate rows '1'-'8'
    if chars[1] < '1' || chars[1] > '8' {
        return None;
    }
    if chars[3] < '1' || chars[3] > '8' {
        return None;
    }

    let from_col = (chars[0] as u8 - b'a') as usize;
    let from_row = 8 - chars[1].to_digit(10).unwrap() as usize;

    let to_col = (chars[2] as u8 - b'a') as usize;
    let to_row = 8 - chars[3].to_digit(10).unwrap() as usize;

    Some(Move {
        from_row,
        from_col,
        to_row,
        to_col,
    })
}

fn board_to_fen(board: &Board) -> String {
    // Simplified FEN generator
    let mut fen = String::new();
    for r in 0..8 {
        let mut empty = 0;
        for c in 0..8 {
            if let Some(p) = board.get_piece(r, c) {
                if empty > 0 {
                    fen.push_str(&empty.to_string());
                    empty = 0;
                }
                let char = match p.piece_type {
                    PieceType::Pawn => 'p',
                    PieceType::Knight => 'n',
                    PieceType::Bishop => 'b',
                    PieceType::Rook => 'r',
                    PieceType::Queen => 'q',
                    PieceType::King => 'k',
                };
                fen.push(if p.color == Color::White {
                    char.to_ascii_uppercase()
                } else {
                    char
                });
            } else {
                empty += 1;
            }
        }
        if empty > 0 {
            fen.push_str(&empty.to_string());
        }
        if r < 7 {
            fen.push('/');
        }
    }

    let turn = if board.turn == Color::White { "w" } else { "b" };
    format!("{} {} - - 0 1", fen, turn)
}
