const PIECES = {
    w: { k: '♔', q: '♕', r: '♖', b: '♗', n: '♘', p: '♙' },
    b: { k: '♚', q: '♛', r: '♜', b: '♝', n: '♞', p: '♟' }
};

class ChessGame {
    constructor() {
        this.board = [];
        this.turn = 'w'; // 'w' or 'b'
        this.selectedSquare = null;
        this.validMoves = [];
        this.history = [];

        this.initBoard();
        this.renderBoard();
        this.setupEventListeners();
    }

    initBoard() {
        // 8x8 board. null = empty.
        // Format: { type: 'p', color: 'w' }
        this.board = Array(8).fill(null).map(() => Array(8).fill(null));

        const setupRow = (row, color, pieces) => {
            pieces.forEach((type, col) => {
                this.board[row][col] = { type, color };
            });
        };

        const backRow = ['r', 'n', 'b', 'q', 'k', 'b', 'n', 'r'];
        const pawnRow = Array(8).fill('p');

        setupRow(0, 'b', backRow);
        setupRow(1, 'b', pawnRow);
        setupRow(6, 'w', pawnRow);
        setupRow(7, 'w', backRow);
    }

    renderBoard() {
        const boardEl = document.getElementById('board');
        boardEl.innerHTML = '';

        for (let row = 0; row < 8; row++) {
            for (let col = 0; col < 8; col++) {
                const square = document.createElement('div');
                square.className = `square ${(row + col) % 2 === 0 ? 'light' : 'dark'}`;
                square.dataset.row = row;
                square.dataset.col = col;

                const piece = this.board[row][col];
                if (piece) {
                    const pieceSpan = document.createElement('span');
                    pieceSpan.className = 'piece';
                    pieceSpan.textContent = PIECES[piece.color][piece.type];
                    // Make pieces selectable only if it's their turn
                    if (piece.color === this.turn) {
                        pieceSpan.style.cursor = 'grab';
                    } else {
                        pieceSpan.style.cursor = 'default';
                    }
                    square.appendChild(pieceSpan);
                }

                if (this.selectedSquare && this.selectedSquare.row === row && this.selectedSquare.col === col) {
                    square.classList.add('selected');
                }

                if (this.validMoves.some(m => m.row === row && m.col === col)) {
                    square.classList.add('valid-move');
                }

                square.addEventListener('click', () => this.handleSquareClick(row, col));
                boardEl.appendChild(square);
            }
        }

        this.updateStatus();
    }

    handleSquareClick(row, col) {
        if (this.gameOver) return;

        const clickedPiece = this.board[row][col];

        // If clicking a valid move for the selected piece
        const move = this.validMoves.find(m => m.row === row && m.col === col);
        if (move) {
            this.makeMove(this.selectedSquare, { row, col });
            return;
        }

        // Selecting a piece
        if (clickedPiece && clickedPiece.color === this.turn) {
            this.selectedSquare = { row, col };
            this.validMoves = this.calculateValidMoves(row, col);
            this.renderBoard();
        } else {
            // Deselect
            this.selectedSquare = null;
            this.validMoves = [];
            this.renderBoard();
        }
    }

