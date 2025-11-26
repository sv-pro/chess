use chess_engine::{get_best_move_core, Board, Color, Move, PieceType};

fn main() {
    println!("Welcome to Console Chess!");
    println!("You play as White. Enter moves as 'e2e4'.");

    // Setup initial board
    let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let mut user_color = Color::White;
    let mut history: Vec<String> = Vec::new();

    // Rustyline setup
    let mut rl = rustyline::DefaultEditor::new().unwrap();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        print_board(&board);

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
                                println!("  /save    - Print current FEN");
                                println!("  /history - Show move history");
                                println!("  /new     - Start new game");
                                println!("  /swap    - Swap sides");
                                println!("  /quit    - Exit");
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
            let fen = board_to_fen(&board);
            if let Some(m) = get_best_move_core(&fen, 3) {
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
            } else {
                println!("Bot has no moves. Game Over.");
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}

fn print_board(board: &Board) {
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
