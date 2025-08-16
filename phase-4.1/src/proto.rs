use std::{
    char::from_u32,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Color {
    None,
    White,
    Black,
    Sentinel,
}

impl Color {
    pub fn opposite(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
            _ => self,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Color::None => write!(f, "None"),
            Color::White => write!(f, "White"),
            Color::Black => write!(f, "Black"),
            Color::Sentinel => write!(f, "Sentinel"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Wl {
    Win,
    Lose,
    Tie,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Move {
    Mv { x_ah: u32, y_18: u32 },
    Pass,
    GiveUp,
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Pass => write!(f, "PASS"),
            Self::GiveUp => write!(f, "GIVEUP"),
            Self::Mv { x_ah, y_18 } => {
                let cx = from_u32(x_ah + ('A' as u32) - 1).unwrap();
                let cy = from_u32(y_18 + ('1' as u32) - 1).unwrap();
                write!(f, "{cx}{cy}")
            }
        }
    }
}

pub enum SendCommand<'a> {
    Open { player_name: &'a str },
    Move(Move),
}

impl<'a> Display for SendCommand<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Open { player_name } => writeln!(f, "OPEN {player_name}"),
            Self::Move(m) => writeln!(f, "MOVE {m}"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum RecvCommand {
    Start {
        color: Color,
        opponent_name: String,
        assigned_time_ms: i32,
    },
    Move(Move),
    Ack {
        assigned_time_ms: i32,
    },
    End {
        result: Wl,
        /// n
        your_stone_count: u32,
        /// m
        opponent_stone_count: u32,
        reason: String,
    },
    Bye {
        stat: Vec<PlayerStat>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct PlayerStat {
    pub player_name: String,
    pub score: i32,
    pub wins: u32,
    pub loses: u32,
}

impl Display for PlayerStat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} (Win {}, Lose {})",
            self.player_name, self.score, self.wins, self.loses
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_send_command() {
        use self::Move::*;
        use SendCommand::*;
        assert_eq!(
            Open {
                player_name: "Anon."
            }
            .to_string(),
            "OPEN Anon.\n".to_string()
        );
        assert_eq!(
            Move(Mv { x_ah: 3, y_18: 4 }).to_string(),
            "MOVE C4\n".to_string()
        );
    }
}
