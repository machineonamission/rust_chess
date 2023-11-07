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
}

impl Display for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (row, prow) in self.board.iter().enumerate() {
            for (col, piece) in prow.iter().enumerate() {
                let piecestring = match piece {
                    Some(p) => {
                        match p.color {
                            Color::Black => {
                                match p.piece_type {
                                    PieceType::Pawn => { "p" }
                                    PieceType::Knight => { "n" }
                                    PieceType::Bishop => { "b" }
                                    PieceType::Rook => { "r" }
                                    PieceType::Queen => { "q" }
                                    PieceType::King => { "k" }
                                }
                            }
                            Color::White => {
                                match p.piece_type {
                                    PieceType::Pawn => { "P" }
                                    PieceType::Knight => { "N" }
                                    PieceType::Bishop => { "B" }
                                    PieceType::Rook => { "R" }
                                    PieceType::Queen => { "Q" }
                                    PieceType::King => { "K" }
                                }
                            }
                        }
                    }
                    None => {
                        " "
                    }
                };
                let colored = if row % 2 == col % 2 {
                    piecestring.on_truecolor(0xb5, 0x88, 0x63).black()
                } else {
                    piecestring.on_truecolor(0xf0, 0xd9, 0xb5).black()
                };
                f.write_fmt(
                    format_args!("{}", colored)
                )?;
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
    pub en_passant_capture: bool, // if the move was en passant
    pub en_passant_able: bool // if the pawn moved 2 squares
}

pub fn is_valid_square(row: i8, col: i8) -> Option<Square> {
    if (0i8..8i8).contains(&row) && (0i8..8i8).contains(&col) {
        Some((row, col))
    } else {
        None
    }
}


impl Game {
    pub fn piece_at_square(&self, square: &Square) -> &Option<Piece> {
        match is_valid_square(square.0, square.1) {
            Some((row,col)) => {
                &self.board[row as usize][col as usize]
            }
            None => {
                &None
            }
        }
    }
    fn generic_move(&self, from: &Square, to: Square) -> Option<Move> {
        // return no move if invalid
        is_valid_square(to.0, to.1)?;
        // unwrap is fine here because from should always be valid
        let color = self.piece_at_square(from).as_ref().unwrap().color;
        let capture = self.piece_at_square(&to);
        match capture {
            None => {
                Some(Move {
                    from: *from,
                    to,
                    capture: None,
                    castle: false,
                    promotion: None,
                    en_passant_capture: false,
                    en_passant_able: false,
                })
            }
            Some(capture_piece) => {
                if capture_piece.color != color {
                    Some(Move {
                        from: *from,
                        to,
                        capture: Some(capture_piece.piece_type),
                        castle: false,
                        promotion: None,
                        en_passant_capture: false,
                        en_passant_able: false,
                    })
                } else {
                    None
                }
            }
        }
    }

    pub fn legal_moves_on_square(&self, square: Square) -> Vec<Move> {
        let piece = self.piece_at_square(&square);
        let mut moves = vec!();
        if let Some(piece_some) = piece {
            let (row, col) = square;
            match piece_some.piece_type {
                PieceType::Pawn => {
                    // TODO: en passant
                    // if to increase row or decrease row
                    let direction: i8 = match piece_some.color {
                        Color::Black => { 1 }
                        Color::White => { -1 }
                    };
                    let mut pawn_moves: Vec<Move> = vec!();
                    let torow = row + direction;
                    // diagonal captures
                    for capture_direction in [-1i8, 1i8] {
                        // if the diagonal is a valid square
                        if let Some(capture_square) = is_valid_square(torow, col + capture_direction) {
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
                                        en_passant_capture: false,
                                        en_passant_able: false,
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
                                    en_passant_capture: true,
                                    en_passant_able: false,
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
                            en_passant_capture: false,
                            en_passant_able: false,
                        });
                        // this can only happen if the last square was empty and pawns at initial rows
                        // pawns cant move backwards nor jump over other pieces
                        if (row == 6 && piece_some.color == Color::White) || (row == 1 && piece_some.color == Color::Black) {
                            // always valid square
                            let two_ahead = (row + direction * 2, col);
                            if self.piece_at_square(&two_ahead).is_none() {
                                pawn_moves.push(Move {
                                    from: square,
                                    to: two_ahead,
                                    capture: None,
                                    castle: false,
                                    promotion: None,
                                    en_passant_capture: false,
                                    en_passant_able: true,
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
                    // knight can move any combination of 2 and 1, positive or negative
                    let mut knight_moves: [Square; 8] = [(0, 0); 8];
                    let mut i: usize = 0;
                    for big in [-2i8, 2i8] {
                        for small in [-1i8, 1i8] {
                            knight_moves[i] = (big, small);
                            i += 1;
                            knight_moves[i] = (small, big);
                            i += 1;
                        }
                    }
                    for mov in knight_moves {
                        if let Some(m) = self.generic_move(&square, (row + mov.0, col + mov.1)) {
                            moves.push(m);
                        }
                    }
                }
                PieceType::King => {
                    let mut king_moves: [Square; 8] = [(0, 0); 8];
                    let mut i: usize = 0;
                    for krow in -1i8..2i8 {
                        for kcol in -1i8..2i8 {
                            if krow != 0 || kcol != 0 {
                                king_moves[i] = (krow, kcol);
                                i += 1;
                            }
                        }
                    }
                    for mov in king_moves {
                        if let Some(m) = self.generic_move(&square, (row + mov.0, col + mov.1)) {
                            moves.push(m);
                        }
                    }
                }
                // queen, rook, and bishop all move similairly so theyre lumped together
                _ => {
                    if piece_some.piece_type == PieceType::Rook || piece_some.piece_type == PieceType::Queen {
                        for (mrow, mcol) in [(1i8, 0i8), (0i8, 1i8), (-1i8, 0i8), (0i8, -1i8)] {
                            let mut offset = (mrow, mcol);
                            while let Some(m) = self.generic_move(&square, (row + offset.0, col + offset.1)) {
                                let capture = m.capture.is_some();
                                moves.push(m);
                                if capture {
                                    break;
                                }
                                offset.0 += mrow;
                                offset.1 += mcol;
                            }
                        }
                    }
                    if piece_some.piece_type == PieceType::Bishop || piece_some.piece_type == PieceType::Queen {
                        for (mrow, mcol) in [(1i8, 1i8), (-1i8, 1i8), (1i8, -1i8), (-1i8, -1i8)] {
                            let mut offset = (mrow, mcol);
                            while let Some(m) = self.generic_move(&square, (row + offset.0, col + offset.1)) {
                                let capture = m.capture.is_some();
                                moves.push(m);
                                if capture {
                                    break;
                                }
                                offset.0 += mrow;
                                offset.1 += mcol;
                            }
                        }
                    }
                }
            }
        }
        moves
    }
    pub fn move_piece(&mut self, from: &Square, to: &Square) {
        self.board[to.0 as usize][to.1 as usize] = self.board[from.0 as usize][from.1 as usize].take();
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
    PieceType::Rook
];

pub fn default_game() -> Game {
    const INIT: Option<Piece> = None;
    let mut game = Game {
        board: [[INIT; 8]; 8],
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
        moves: vec!(),
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
    game
}