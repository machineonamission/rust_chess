use colored::*;
use std::fmt::{Display, Formatter};
use std::time::Instant;

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
pub enum Castling {
    BlackKingside,
    BlackQueenside,
    WhiteKingside,
    WhiteQueenside,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub fn invert(&self) -> Self {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

// (row, col)
pub type Square = (i8, i8);

#[derive(Clone, Copy, Debug)]
pub struct CastlingRights {
    pub white_queenside: bool,
    pub white_kingside: bool,
    pub black_queenside: bool,
    pub black_kingside: bool,
}

impl Default for CastlingRights {
    fn default() -> Self {
        CastlingRights {
            white_queenside: true,
            white_kingside: true,
            black_queenside: true,
            black_kingside: true,
        }
    }
}

pub struct Game {
    pub board: [[Option<Piece>; 8]; 8],
    pub turn: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_target_square: Option<Square>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
    pub moves: Vec<Move>,
    pub legal_moves: [[Vec<Move>; 8]; 8],
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

#[derive(Clone, Debug)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub capture: Option<PieceType>,
    pub castle: Option<Castling>,
    pub losing_castle_rights: CastlingRights,
    pub promotion: Option<PieceType>,
    // if the move was en passant
    pub en_passant_capture: Option<Square>,
    // what square did the pawn double move over
    pub en_passant_target_square: Option<Square>,
    // half move clock
    pub halfmove_clock: u8,
}

impl Default for Move {
    fn default() -> Self {
        Move {
            from: (0, 0),
            to: (0, 0),
            capture: None,
            castle: None,
            losing_castle_rights: CastlingRights {
                white_queenside: false,
                white_kingside: false,
                black_queenside: false,
                black_kingside: false,
            },
            promotion: None,
            en_passant_capture: None,
            en_passant_target_square: None,
            halfmove_clock: 0,
        }
    }
}

pub fn is_valid_square((row, col): &Square) -> Option<Square> {
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
                halfmove_clock: self.halfmove_clock + 1,
                ..Default::default()
            }),
            Some(capture_piece) => {
                if capture_piece.color != color {
                    Some(Move {
                        from: *from,
                        to,
                        capture: Some(capture_piece.piece_type),
                        ..Default::default()
                    })
                } else {
                    None
                }
            }
        }
    }
    pub fn legal_moves_on_square(&self, square: Square) -> &Vec<Move> {
        &self.legal_moves[square.0 as usize][square.1 as usize]
    }

    fn compute_legal_moves_on_square(&self, square: Square) -> Vec<Move> {
        let piece = self.piece_at_square(&square);
        let mut moves = vec![];
        if let Some(piece_some) = piece {
            if piece_some.color != self.turn {
                return moves;
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
                                        ..Default::default()
                                    });
                                }
                                // no piece but en passant time
                            } else if Some(capture_square) == self.en_passant_target_square {
                                pawn_moves.push(Move {
                                    from: square,
                                    to: capture_square,
                                    capture: Some(PieceType::Pawn),
                                    en_passant_capture: Some((row, col + capture_direction)),
                                    ..Default::default()
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
                            ..Default::default()
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
                                    en_passant_target_square: Some((row + direction, col)),
                                    ..Default::default()
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
                    // get castling rights for our color
                    let castling_queenside = match piece_some.color {
                        Color::Black => self.castling_rights.black_queenside,
                        Color::White => self.castling_rights.white_queenside,
                    };
                    let castling_kingside = match piece_some.color {
                        Color::Black => self.castling_rights.black_kingside,
                        Color::White => self.castling_rights.white_kingside,
                    };
                    // update castling rights only if needed so we can unmake move
                    let lose_all_castling = CastlingRights {
                        white_queenside: piece_some.color == Color::White
                            && self.castling_rights.white_queenside,
                        white_kingside: piece_some.color == Color::White
                            && self.castling_rights.white_kingside,
                        black_queenside: piece_some.color == Color::Black
                            && self.castling_rights.black_queenside,
                        black_kingside: piece_some.color == Color::Black
                            && self.castling_rights.black_kingside,
                    };
                    if castling_kingside {
                        // king and rook will be in valid positions if true, just check if inbetween is empty
                        if self.piece_at_square(&(row, 5i8)).is_none()
                            && self.piece_at_square(&(row, 6i8)).is_none()
                        {
                            moves.push(Move {
                                from: square,
                                to: (row, 6i8),
                                castle: Some(match piece_some.color {
                                    Color::Black => Castling::BlackKingside,
                                    Color::White => Castling::WhiteKingside,
                                }),
                                losing_castle_rights: lose_all_castling,
                                halfmove_clock: self.halfmove_clock + 1,
                                ..Default::default()
                            })
                        }
                    }
                    if castling_queenside {
                        // king and rook will be in valid positions if true, just check if inbetween is empty
                        if self.piece_at_square(&(row, 1i8)).is_none()
                            && self.piece_at_square(&(row, 2i8)).is_none()
                            && self.piece_at_square(&(row, 3i8)).is_none()
                        {
                            moves.push(Move {
                                from: square,
                                to: (row, 2i8),
                                castle: Some(match piece_some.color {
                                    Color::Black => Castling::BlackQueenside,
                                    Color::White => Castling::WhiteQueenside,
                                }),
                                losing_castle_rights: lose_all_castling,
                                ..Default::default()
                            })
                        }
                    }
                }
                // queen, rook, and bishop all move similairly so theyre lumped together
                _ => {
                    // given a direction, repeatedly move until unable (capture, own piece, edge of board)
                    let mut repeated_moves_on_direction = |dirs: [(i8, i8); 4]| {
                        // so we aren't computing this constantly
                        let rook = piece_some.piece_type == PieceType::Rook;

                        for (mrow, mcol) in dirs {
                            let mut offset = (mrow, mcol);
                            while let Some(mut m) =
                                self.generic_move(&square, (row + offset.0, col + offset.1))
                            {
                                let capture = m.capture.is_some();

                                // handle castling rights
                                if rook {
                                    match piece_some.color {
                                        Color::Black => {
                                            if row == 0 {
                                                match col {
                                                    0 => {
                                                        m.losing_castle_rights.black_queenside =
                                                            self.castling_rights.black_queenside
                                                    }

                                                    7 => {
                                                        m.losing_castle_rights.black_kingside =
                                                            self.castling_rights.black_kingside
                                                    }

                                                    _ => {}
                                                }
                                            }
                                        }
                                        Color::White => {
                                            if row == 7 && (col == 0 || col == 7) {
                                                match col {
                                                    0 => {
                                                        m.losing_castle_rights.white_queenside =
                                                            self.castling_rights.white_queenside
                                                    }

                                                    7 => {
                                                        m.losing_castle_rights.white_kingside =
                                                            self.castling_rights.white_kingside
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }

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
    fn move_piece(&mut self, from: &Square, to: &Square) {
        self.board[to.0 as usize][to.1 as usize] =
            self.board[from.0 as usize][from.1 as usize].take();
    }
    fn any_king_captures(&self) -> bool {
        for row2 in 0i8..8 {
            for col2 in 0i8..8 {
                for mv2 in self.legal_moves_on_square((row2, col2)) {
                    if let Some(c) = mv2.capture {
                        if c == PieceType::King {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    fn validate_move(&mut self, mov: &Move) -> bool {
        self.make_move(mov);
        self.compute_legal_moves(false);
        let caps = self.any_king_captures();
        self.unmake_move();
        self.compute_legal_moves(false);
        // if caps {
        //     dbg!(mov);
        // }
        !caps
    }
    fn compute_legal_moves(&mut self, validate_king_moves: bool) {
        let now = Instant::now();
        let mut legal_moves: [[Vec<Move>; 8]; 8] = Default::default();
        for row in 0i8..8 {
            for col in 0i8..8 {
                // compute moves normally
                let mut square_legal_moves = self.compute_legal_moves_on_square((row, col));

                if validate_king_moves {
                    // let before = square_legal_moves.len();
                    square_legal_moves.retain(|m| self.validate_move(m));
                    // println!("{} {}", before, legal_moves.len());
                }

                legal_moves[row as usize][col as usize] = square_legal_moves;
            }
        }
        self.legal_moves = legal_moves;

        if validate_king_moves {
            let elapsed = now.elapsed();
            println!("Move computing took {:?}", elapsed);
        }
    }
    fn make_move(&mut self, mov: &Move) {
        // full move clock
        if self.turn == Color::Black {
            self.fullmove_number += 1;
        }
        // half move clock
        self.halfmove_clock = mov.halfmove_clock;
        // en passant move
        self.en_passant_target_square = mov.en_passant_target_square;
        // en passant capture
        if let Some(c) = mov.en_passant_capture {
            self.board[c.0 as usize][c.1 as usize] = None;
        }
        // castling rook
        if let Some(c) = mov.castle {
            match c {
                Castling::BlackKingside => self.move_piece(&(0i8, 7i8), &(0i8, 5i8)),
                Castling::BlackQueenside => self.move_piece(&(0i8, 0i8), &(0i8, 3i8)),
                Castling::WhiteKingside => self.move_piece(&(7i8, 7i8), &(7i8, 5i8)),
                Castling::WhiteQueenside => self.move_piece(&(7i8, 0i8), &(7i8, 3i8)),
            }
        }
        // castling rights
        self.castling_rights.black_queenside &= !mov.losing_castle_rights.black_queenside;
        self.castling_rights.black_kingside &= !mov.losing_castle_rights.black_kingside;
        self.castling_rights.white_queenside &= !mov.losing_castle_rights.white_queenside;
        self.castling_rights.white_kingside &= !mov.losing_castle_rights.white_kingside;
        // promotion
        if let Some(p) = mov.promotion {
            self.board[mov.from.0 as usize][mov.from.1 as usize]
                .unwrap()
                .piece_type = p;
        }
        // move the piece
        self.move_piece(&mov.from, &mov.to);
        // push move
        self.moves.push(mov.clone());
        // update turn
        self.turn = self.turn.invert();
    }
    fn unmake_move(&mut self) -> bool {
        let last_mov = self.moves.pop();
        if last_mov.is_none() {
            return false;
        }
        self.turn = self.turn.invert();
        let mov = last_mov.unwrap();
        self.move_piece(&mov.to, &mov.from);

        // full move clock
        if self.turn == Color::Black {
            self.fullmove_number -= 1;
        }
        // half move clock
        self.halfmove_clock = match self.moves.last() {
            None => 0,
            Some(mv) => mv.halfmove_clock,
        };

        // en passant move
        let last_move = self.moves.last();
        if let Some(lm) = last_move {
            self.en_passant_target_square = lm.en_passant_target_square;
        } else {
            self.en_passant_target_square = None;
        }

        // en passant capture
        if let Some(c) = mov.en_passant_capture {
            self.board[c.0 as usize][c.1 as usize] = Some(Piece {
                piece_type: PieceType::Pawn,
                color: self.turn.invert(),
            });
        } else if let Some(c) = mov.capture {
            self.board[mov.to.0 as usize][mov.to.1 as usize] = Some(Piece {
                piece_type: c,
                color: self.turn.invert(),
            });
        }
        // castling rook
        if let Some(c) = mov.castle {
            match c {
                Castling::BlackKingside => self.move_piece(&(0i8, 5i8), &(0i8, 7i8)),
                Castling::BlackQueenside => self.move_piece(&(0i8, 3i8), &(0i8, 0i8)),
                Castling::WhiteKingside => self.move_piece(&(7i8, 5i8), &(7i8, 7i8)),
                Castling::WhiteQueenside => self.move_piece(&(7i8, 3i8), &(7i8, 0i8)),
            }
        }
        // castling rights
        self.castling_rights.black_queenside |= mov.losing_castle_rights.black_queenside;
        self.castling_rights.black_kingside |= mov.losing_castle_rights.black_kingside;
        self.castling_rights.white_queenside |= mov.losing_castle_rights.white_queenside;
        self.castling_rights.white_kingside |= mov.losing_castle_rights.white_kingside;
        // promotion
        if mov.promotion.is_some() {
            self.board[mov.from.0 as usize][mov.from.1 as usize]
                .unwrap()
                .piece_type = PieceType::Pawn;
        }
        true
    }
    pub fn unmake_move_and_recalculate(&mut self) {
        self.unmake_move();
        // recompute legal moves
        self.compute_legal_moves(true);
    }
    pub fn request_move(&mut self, from: &Square, to: &Square) -> bool {
        // clone here because I can't borrow self in self.legal_moves_on_square and self.make_move
        for mov in self.legal_moves_on_square(*from).clone() {
            if mov.to == *to {
                self.make_move(&mov);
                self.compute_legal_moves(true);
                return true;
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

impl Default for Game {
    fn default() -> Self {
        const INIT_PIECE: Option<Piece> = None;
        let mut game = Game {
            board: [[INIT_PIECE; 8]; 8], // empty board
            turn: Color::White,
            castling_rights: Default::default(), // all true
            en_passant_target_square: None,
            halfmove_clock: 0,
            fullmove_number: 0,
            moves: Default::default(),       // empty vec
            legal_moves: Default::default(), // empty vec
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
        game.compute_legal_moves(true);
        game
    }
}
