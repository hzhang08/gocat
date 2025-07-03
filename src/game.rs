use crate::sgf_parser::{SgfData, Player, Move};

#[derive(Clone, Copy, PartialEq)]
pub enum Stone {
    Empty,
    Black,
    White,
}

pub struct GoGame {
    pub board_size: usize,
    pub board: Vec<Vec<Stone>>,
    pub moves: Vec<Move>,
    pub move_idx: usize,
    pub metadata: Vec<(String, String)>,
}

impl GoGame {
    pub fn new(sgf: SgfData) -> Self {
        let mut board = vec![vec![Stone::Empty; sgf.board_size]; sgf.board_size];
        for &(x, y) in &sgf.ab {
            board[y][x] = Stone::Black;
        }
        for &(x, y) in &sgf.aw {
            board[y][x] = Stone::White;
        }
        GoGame {
            board_size: sgf.board_size,
            board,
            moves: sgf.moves,
            move_idx: 0,
            metadata: sgf.metadata,
        }
    }

    pub fn reset_board(&mut self, sgf: &SgfData) {
        self.board = vec![vec![Stone::Empty; self.board_size]; self.board_size];
        for &(x, y) in &sgf.ab {
            self.board[y][x] = Stone::Black;
        }
        for &(x, y) in &sgf.aw {
            self.board[y][x] = Stone::White;
        }
    }

    pub fn apply_moves(&mut self, up_to: usize) {
        self.reset_board(&SgfData {
            board_size: self.board_size,
            moves: vec![],
            ab: self.board.iter().enumerate().flat_map(|(y, row)| row.iter().enumerate().filter_map(move |(x, &s)| if s == Stone::Black { Some((x, y)) } else { None }).collect::<Vec<_>>()).collect(),
            aw: self.board.iter().enumerate().flat_map(|(y, row)| row.iter().enumerate().filter_map(move |(x, &s)| if s == Stone::White { Some((x, y)) } else { None }).collect::<Vec<_>>()).collect(),
            metadata: self.metadata.clone(),
        });
        for i in 0..up_to {
            if let Some(mv) = self.moves.get(i) {
                self.board[mv.y][mv.x] = match mv.player {
                    Player::Black => Stone::Black,
                    Player::White => Stone::White,
                };
            }
        }
    }

    pub fn next_move(&mut self) {
        if self.move_idx < self.moves.len() {
            self.move_idx += 1;
            self.apply_moves(self.move_idx);
        }
    }

    pub fn prev_move(&mut self) {
        if self.move_idx > 0 {
            self.move_idx -= 1;
            self.apply_moves(self.move_idx);
        }
    }

    pub fn current_player(&self) -> Player {
        if self.move_idx < self.moves.len() {
            self.moves[self.move_idx].player.clone()
        } else {
            Player::Black
        }
    }
}
