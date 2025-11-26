use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Basic types
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    White,
    Black,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

#[derive(Clone, PartialEq)]
pub struct Board {
    pub squares: [Option<Piece>; 64],
    pub turn: Color,
}

impl Board {
    pub fn new() -> Self {
        // Initialize empty board
        let squares = [None; 64];
        Board {
            squares,
            turn: Color::White,
        }
    }

    pub fn from_fen(fen: &str) -> Self {
        let mut board = Board::new();
        let parts: Vec<&str> = fen.split_whitespace().collect();

        // 1. Piece placement
        let rows: Vec<&str> = parts[0].split('/').collect();
        for (r, row) in rows.iter().enumerate() {
            let mut c = 0;
            for char in row.chars() {
                if let Some(digit) = char.to_digit(10) {
                    c += digit as usize;
                } else {
                    let color = if char.is_uppercase() {
                        Color::White
                    } else {
                        Color::Black
                    };
                    let piece_type = match char.to_ascii_lowercase() {
                        'p' => PieceType::Pawn,
                        'n' => PieceType::Knight,
                        'b' => PieceType::Bishop,
                        'r' => PieceType::Rook,
                        'q' => PieceType::Queen,
                        'k' => PieceType::King,
                        _ => panic!("Invalid FEN character"),
                    };
                    board.squares[r * 8 + c] = Some(Piece { piece_type, color });
                    c += 1;
                }
            }
        }

        // 2. Active color
        if parts.len() > 1 {
            board.turn = if parts[1] == "w" {
                Color::White
            } else {
                Color::Black
            };
        }

        board
    }

    pub fn get_piece(&self, row: usize, col: usize) -> Option<Piece> {
        if row >= 8 || col >= 8 {
            return None;
        }
        self.squares[row * 8 + col]
    }

