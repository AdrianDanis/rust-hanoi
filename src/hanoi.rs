use std;

pub type Stack = u32;
pub type PieceHeight = u32;

#[derive(Debug, Clone, Copy)]
pub enum Colour {
    Black,
    White,
}

#[derive(Debug, Clone, Copy)]
pub struct PieceState {
    pub stack: Stack,
    pub height: PieceHeight,
}

#[derive(Debug, Clone, Copy)]
pub struct Piece {
    pub state: PieceState,
    pub num: u32,
}

impl Piece {
    pub fn colour(&self) -> Colour {
        match self.num % 2 {
            0 => Colour::Black,
            _ => Colour::White,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Error {
    InvalidStack,
    ZeroPieces,
}

type GameResult<T> = Result<T, Error>;

pub struct PiecesIter<'a> {
    state_enum: std::iter::Enumerate<std::slice::Iter<'a, PieceState>>,
}

impl<'a> Iterator for PiecesIter<'a> {
    type Item = Piece;
    fn next(&mut self) -> Option<Piece> {
        if let Some((num, state)) = self.state_enum.next() {
            Some(Piece{state: *state, num:num as u32})
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct GameState {
    start_stack: Stack,
    num_pieces: u32,
    num_stacks: u32,
    pieces: Vec<PieceState>,
}

impl GameState {
    fn update_piece(&mut self, piece: u32, state: PieceState) {
        self.pieces[piece as usize] = state;
    }
    pub fn new(start: Stack, num_stacks: u32, num_pieces: u32) -> GameResult<GameState> {
        if start >= num_stacks {
            return Err(Error::InvalidStack);
        }
        if num_pieces == 0 {
            return Err(Error::ZeroPieces);
        }
        let mut pieces = vec!();
        for i in 0..num_pieces {
            pieces.push(PieceState{stack: start, height: i})
        }
        Ok(GameState { start_stack: start, num_pieces: num_pieces, pieces: pieces, num_stacks: num_stacks })
    }
    pub fn pieces_iter(&self) -> PiecesIter {
        PiecesIter {state_enum: self.pieces.iter().enumerate() }
    }
    // This should be an iterator
    pub fn num_pieces(&self) -> u32 {
        self.num_pieces
    }
    pub fn get_piece(&self, num: u32) -> Piece {
        Piece {
            state: self.pieces[num as usize],
            num: num,
        }
    }
    pub fn num_stacks(&self) -> u32 {
        self.num_stacks
    }
    pub fn stack_top(&self, stack: Stack) -> Option<Piece> {
        self.pieces_iter()
            // Only want items from the correct stack
            .filter(|ref p| p.state.stack == stack)
            // fold to find the highest one, or none
            .fold(None, |highest, next| highest.map_or(Some(next),
                    |h| if next.state.height > h.state.height { Some(next) } else { highest }
                )
            )
    }
    pub fn valid_stack(&self, s: Stack) -> bool {
        s < self.num_stacks()
    }
    pub fn try_move(&mut self, from: Stack, to: Stack) -> GameResult<bool> {
        if !self.valid_stack(from) || !self.valid_stack(to) {
            return Err(Error::InvalidStack);
        }
        // lookup the from piece
        let from_piece = self.stack_top(from).ok_or(Error::InvalidStack)?;
        if from == to {
            // same stack, do nothing
            Ok(true)
        } else if let Some(highest) = self.stack_top(to) {
            // check if we are smaller (i.e. have a larger num) than the destination
            if from_piece.num > highest.num {
                self.update_piece(from_piece.num, PieceState{stack: to, height: highest.state.height + 1});
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            // destination empty
            self.update_piece(from_piece.num, PieceState{stack: to, height: 0});
            Ok(true)
        }
    }
    pub fn complete(&self) -> bool {
        let test_stack = self.pieces[0].stack;
        test_stack != self.start_stack && self.pieces.iter().all(|&x| x.stack == test_stack)
    }
}
