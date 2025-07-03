use thiserror::Error;

#[derive(Debug, Error)]
pub enum SgfParseError {
    #[error("Invalid SGF format")]
    InvalidFormat,
    #[error("Missing board size (SZ) property")]
    MissingBoardSize,
}

#[derive(Debug, Clone)]
pub enum Player {
    Black,
    White,
}

#[derive(Debug, Clone)]
pub struct Move {
    pub player: Player,
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone)]
pub struct SgfData {
    pub board_size: usize,
    pub moves: Vec<Move>,
    pub ab: Vec<(usize, usize)>, // Add Black stones
    pub aw: Vec<(usize, usize)>, // Add White stones
    pub metadata: Vec<(String, String)>,
}

pub fn parse_sgf(sgf: &str) -> Result<SgfData, SgfParseError> {
    // Very minimal SGF parser for basic Go games
    let mut board_size = None;
    let mut moves = Vec::new();
    let mut ab = Vec::new();
    let mut aw = Vec::new();
    let mut metadata = Vec::new();

    let prop_re = regex::Regex::new(r"([A-Z]+)\[([^\]]*)\]").unwrap();
    for cap in prop_re.captures_iter(sgf) {
        let key = &cap[1];
        let value = &cap[2];
        match key {
            "SZ" => {
                board_size = value.parse::<usize>().ok();
            }
            "AB" => {
                if value.len() == 2 {
                    ab.push(sgf_coords_to_xy(value));
                }
            }
            "AW" => {
                if value.len() == 2 {
                    aw.push(sgf_coords_to_xy(value));
                }
            }
            "B" | "W" => {
                if value.len() == 2 {
                    let (x, y) = sgf_coords_to_xy(value);
                    let player = if key == "B" { Player::Black } else { Player::White };
                    moves.push(Move { player, x, y });
                }
            }
            _ => {
                if value.len() > 0 {
                    metadata.push((key.to_string(), value.to_string()));
                }
            }
        }
    }

    let board_size = board_size.ok_or(SgfParseError::MissingBoardSize)?;
    Ok(SgfData { board_size, moves, ab, aw, metadata })
}

fn sgf_coords_to_xy(s: &str) -> (usize, usize) {
    // SGF coords: 'aa' = (0,0), 'ab' = (0,1), etc.
    let bytes = s.as_bytes();
    (
        (bytes[0] - b'a') as usize,
        (bytes[1] - b'a') as usize,
    )
}