    calculateValidMoves(row, col) {
        const piece = this.board[row][col];
        if (!piece) return [];

        let moves = [];

        // Basic logic for now - to be expanded
        // This is a simplified version. Real chess logic is complex.

        const addMoveIfValid = (r, c) => {
            if (r >= 0 && r < 8 && c >= 0 && c < 8) {
                const target = this.board[r][c];
                if (!target || target.color !== piece.color) {
                    moves.push({ row: r, col: c });
                }
            }
        };

        const directions = {
            'p': [], // Handled separately
            'r': [[0, 1], [0, -1], [1, 0], [-1, 0]],
            'b': [[1, 1], [1, -1], [-1, 1], [-1, -1]],
            'n': [[2, 1], [2, -1], [-2, 1], [-2, -1], [1, 2], [1, -2], [-1, 2], [-1, -2]],
            'q': [[0, 1], [0, -1], [1, 0], [-1, 0], [1, 1], [1, -1], [-1, 1], [-1, -1]],
            'k': [[0, 1], [0, -1], [1, 0], [-1, 0], [1, 1], [1, -1], [-1, 1], [-1, -1]]
        };

        if (piece.type === 'p') {
            const direction = piece.color === 'w' ? -1 : 1;
            // Move forward 1
            if (!this.board[row + direction][col]) {
                moves.push({ row: row + direction, col: col });
                // Move forward 2 (initial)
                if ((piece.color === 'w' && row === 6) || (piece.color === 'b' && row === 1)) {
                    if (!this.board[row + direction * 2][col]) {
                        moves.push({ row: row + direction * 2, col: col });
                    }
                }
            }
            // Capture
            [[direction, 1], [direction, -1]].forEach(([dr, dc]) => {
                const r = row + dr;
                const c = col + dc;
                if (r >= 0 && r < 8 && c >= 0 && c < 8) {
                    const target = this.board[r][c];
                    if (target && target.color !== piece.color) {
                        moves.push({ row: r, col: c });
                    }
                }
            });
        } else if (['r', 'b', 'q'].includes(piece.type)) {
            // Sliding pieces
            directions[piece.type].forEach(([dr, dc]) => {
                let r = row + dr;
                let c = col + dc;
                while (r >= 0 && r < 8 && c >= 0 && c < 8) {
                    const target = this.board[r][c];
                    if (!target) {
                        moves.push({ row: r, col: c });
                    } else {
                        if (target.color !== piece.color) {
                            moves.push({ row: r, col: c });
                        }
                        break; // Blocked
                    }
                    r += dr;
                    c += dc;
                }
            });
        } else {
            // King and Knight (non-sliding)
            directions[piece.type].forEach(([dr, dc]) => {
                addMoveIfValid(row + dr, col + dc);
            });
        }

        // Filter moves that leave king in check
        // We only do this if we are not already simulating a move to prevent infinite recursion
        if (!this.simulating) {
            moves = moves.filter(move => {
                // Simulate move
                const originalPiece = this.board[move.row][move.col];
                const sourcePiece = this.board[row][col];

                this.board[move.row][move.col] = sourcePiece;
                this.board[row][col] = null;
                this.simulating = true;

                const inCheck = this.isInCheck(piece.color);

                // Undo move
                this.board[row][col] = sourcePiece;
                this.board[move.row][move.col] = originalPiece;
                this.simulating = false;

                return !inCheck;
            });
        }

        return moves;
    }

    isInCheck(color) {
        // Find King
        let kingPos = null;
        for (let r = 0; r < 8; r++) {
            for (let c = 0; c < 8; c++) {
                const p = this.board[r][c];
                if (p && p.color === color && p.type === 'k') {
                    kingPos = { row: r, col: c };
                    break;
                }
            }
            if (kingPos) break;
        }

        if (!kingPos) return false; // Should not happen

        const opponentColor = color === 'w' ? 'b' : 'w';
        return this.isSquareAttacked(kingPos.row, kingPos.col, opponentColor);
    }

