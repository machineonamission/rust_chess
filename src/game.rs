use colored::*;
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, PartialEq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Color {
    Black,
    White,
}

#[derive(Clone, Copy)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

// (row, col)
pub type Square = (i8, i8);

pub struct CastlingRights {
    pub white_queenside: bool,
    pub white_kingside: bool,
    pub black_queenside: bool,
    pub black_kingside: bool,
}

pub struct Game {
    pub board: [[Option<Piece>; 8]; 8],
    pub turn: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_target_square: Option<Square>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
    pub moves: Vec<Move>,
    pub legal_moves: [[Vec<Move>; 8]; 8]
}

impl Display for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (row, prow) in self.board.iter().enumerate() {
            for (col, piece) in prow.iter().enumerate() {
                let piecestring = match piece {
                    Some(p) => match p.color {
                        Color::Black => match p.piece_type {
                            PieceType::Pawn => "p",
                            PieceType::Knight => "n",
                            PieceType::Bishop => "b",
                            PieceType::Rook => "r",
                            PieceType::Queen => "q",
                            PieceType::King => "k",
                        },
                        Color::White => match p.piece_type {
                            PieceType::Pawn => "P",
                            PieceType::Knight => "N",
                            PieceType::Bishop => "B",
                            PieceType::Rook => "R",
                            PieceType::Queen => "Q",
                            PieceType::King => "K",
                        },
                    },
                    None => " ",
                };
                let colored = if row % 2 == col % 2 {
                    piecestring.on_truecolor(0xb5, 0x88, 0x63).black()
                } else {
                    piecestring.on_truecolor(0xf0, 0xd9, 0xb5).black()
                };
                f.write_fmt(format_args!("{}", colored))?;
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub capture: Option<PieceType>,
    pub castle: bool,
    pub promotion: Option<PieceType>,
    // if the move was en passant
    pub en_passant_capture: Option<Square>,
    // what square did the pawn double move over
    pub en_passant_target_square: Option<Square>,
}

pub fn is_valid_square((row,col): &Square) -> Option<Square> {
    if (0i8..8i8).contains(row) && (0i8..8i8).contains(col) {
        Some((*row, *col))
    } else {
        None
    }
}

impl Game {
    pub fn piece_at_square(&self, square: &Square) -> &Option<Piece> {
        match is_valid_square(square) {
            Some((row, col)) => &self.board[row as usize][col as usize],
            None => &None,
        }
    }
    fn generic_move(&self, from: &Square, to: Square) -> Option<Move> {
        // return no move if invalid
        is_valid_square(&to)?;
        // unwrap is fine here because from should always be valid
        let color = self.piece_at_square(from).as_ref().unwrap().color;
        let capture = self.piece_at_square(&to);
        match capture {
            None => Some(Move {
                from: *from,
                to,
                capture: None,
                castle: false,
                promotion: None,
                en_passant_target_square: None,
                en_passant_capture: None,
            }),
            Some(capture_piece) => {
                if capture_piece.color != color {
                    Some(Move {
                        from: *from,
                        to,
                        capture: Some(capture_piece.piece_type),
                        castle: false,
                        promotion: None,
                        en_passant_target_square: None,
                        en_passant_capture: None,
                    })
                } else {
                    None
                }
            }
        }
    }
    pub fn legal_moves_on_square(&self, square:Square) -> Vec<Move> {
        self.legal_moves[square.0 as usize][square.1 as usize].clone()
    }