    pub fn make_move(&mut self, m: &Move) {
        let piece = self.squares[m.from_row * 8 + m.from_col].take();
        self.squares[m.to_row * 8 + m.to_col] = piece;

        // Pawn promotion (auto-queen for simplicity in this engine version)
        if let Some(mut p) = self.squares[m.to_row * 8 + m.to_col] {
            if p.piece_type == PieceType::Pawn {
                if (p.color == Color::White && m.to_row == 0)
                    || (p.color == Color::Black && m.to_row == 7)
                {
                    p.piece_type = PieceType::Queen;
                    self.squares[m.to_row * 8 + m.to_col] = Some(p);
                }
            }
        }

        self.turn = match self.turn {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Move {
    pub from_row: usize,
    pub from_col: usize,
    pub to_row: usize,
    pub to_col: usize,
}

impl Move {
    #[allow(dead_code)]
    fn to_string(&self) -> String {
        format!(
            "{},{},{},{}",
            self.from_row, self.from_col, self.to_row, self.to_col
        )
    }
}

// Evaluation
const PAWN_VAL: i32 = 100;
const KNIGHT_VAL: i32 = 320;
const BISHOP_VAL: i32 = 330;
const ROOK_VAL: i32 = 500;
const QUEEN_VAL: i32 = 900;
const KING_VAL: i32 = 20000;

fn evaluate(board: &Board) -> i32 {
    let mut score = 0;
    for i in 0..64 {
        if let Some(piece) = board.squares[i] {
            let val = match piece.piece_type {
                PieceType::Pawn => PAWN_VAL,
                PieceType::Knight => KNIGHT_VAL,
                PieceType::Bishop => BISHOP_VAL,
                PieceType::Rook => ROOK_VAL,
                PieceType::Queen => QUEEN_VAL,
                PieceType::King => KING_VAL,
            };

            if piece.color == Color::White {
                score += val;
            } else {
                score -= val;
            }
        }
    }
    score
}

// Move Generation (Simplified for brevity, but functional)
pub fn generate_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();
    for r in 0..8 {
        for c in 0..8 {
            if let Some(piece) = board.get_piece(r, c) {
                if piece.color == board.turn {
                    // Directions
                    let dirs = match piece.piece_type {
                        PieceType::Pawn => Vec::new(),
                        PieceType::Knight => vec![
                            (-2, -1),
                            (-2, 1),
                            (-1, -2),
                            (-1, 2),
                            (1, -2),
                            (1, 2),
                            (2, -1),
                            (2, 1),
                        ],
                        PieceType::Bishop => vec![(-1, -1), (-1, 1), (1, -1), (1, 1)],
                        PieceType::Rook => vec![(-1, 0), (1, 0), (0, -1), (0, 1)],
                        PieceType::Queen | PieceType::King => vec![
                            (-1, -1),
                            (-1, 0),
                            (-1, 1),
                            (0, -1),
                            (0, 1),
                            (1, -1),
                            (1, 0),
                            (1, 1),
                        ],
                    };

                    if piece.piece_type == PieceType::Pawn {
                        let dir = if piece.color == Color::White { -1 } else { 1 };
                        // Move 1
                        let r1 = (r as i32 + dir) as usize;
                        if r1 < 8 && board.get_piece(r1, c).is_none() {
                            moves.push(Move {
                                from_row: r,
                                from_col: c,
                                to_row: r1,
                                to_col: c,
                            });
                            // Move 2
                            if (piece.color == Color::White && r == 6)
                                || (piece.color == Color::Black && r == 1)
                            {
                                let r2 = (r as i32 + dir * 2) as usize;
                                if r2 < 8 && board.get_piece(r2, c).is_none() {
                                    moves.push(Move {
                                        from_row: r,
                                        from_col: c,
                                        to_row: r2,
                                        to_col: c,
                                    });
                                }
                            }
                        }
                        // Captures
                        for dc in [-1, 1] {
                            let r_cap = (r as i32 + dir) as usize;
                            let c_cap = (c as i32 + dc) as usize;
                            if r_cap < 8 && c_cap < 8 {
                                if let Some(target) = board.get_piece(r_cap, c_cap) {
                                    if target.color != piece.color {
                                        moves.push(Move {
                                            from_row: r,
                                            from_col: c,
                                            to_row: r_cap,
                                            to_col: c_cap,
                                        });
                                    }
                                }
                            }
                        }
                    } else if piece.piece_type == PieceType::Knight
                        || piece.piece_type == PieceType::King
                    {
                        for (dr, dc) in dirs {
                            let nr = r as i32 + dr;
                            let nc = c as i32 + dc;
                            if nr >= 0 && nr < 8 && nc >= 0 && nc < 8 {
                                let nr = nr as usize;
                                let nc = nc as usize;
                                let target = board.get_piece(nr, nc);
                                if target.is_none() || target.unwrap().color != piece.color {
                                    moves.push(Move {
                                        from_row: r,
                                        from_col: c,
                                        to_row: nr,
                                        to_col: nc,
                                    });
                                }
                            }
                        }
                    } else {
                        // Sliding
                        for (dr, dc) in dirs {
                            let mut nr = r as i32 + dr;
                            let mut nc = c as i32 + dc;
                            while nr >= 0 && nr < 8 && nc >= 0 && nc < 8 {
                                let unr = nr as usize;
                                let unc = nc as usize;
                                let target = board.get_piece(unr, unc);
                                if target.is_none() {
                                    moves.push(Move {
                                        from_row: r,
                                        from_col: c,
                                        to_row: unr,
                                        to_col: unc,
                                    });
                                } else {
                                    if target.unwrap().color != piece.color {
                                        moves.push(Move {
                                            from_row: r,
                                            from_col: c,
                                            to_row: unr,
                                            to_col: unc,
                                        });
                                    }
                                    break;
                                }
                                nr += dr;
                                nc += dc;
                            }
                        }
                    }
                }
            }
        }
    }

    moves
        .into_iter()
        .filter(|m| {
            let mut b_clone = board.clone();
            b_clone.make_move(m);
            !is_in_check(&b_clone, board.turn)
        })
        .collect()
}

fn is_in_check(board: &Board, color: Color) -> bool {
    // Find King
    let mut king_pos = None;
    for i in 0..64 {
        if let Some(p) = board.squares[i] {
            if p.piece_type == PieceType::King && p.color == color {
                king_pos = Some(i);
                break;
            }
        }
    }

    let king_idx = match king_pos {
        Some(idx) => idx,
        None => return true, // King captured
    };

    let kr = king_idx / 8;
    let kc = king_idx % 8;

    let opponent = if color == Color::White {
        Color::Black
    } else {
        Color::White
    };

    // 1. Pawn attacks
    let enemy_dir = if opponent == Color::White { -1 } else { 1 };

    for dc in [-1, 1] {
        let r = (kr as i32 - enemy_dir) as usize;
        let c = (kc as i32 + dc) as usize;
        if r < 8 && c < 8 {
            if let Some(p) = board.get_piece(r, c) {
                if p.color == opponent && p.piece_type == PieceType::Pawn {
                    return true;
                }
            }
        }
    }

    // 2. Knight attacks
    for (dr, dc) in [
        (-2, -1),
        (-2, 1),
        (-1, -2),
        (-1, 2),
        (1, -2),
        (1, 2),
        (2, -1),
        (2, 1),
    ] {
        let r = (kr as i32 + dr) as usize;
        let c = (kc as i32 + dc) as usize;
        if r < 8 && c < 8 {
            if let Some(p) = board.get_piece(r, c) {
                if p.color == opponent && p.piece_type == PieceType::Knight {
                    return true;
                }
            }
        }
    }

    // 3. Sliding & King
    let dirs = [
        (-1, 0),
        (1, 0),
        (0, -1),
        (0, 1), // Rook/Queen
        (-1, -1),
        (-1, 1),
        (1, -1),
        (1, 1), // Bishop/Queen
    ];

    for (i, (dr, dc)) in dirs.iter().enumerate() {
        let mut r = kr as i32 + dr;
        let mut c = kc as i32 + dc;

        // First step (King check too)
        if r >= 0 && r < 8 && c >= 0 && c < 8 {
            if let Some(p) = board.get_piece(r as usize, c as usize) {
                if p.color == opponent {
                    if p.piece_type == PieceType::King {
                        return true;
                    }
                    if p.piece_type == PieceType::Queen {
                        return true;
                    }
                    if i < 4 && p.piece_type == PieceType::Rook {
                        return true;
                    }
                    if i >= 4 && p.piece_type == PieceType::Bishop {
                        return true;
                    }
                }
                continue;
            }

            // Continue sliding
            r += dr;
            c += dc;
            while r >= 0 && r < 8 && c >= 0 && c < 8 {
                if let Some(p) = board.get_piece(r as usize, c as usize) {
                    if p.color == opponent {
                        if p.piece_type == PieceType::Queen {
                            return true;
                        }
                        if i < 4 && p.piece_type == PieceType::Rook {
                            return true;
                        }
                        if i >= 4 && p.piece_type == PieceType::Bishop {
                            return true;
                        }
                    }
                    break; // Blocked
                }
                r += dr;
                c += dc;
            }
        }
    }

    false
}

// Minimax with Alpha-Beta
pub struct Engine {
    pub board: Board,
}

impl Engine {
    pub fn new(board: Board) -> Self {
        Engine { board }
    }

    pub fn search(&mut self, depth: u8, excluded_moves: &[Move]) -> Option<Move> {
        let mut best_move = None;
        let mut best_score = -100000;
        let alpha = -100000;
        let beta = 100000;

        let mut moves = generate_moves(&self.board);

        // Filter excluded moves
        if !excluded_moves.is_empty() {
            moves.retain(|m| !excluded_moves.contains(m));
        }

        for m in moves {
            let mut new_board = self.board.clone();
            new_board.make_move(&m);

            let score = -self.alpha_beta(&new_board, depth - 1, -beta, -alpha);

            if score > best_score {
                best_score = score;
                best_move = Some(m);
            }
        }

        best_move
    }

    fn alpha_beta(&self, board: &Board, depth: u8, mut alpha: i32, beta: i32) -> i32 {
        if depth == 0 {
            return evaluate(board);
        }

        let moves = generate_moves(board);
        if moves.is_empty() {
            if is_in_check(board, board.turn) {
                return -100000 + (depth as i32); // Checkmate (negative because it's bad for current player)
            }
            return 0; // Stalemate
        }

        let mut max_eval = -1000000;
        for m in moves {
            let mut b_clone = board.clone();
            b_clone.make_move(&m);
            let eval = -self.alpha_beta(&b_clone, depth - 1, -beta, -alpha);
            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);
            if beta <= alpha {
                break;
            }
        }
        max_eval
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_best_move(fen: &str, depth: u8) -> Result<JsValue, JsValue> {
    let best_move = get_best_move_core(fen, depth, &[]);
    match best_move {
        Some(m) => Ok(serde_wasm_bindgen::to_value(&m).map_err(|e| e.to_string())?),
        None => Err(JsValue::from_str("No moves available")),
    }
}

pub fn get_best_move_core(fen: &str, depth: u8, excluded_moves: &[Move]) -> Option<Move> {
    let board = Board::from_fen(fen);
    let mut engine = Engine::new(board);
    engine.search(depth, excluded_moves)
}