    isSquareAttacked(row, col, attackerColor) {
        // Check for attacks from all directions
        // This is essentially checking if a piece of type X could move to (row, col)

        // Check Pawn attacks
        const pawnDir = attackerColor === 'w' ? 1 : -1; // White attacks upwards (lower row index) from their perspective? No, white pawns are at row 6 moving to 0? Wait.
        // My board setup: Row 0 is Black pieces. Row 7 is White pieces.
        // White pawns move Row 6 -> 5. Direction is -1.
        // Black pawns move Row 1 -> 2. Direction is +1.

        // If checking if White attacks (row, col), we look for White pawns at (row+1, col±1)
        // If checking if Black attacks (row, col), we look for Black pawns at (row-1, col±1)

        const attackDir = attackerColor === 'w' ? 1 : -1;
        // Wait, if I am at (row, col) and want to know if a White pawn attacks me.
        // A White pawn at (r, c) attacks (r-1, c-1) and (r-1, c+1).
        // So if I am at (row, col), I am attacked by White pawn if White pawn is at (row+1, col±1).

        const pawnRow = row + (attackerColor === 'w' ? 1 : -1);
        if (pawnRow >= 0 && pawnRow < 8) {
            if (col - 1 >= 0) {
                const p = this.board[pawnRow][col - 1];
                if (p && p.color === attackerColor && p.type === 'p') return true;
            }
            if (col + 1 < 8) {
                const p = this.board[pawnRow][col + 1];
                if (p && p.color === attackerColor && p.type === 'p') return true;
            }
        }

        // Check Knight attacks
        const knightMoves = [[2, 1], [2, -1], [-2, 1], [-2, -1], [1, 2], [1, -2], [-1, 2], [-1, -2]];
        for (const [dr, dc] of knightMoves) {
            const r = row + dr, c = col + dc;
            if (r >= 0 && r < 8 && c >= 0 && c < 8) {
                const p = this.board[r][c];
                if (p && p.color === attackerColor && p.type === 'n') return true;
            }
        }

        // Check King attacks (for adjacent kings)
        const kingMoves = [[0, 1], [0, -1], [1, 0], [-1, 0], [1, 1], [1, -1], [-1, 1], [-1, -1]];
        for (const [dr, dc] of kingMoves) {
            const r = row + dr, c = col + dc;
            if (r >= 0 && r < 8 && c >= 0 && c < 8) {
                const p = this.board[r][c];
                if (p && p.color === attackerColor && p.type === 'k') return true;
            }
        }

        // Check Sliding pieces (Rook, Bishop, Queen)
        const straightDirs = [[0, 1], [0, -1], [1, 0], [-1, 0]];
        const diagDirs = [[1, 1], [1, -1], [-1, 1], [-1, -1]];

        // Rooks and Queens
        for (const [dr, dc] of straightDirs) {
            let r = row + dr, c = col + dc;
            while (r >= 0 && r < 8 && c >= 0 && c < 8) {
                const p = this.board[r][c];
                if (p) {
                    if (p.color === attackerColor && (p.type === 'r' || p.type === 'q')) return true;
                    break; // Blocked
                }
                r += dr;
                c += dc;
            }
        }

        // Bishops and Queens
        for (const [dr, dc] of diagDirs) {
            let r = row + dr, c = col + dc;
            while (r >= 0 && r < 8 && c >= 0 && c < 8) {
                const p = this.board[r][c];
                if (p) {
                    if (p.color === attackerColor && (p.type === 'b' || p.type === 'q')) return true;
                    break; // Blocked
                }
                r += dr;
                c += dc;
            }
        }

        return false;
    }

    makeMove(from, to) {
        const piece = this.board[from.row][from.col];
        this.board[to.row][to.col] = piece;
        this.board[from.row][from.col] = null;

        // Pawn promotion (auto-queen for simplicity)
        if (piece.type === 'p' && (to.row === 0 || to.row === 7)) {
            piece.type = 'q';
        }

        this.turn = this.turn === 'w' ? 'b' : 'w';
        this.selectedSquare = null;
        this.validMoves = [];
        this.renderBoard();
        this.checkGameStatus();
    }

    checkGameStatus() {
        const hasMoves = this.hasAnyValidMoves(this.turn);
        const inCheck = this.isInCheck(this.turn);

        const statusEl = document.getElementById('status');

        if (!hasMoves) {
            if (inCheck) {
                statusEl.textContent = `Checkmate! ${this.turn === 'w' ? "Black" : "White"} wins!`;
            } else {
                statusEl.textContent = "Stalemate! Draw.";
            }
            // Disable further interaction
            this.gameOver = true;
        } else if (inCheck) {
            statusEl.textContent = `${this.turn === 'w' ? "White" : "Black"}'s Turn (Check!)`;
        } else {
            statusEl.textContent = `${this.turn === 'w' ? "White" : "Black"}'s Turn`;
        }
    }

    hasAnyValidMoves(color) {
        for (let r = 0; r < 8; r++) {
            for (let c = 0; c < 8; c++) {
                const piece = this.board[r][c];
                if (piece && piece.color === color) {
                    const moves = this.calculateValidMoves(r, c);
                    if (moves.length > 0) return true;
                }
            }
        }
        return false;
    }

    updateStatus() {
        // Status is now handled by checkGameStatus, but we might need this for initial render
        if (!this.gameOver) {
            const statusEl = document.getElementById('status');
            statusEl.textContent = `${this.turn === 'w' ? "White" : "Black"}'s Turn`;
        }
    }

    setupEventListeners() {
        document.getElementById('reset-btn').addEventListener('click', () => {
            this.initBoard();
            this.turn = 'w';
            this.selectedSquare = null;
            this.validMoves = [];
            this.gameOver = false;
            this.renderBoard();
            this.updateStatus();
        });
    }
}

// Start the game
new ChessGame();