    pub fn compute_legal_moves_on_square(&self, square: Square) -> Vec<Move> {
        let piece = self.piece_at_square(&square);
        let mut moves = vec![];
        if let Some(piece_some) = piece {
            if piece_some.color != self.turn {
                return moves
            }
            let (row, col) = square;
            match piece_some.piece_type {
                PieceType::Pawn => {
                    // if to increase row or decrease row
                    let direction: i8 = match piece_some.color {
                        Color::Black => 1,
                        Color::White => -1,
                    };
                    let mut pawn_moves: Vec<Move> = vec![];
                    let torow = row + direction;
                    // diagonal captures
                    for capture_direction in [-1i8, 1i8] {
                        // if the diagonal is a valid square
                        if let Some(capture_square) =
                            is_valid_square(&(torow, col + capture_direction))
                        {
                            // if there's a piece on the diagonal
                            if let Some(capture) = self.piece_at_square(&capture_square) {
                                // if the piece is captureable
                                if capture.color != piece_some.color {
                                    pawn_moves.push(Move {
                                        from: square,
                                        to: capture_square,
                                        capture: Some(capture.piece_type),
                                        castle: false,
                                        promotion: None,
                                        en_passant_target_square: None,
                                        en_passant_capture: None,
                                    });
                                }
                                // no piece but en passant time
                            } else if Some(capture_square) == self.en_passant_target_square {
                                pawn_moves.push(Move {
                                    from: square,
                                    to: capture_square,
                                    capture: Some(PieceType::Pawn),
                                    castle: false,
                                    promotion: None,
                                    en_passant_target_square: None,
                                    en_passant_capture: Some((row, col + capture_direction)),
                                });
                            }
                        }
                    }
                    // if directly ahead is empty
                    // there's no reason this would ever be invalid, pawns promote when they reach the end
                    let one_ahead = (torow, col);
                    if self.piece_at_square(&one_ahead).is_none() {
                        pawn_moves.push(Move {
                            from: square,
                            to: one_ahead,
                            capture: None,
                            castle: false,
                            promotion: None,
                            en_passant_target_square: None,
                            en_passant_capture: None,
                        });
                        // this can only happen if the last square was empty and pawns at initial rows
                        // pawns cant move backwards nor jump over other pieces
                        if (row == 6 && piece_some.color == Color::White)
                            || (row == 1 && piece_some.color == Color::Black)
                        {
                            // always valid square
                            let two_ahead = (row + direction * 2, col);
                            if self.piece_at_square(&two_ahead).is_none() {
                                pawn_moves.push(Move {
                                    from: square,
                                    to: two_ahead,
                                    capture: None,
                                    castle: false,
                                    promotion: None,
                                    en_passant_target_square: Some((row + direction, col)),
                                    en_passant_capture: None,
                                });
                            }
                        }
                    }
                    // pawns cant move backwards so i dont need to validate this for color
                    let promotion = torow == 7 || torow == 0;
                    for mut mov in pawn_moves {
                        if promotion {
                            mov.promotion = Some(PieceType::Queen);
                            moves.push(mov.clone());
                            mov.promotion = Some(PieceType::Knight);
                            moves.push(mov.clone());
                        } else {
                            moves.push(mov);
                        }
                    }
                }
                PieceType::Knight => {
                    // i can generate this dynamically but it's almost certainly faster hardcoded
                    const KNIGHT_MOVES: [(i8, i8); 8] = [
                        (-2, -1),
                        (-1, -2),
                        (-2, 1),
                        (1, -2),
                        (2, -1),
                        (-1, 2),
                        (2, 1),
                        (1, 2),
                    ];
                    for mov in KNIGHT_MOVES {
                        if let Some(m) = self.generic_move(&square, (row + mov.0, col + mov.1)) {
                            moves.push(m);
                        }
                    }
                }
                PieceType::King => {
                    // i can generate this dynamically but it's almost certainly faster hardcoded
                    const KING_MOVES: [(i8, i8); 8] = [
                        (-1, -1),
                        (-1, 0),
                        (-1, 1),
                        (0, -1),
                        (0, 1),
                        (1, -1),
                        (1, 0),
                        (1, 1),
                    ];

                    for mov in KING_MOVES {
                        if let Some(m) = self.generic_move(&square, (row + mov.0, col + mov.1)) {
                            moves.push(m);
                        }
                    }
                }
                // queen, rook, and bishop all move similairly so theyre lumped together
                _ => {
                    // given a direction, repeatedly move until unable (capture, own piece, edge of board)
                    let mut repeated_moves_on_direction = |dirs: [(i8, i8); 4]| {
                        for (mrow, mcol) in dirs {
                            let mut offset = (mrow, mcol);
                            while let Some(m) =
                                self.generic_move(&square, (row + offset.0, col + offset.1))
                            {
                                let capture = m.capture.is_some();
                                moves.push(m);
                                if capture {
                                    break;
                                }
                                offset.0 += mrow;
                                offset.1 += mcol;
                            }
                        }
                    };
                    // rows and files
                    if piece_some.piece_type == PieceType::Rook
                        || piece_some.piece_type == PieceType::Queen
                    {
                        repeated_moves_on_direction([
                            (1i8, 0i8),
                            (0i8, 1i8),
                            (-1i8, 0i8),
                            (0i8, -1i8),
                        ]);
                    }
                    // diagonals
                    if piece_some.piece_type == PieceType::Bishop
                        || piece_some.piece_type == PieceType::Queen
                    {
                        repeated_moves_on_direction([
                            (1i8, 1i8),
                            (-1i8, 1i8),
                            (1i8, -1i8),
                            (-1i8, -1i8),
                        ]);
                    }
                }
            }
        }
        moves
    }
    pub fn move_piece(&mut self, from: &Square, to: &Square) {
        self.board[to.0 as usize][to.1 as usize] =
            self.board[from.0 as usize][from.1 as usize].take();
    }
    pub fn compute_legal_moves(&mut self) {
        for row in 0i8..8 {
            for col in 0i8..8 {
                self.legal_moves[row as usize][col as usize] = self.compute_legal_moves_on_square((row, col))
            }
        }
    }
    pub fn make_move(&mut self, piece:Piece, mov:Move) {
        // full move clock
        if self.turn == Color::Black {
            self.fullmove_number += 1;
        }
        // half move clock
        if mov.capture.is_some() || piece.piece_type == PieceType::Pawn {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }
        // en passant move
        self.en_passant_target_square = mov.en_passant_target_square;
        // en passant capture
        if let Some(c) = mov.en_passant_capture {
            self.board[c.0 as usize][c.1 as usize] = None;
        }

        // move the piece
        self.move_piece(&mov.from, &mov.to);
        self.turn = match self.turn {
            Color::Black => Color::White,
            Color::White => Color::Black
        };
        self.compute_legal_moves();
    }
    pub fn request_move(&mut self, from:&Square, to:&Square) -> bool {
        // explicitly reject moving the opponent's pieces because obviously
        let piece = self.piece_at_square(from).unwrap();
        if piece.color != self.turn {
            return false
        }
        for mov in self.legal_moves_on_square(*from) {
            if mov.to == *to {
                self.make_move(piece, mov);
                return true
            }
        }
        false
    }
}

const INITIAL_ROW: [PieceType; 8] = [
    PieceType::Rook,
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Queen,
    PieceType::King,
    PieceType::Bishop,
    PieceType::Knight,
    PieceType::Rook,
];

pub fn default_game() -> Game {
    const INIT_PIECE: Option<Piece> = None;
    let mut game = Game {
        board: [[INIT_PIECE; 8]; 8],
        turn: Color::White,
        castling_rights: CastlingRights {
            white_queenside: true,
            white_kingside: true,
            black_queenside: true,
            black_kingside: true,
        },
        en_passant_target_square: None,
        halfmove_clock: 0,
        fullmove_number: 0,
        moves: Default::default(),
        legal_moves: Default::default()
    };
    // initialize top and bottom rows with the starting arrangement
    for (index, piecetype) in INITIAL_ROW.iter().enumerate() {
        game.board[0][index] = Some(Piece {
            piece_type: *piecetype,
            color: Color::Black,
        });
        game.board[7][index] = Some(Piece {
            piece_type: *piecetype,
            color: Color::White,
        });
    }
    // initialize pawns
    for i in 0..8 {
        game.board[1][i] = Some(Piece {
            piece_type: PieceType::Pawn,
            color: Color::Black,
        });
        game.board[6][i] = Some(Piece {
            piece_type: PieceType::Pawn,
            color: Color::White,
        });
    }
    game.compute_legal_moves();
    game
}
