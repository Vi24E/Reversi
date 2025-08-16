use crate::proto::{Color, Move, PlayerStat, RecvCommand, Wl};

macro_rules! rule {
    () => {};
    (
        $name:ident ($input:ident) -> $ty:ty,
        expect: $expect:literal {
            $($tt:tt)*
        }
        $($rest:tt)*
    ) => {
        #[inline]
        fn $name<'input>(it: &mut impl Iterator<Item = &'input str>) -> Result<$ty, String> {
            let $input = it.next().ok_or(format!(concat!("Expected ", $expect, ", got EOF")))?;
            $($tt)*
            #[allow(unreachable_code)]
            Err(format!(concat!("Expected ", $expect, ", got `{}`"), $input))
        }
        rule!($($rest)*);
    };
}

rule! {
    parse_wl (input) -> Wl, expect: "win/lose/tie" {
        match input {
            "WIN" => return Ok(Wl::Win),
            "LOSE" => return Ok(Wl::Lose),
            "TIE" => return Ok(Wl::Tie),
            _ => (),
        };
    }

    parse_wb (input) -> Color, expect: "black/white" {
        match input {
            "BLACK" => return Ok(Color::Black),
            "WHITE" => return Ok(Color::White),
            _ => (),
        };
    }

    parse_mv (input) -> Move, expect: "move" {
        if input == "PASS" {
            return Ok(Move::Pass);
        }
        if input == "GIVEUP" {
            return Ok(Move::GiveUp);
        }
        if input.len() == 2 {
            let mut chars = input.chars();
            let x = chars.next().unwrap();
            let y = chars.next().unwrap();
            if matches!(x, 'A'..='H') && matches!(y, '1'..='8') {
                let x_ah = x as u32 - 'A' as u32 + 1;
                let y_18 = y as u32 - '1' as u32 + 1;
                return Ok(Move::Mv { x_ah, y_18 });
            }
        }
    }

    parse_int (input) -> i32, expect: "integer" {
        if let Ok(score) = input.parse() {
            return Ok(score);
        }
    }

    parse_uint (input) -> u32, expect: "unsigned integer" {
        if let Ok(score) = input.parse() {
            return Ok(score);
        }
    }

    parse_string (input) -> String, expect: "string" {
        return Ok(input.to_string());
    }
}

pub fn parse(s: &str) -> Result<RecvCommand, String> {
    let mut iter = s.split_whitespace().peekable();
    let cmd = iter.next().ok_or("Empty command")?;
    match cmd {
        "START" => {
            let color = parse_wb(&mut iter)?;
            let opponent_name = parse_string(&mut iter)?;
            let assigned_time_ms = parse_int(&mut iter)?;
            Ok(RecvCommand::Start {
                color,
                opponent_name,
                assigned_time_ms,
            })
        }
        "END" => {
            let result = parse_wl(&mut iter)?;
            let your_stone_count = parse_uint(&mut iter)?;
            let opponent_stone_count = parse_uint(&mut iter)?;
            let reason = parse_string(&mut iter)?;
            Ok(RecvCommand::End {
                result,
                your_stone_count,
                opponent_stone_count,
                reason,
            })
        }
        "MOVE" => {
            let mv = parse_mv(&mut iter)?;
            Ok(RecvCommand::Move(mv))
        }
        "ACK" => {
            let assigned_time_ms = parse_int(&mut iter)?;
            Ok(RecvCommand::Ack { assigned_time_ms })
        }
        "BYE" => {
            let mut stat = Vec::new();
            while iter.peek().is_some() {
                let player_name = parse_string(&mut iter)?;
                let score = parse_int(&mut iter)?;
                let wins = parse_uint(&mut iter)?;
                let loses = parse_uint(&mut iter)?;
                stat.push(PlayerStat {
                    player_name,
                    score,
                    wins,
                    loses,
                });
            }
            Ok(RecvCommand::Bye { stat })
        }
        _ => Err("Invalid command")?,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_recvs() {
        let res = parse("START BLACK alice 500000");
        assert!(res.is_ok(), "{res:?}");
        let res = parse("ACK  499995");
        assert!(res.is_ok(), "{res:?}");
        let res = parse("ACK  -100");
        assert!(res.is_ok(), "{res:?}");
        let res = parse("MOVE G7");
        assert!(res.is_ok(), "{res:?}");
        let res = parse("BYE Anon1 -4 0 4");
        assert!(res.is_ok(), "{res:?}");
        let res = parse("BYE Anon1 -4 0 4 Anon2 4 4 0");
        assert!(res.is_ok(), "{res:?}");
        let res = parse("END TIE 0 0 INVALID_COMMAND");
        assert!(res.is_ok(), "{res:?}");
    }
}
