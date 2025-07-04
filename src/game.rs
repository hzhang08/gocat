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
    pub original_sgf: SgfData,
    pub original_sgf_path: Option<String>,
}

impl GoGame {
    pub fn current_triangles(&self) -> &Vec<(usize, usize)> {
        if self.move_idx > 0 && self.move_idx <= self.moves.len() {
            &self.moves[self.move_idx - 1].triangles
        } else {
            static EMPTY: Vec<(usize, usize)> = Vec::new();
            &EMPTY
        }
    }
    pub fn current_triangles_mut(&mut self) -> Option<&mut Vec<(usize, usize)>> {
        if self.move_idx > 0 && self.move_idx <= self.moves.len() {
            Some(&mut self.moves[self.move_idx - 1].triangles)
        } else {
            None
        }
    }
}


impl GoGame {
    pub fn new(sgf: SgfData, sgf_path: Option<String>) -> Self {
        let mut board = vec![vec![Stone::Empty; sgf.board_size]; sgf.board_size];
        for &(x, y) in &sgf.ab {
            board[y][x] = Stone::Black;
        }
        for &(x, y) in &sgf.aw {
            board[y][x] = Stone::White;
        }
        let moves = sgf.moves.clone();
        let metadata = sgf.metadata.clone();

        GoGame {
            board_size: sgf.board_size,
            board,
            moves,
            move_idx: 0,
            metadata,

            original_sgf: sgf,
            original_sgf_path: sgf_path,
        }
    }

    pub fn reset_board(&mut self) {
        self.board = vec![vec![Stone::Empty; self.board_size]; self.board_size];
        for &(x, y) in &self.original_sgf.ab {
            self.board[y][x] = Stone::Black;
        }
        for &(x, y) in &self.original_sgf.aw {
            self.board[y][x] = Stone::White;
        }
    }

    pub fn apply_moves(&mut self, up_to: usize) {
        self.reset_board();
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
