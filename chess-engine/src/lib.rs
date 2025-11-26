use wasm_bindgen::prelude::*;
use std::collections::HashMap;

// Basic types
#[derive(Clone, Copy, PartialEq, Debug)]
enum PieceType { Pawn, Knight, Bishop, Rook, Queen, King }

#[derive(Clone, Copy, PartialEq, Debug)]
enum Color { White, Black }

#[derive(Clone, Copy, PartialEq, Debug)]
struct Piece {
    piece_type: PieceType,
    color: Color,
}

#[derive(Clone)]
struct Board {
    squares: [Option<Piece>; 64],
    turn: Color,
}

impl Board {
    fn new() -> Self {
        // Initialize empty board
        let mut squares = [None; 64];
        Board { squares, turn: Color::White }
    }

    fn from_fen(fen: &str) -> Self {
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
                    let color = if char.is_uppercase() { Color::White } else { Color::Black };
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
            board.turn = if parts[1] == "w" { Color::White } else { Color::Black };
        }

        board
    }

    fn get_piece(&self, row: usize, col: usize) -> Option<Piece> {
        if row >= 8 || col >= 8 { return None; }
        self.squares[row * 8 + col]
    }

    fn make_move(&mut self, m: &Move) {
        let piece = self.squares[m.from_row * 8 + m.from_col].take();
        self.squares[m.to_row * 8 + m.to_col] = piece;
        
        // Pawn promotion (auto-queen for simplicity in this engine version)
        if let Some(mut p) = self.squares[m.to_row * 8 + m.to_col] {
             if p.piece_type == PieceType::Pawn {
                 if (p.color == Color::White && m.to_row == 0) || (p.color == Color::Black && m.to_row == 7) {
                     // Note: My FEN parser puts row 0 at top (Black side usually in FEN standard? Wait.)
                     // FEN rank 8 is the first row in the string.
                     // So row 0 in my array is Rank 8 (Black back rank).
                     // Row 7 in my array is Rank 1 (White back rank).
                     // White moves "up" (index decreases? No, standard FEN:
                     // rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
                     // Row 0: rnbqkbnr (Black)
                     // Row 7: RNBQKBNR (White)
                     // So White pawns are at Row 6, moving to Row 0.
                     // Black pawns are at Row 1, moving to Row 7.
                     
                     // Wait, in my JS implementation:
                     // setupRow(0, 'b', backRow); -> Row 0 is Black
                     // setupRow(7, 'w', backRow); -> Row 7 is White
                     // White moves 6 -> 0?
                     // Let's check JS logic:
                     // const direction = piece.color === 'w' ? -1 : 1;
                     // So White moves -1 (Decreasing row index).
                     // So White promotes at Row 0.
                     // Black promotes at Row 7.
                     
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

#[derive(Clone, Debug)]
struct Move {
    from_row: usize,
    from_col: usize,
    to_row: usize,
    to_col: usize,
}

impl Move {
    fn to_string(&self) -> String {
        // Convert to "fromRow,fromCol,toRow,toCol" format for JS to parse easily
        format!("{},{},{},{}", self.from_row, self.from_col, self.to_row, self.to_col)
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
            
            // Simple positional tweaks could be added here
            
            if piece.color == Color::White {
                score += val;
            } else {
                score -= val;
            }
        }
    }
    // Return score from perspective of White.
    // If it's Black's turn, we might want to negate it in search, but let's keep it absolute here.
    score
}

// Move Generation (Simplified for brevity, but functional)
fn generate_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();
    for r in 0..8 {
        for c in 0..8 {
            if let Some(piece) = board.get_piece(r, c) {
                if piece.color == board.turn {
                    // Generate pseudo-legal moves
                    // This duplicates logic from JS, but in Rust.
                    
                    // Directions
                    let dirs = match piece.piece_type {
                        PieceType::Pawn => Vec::new(), // Handled separately
                        PieceType::Knight => vec![(-2, -1), (-2, 1), (-1, -2), (-1, 2), (1, -2), (1, 2), (2, -1), (2, 1)],
                        PieceType::Bishop => vec![(-1, -1), (-1, 1), (1, -1), (1, 1)],
                        PieceType::Rook => vec![(-1, 0), (1, 0), (0, -1), (0, 1)],
                        PieceType::Queen | PieceType::King => vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
                    };

                    if piece.piece_type == PieceType::Pawn {
                        let dir = if piece.color == Color::White { -1 } else { 1 };
                        // Move 1
                        let r1 = (r as i32 + dir) as usize;
                        if r1 < 8 && board.get_piece(r1, c).is_none() {
                            moves.push(Move { from_row: r, from_col: c, to_row: r1, to_col: c });
                            // Move 2
                            if (piece.color == Color::White && r == 6) || (piece.color == Color::Black && r == 1) {
                                let r2 = (r as i32 + dir * 2) as usize;
                                if r2 < 8 && board.get_piece(r2, c).is_none() {
                                    moves.push(Move { from_row: r, from_col: c, to_row: r2, to_col: c });
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
                                        moves.push(Move { from_row: r, from_col: c, to_row: r_cap, to_col: c_cap });
                                    }
                                }
                            }
                        }
                    } else if piece.piece_type == PieceType::Knight || piece.piece_type == PieceType::King {
                        for (dr, dc) in dirs {
                            let nr = r as i32 + dr;
                            let nc = c as i32 + dc;
                            if nr >= 0 && nr < 8 && nc >= 0 && nc < 8 {
                                let nr = nr as usize;
                                let nc = nc as usize;
                                let target = board.get_piece(nr, nc);
                                if target.is_none() || target.unwrap().color != piece.color {
                                    moves.push(Move { from_row: r, from_col: c, to_row: nr, to_col: nc });
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
                                    moves.push(Move { from_row: r, from_col: c, to_row: unr, to_col: unc });
                                } else {
                                    if target.unwrap().color != piece.color {
                                        moves.push(Move { from_row: r, from_col: c, to_row: unr, to_col: unc });
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
    // TODO: Filter illegal moves (checks)
    // For this simple bot, we might skip full check validation in generation to save time, 
    // but we should heavily penalize moving into check or leaving king in check in evaluation (King capture = infinity).
    // Actually, AlphaBeta needs legal moves or it will play illegal moves.
    // Let's add a simple check filter.
    
    moves.into_iter().filter(|m| {
        let mut b_clone = board.clone();
        b_clone.make_move(m);
        // Check if own king is attacked.
        // To save code size, let's just assume for now the bot won't make illegal moves if we prioritize king safety enough?
        // No, that's risky.
        // Let's implement `is_attacked`.
        !is_in_check(&b_clone, board.turn)
    }).collect()
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
        None => return true, // King captured (shouldn't happen in legal play)
    };
    
    let kr = king_idx / 8;
    let kc = king_idx % 8;
    
    // Check if any opponent piece attacks (kr, kc)
    // We can reuse generate_moves logic but inverted? 
    // Or just scan board for enemy pieces and see if they hit King.
    
    let opponent = if color == Color::White { Color::Black } else { Color::White };
    
    // Simplified: Just check if any opponent piece can move to King's square.
    // This is expensive but correct.
    // Optimization: Only generate pseudo-legal moves for opponent and see if any hit King.
    
    // Actually, let's do the "attacked by" logic which is faster.
    
    // 1. Pawn attacks
    let pawn_dir = if color == Color::White { -1 } else { 1 }; // Enemy pawns come from opposite direction?
    // No, if I am White (Row 6->0), Enemy is Black (Row 1->7).
    // Enemy pawns at (kr-1, kc±1) attack me?
    // Black pawns move +1. So if Black pawn is at (kr-1), it attacks (kr).
    // Wait. Black pawn at Row 1 moves to Row 2.
    // If King is at Row 2. Black pawn at Row 1 attacks it.
    // So we look at (kr - enemy_dir).
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
    for (dr, dc) in [(-2, -1), (-2, 1), (-1, -2), (-1, 2), (1, -2), (1, 2), (2, -1), (2, 1)] {
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
        (-1, 0), (1, 0), (0, -1), (0, 1), // Rook/Queen
        (-1, -1), (-1, 1), (1, -1), (1, 1) // Bishop/Queen
    ];
    
    for (i, (dr, dc)) in dirs.iter().enumerate() {
        let mut r = kr as i32 + dr;
        let mut c = kc as i32 + dc;
        
        // First step (King check too)
        if r >= 0 && r < 8 && c >= 0 && c < 8 {
             if let Some(p) = board.get_piece(r as usize, c as usize) {
                 if p.color == opponent {
                     if p.piece_type == PieceType::King { return true; }
                     if p.piece_type == PieceType::Queen { return true; }
                     if i < 4 && p.piece_type == PieceType::Rook { return true; }
                     if i >= 4 && p.piece_type == PieceType::Bishop { return true; }
                 }
                 // Blocked by any piece (friend or foe)
                 continue; 
             }
             
             // Continue sliding
             r += dr;
             c += dc;
             while r >= 0 && r < 8 && c >= 0 && c < 8 {
                 if let Some(p) = board.get_piece(r as usize, c as usize) {
                     if p.color == opponent {
                         if p.piece_type == PieceType::Queen { return true; }
                         if i < 4 && p.piece_type == PieceType::Rook { return true; }
                         if i >= 4 && p.piece_type == PieceType::Bishop { return true; }
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
fn minimax(board: &Board, depth: u8, mut alpha: i32, mut beta: i32, maximizing_player: bool) -> i32 {
    if depth == 0 {
        return evaluate(board);
    }

    let moves = generate_moves(board);
    if moves.is_empty() {
        if is_in_check(board, board.turn) {
            return if maximizing_player { -100000 + (depth as i32) } else { 100000 - (depth as i32) }; // Checkmate
        }
        return 0; // Stalemate
    }

    if maximizing_player {
        let mut max_eval = -1000000;
        for m in moves {
            let mut b_clone = board.clone();
            b_clone.make_move(&m);
            let eval = minimax(&b_clone, depth - 1, alpha, beta, false);
            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);
            if beta <= alpha {
                break;
            }
        }
        max_eval
    } else {
        let mut min_eval = 1000000;
        for m in moves {
            let mut b_clone = board.clone();
            b_clone.make_move(&m);
            let eval = minimax(&b_clone, depth - 1, alpha, beta, true);
            min_eval = min_eval.min(eval);
            beta = beta.min(eval);
            if beta <= alpha {
                break;
            }
        }
        min_eval
    }
}

#[wasm_bindgen]
pub fn get_best_move(fen: &str, depth: u8) -> String {
    let board = Board::from_fen(fen);
    let moves = generate_moves(&board);
    
    if moves.is_empty() {
        return "".to_string();
    }

    let maximizing = board.turn == Color::White;
    let mut best_move = moves[0].clone();
    let mut best_val = if maximizing { -1000000 } else { 1000000 };

    for m in moves {
        let mut b_clone = board.clone();
        b_clone.make_move(&m);
        let val = minimax(&b_clone, depth - 1, -1000000, 1000000, !maximizing);
        
        if maximizing {
            if val > best_val {
                best_val = val;
                best_move = m;
            }
        } else {
            if val < best_val {
                best_val = val;
                best_move = m;
            }
        }
    }

    best_move.to_string()
}
