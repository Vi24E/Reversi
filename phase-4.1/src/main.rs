use getopts::Options;
use std::env;
use std::fmt::Display;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::process;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;

mod command_parser;
mod database;
mod play;
mod proto;
mod eval;
mod solver;

use crate::play::{Board, InitGame};
use crate::proto::{Color, Move, PlayerStat, RecvCommand, SendCommand, Wl};

#[derive(Debug, Error)]
enum Error {
    #[error("couldn't read/write via IO stream `{0}`")]
    IO(#[from] std::io::Error),
    #[error("parse error: {0:?}")]
    Parse(String),
    #[error("received invalid command `{0:?}`")]
    Recv(RecvCommand),
}

type Result<T, E = Error> = std::result::Result<T, E>;

struct MyOptions {
    socket_addr: SocketAddr,
    player: String,
    verbose: bool,
}

struct Logger {
    enabled: bool,
}

impl Logger {
    fn new(opt: &MyOptions) -> Self {
        Self {
            enabled: opt.verbose,
        }
    }
    fn received(&self, s: &str) {
        if self.enabled {
            print!("Received: {s}");
        }
    }
    fn sent(&self, s: &str) {
        if self.enabled {
            print!("Sent: {s}");
        }
    }
    fn log(&self, s: impl Display) {
        if self.enabled {
            print!("{s}");
        }
    }
}

enum State {
    WaitStart,
    MyTurn(Option<InitGame>),
    OpTurn(Option<InitGame>),
    EndGame {
        result: Wl,
        your_stone_count: u32,
        opponent_stone_count: u32,
        reason: String,
    },
    Exit {
        stat: Vec<PlayerStat>,
    },
}

fn print_usage(program: &str, opts: &Options) -> ! {
    let brief = format!("Usage: {program} -H HOST -p PORT -n PLAYERNAME");
    print!("{}", opts.usage(&brief));
    process::exit(0);
}

fn parse_args() -> MyOptions {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];

    let mut opts = Options::new();
    opts.optopt("H", "host", "set server host", "HOST");
    opts.optopt("p", "port", "set server port", "PORT");
    opts.optopt("n", "name", "set player name", "PLAYERNAME");
    opts.optflag("v", "verbose", "verbose output");
    opts.optflag("h", "help", "print this help menu");

    let matches = opts.parse(&args[1..]).unwrap_or_else(|fail| {
        println!("{fail}");
        print_usage(program, &opts);
    });
    if matches.opt_present("h") {
        print_usage(program, &opts);
    }

    let host = matches
        .opt_str("H")
        .unwrap_or_else(|| "127.0.0.1".to_string());

    let port = matches.opt_str("p").map_or(3000, |s| s.parse().unwrap());

    let addr = (host, port)
        .to_socket_addrs()
        .expect("hostname must be known")
        .next()
        .expect("hostname must be valid");

    MyOptions {
        socket_addr: addr,
        player: matches.opt_str("n").unwrap_or_else(|| "Anon.".to_string()),
        verbose: matches.opt_present("v"),
    }
}

fn receive_command(reader: &mut BufReader<&TcpStream>, logger: &mut Logger) -> Result<RecvCommand> {
    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    logger.received(&buf);
    let rec = command_parser::parse(buf.as_str()).map_err(Error::Parse)?;
    Ok(rec)
}

fn send_command(
    writer: &mut BufWriter<&TcpStream>,
    logger: &mut Logger,
    command: &SendCommand,
) -> Result<()> {
    let s = command.to_string();
    writer.write_all(s.as_bytes())?;
    writer.flush()?;
    logger.sent(&s);
    Ok(())
}

fn wait_start(reader: &mut BufReader<&TcpStream>, logger: &mut Logger) -> Result<State> {
    match receive_command(reader, logger)? {
        RecvCommand::Bye { stat } => {
            println!("Finished connection.");
            Ok(State::Exit { stat })
        }
        RecvCommand::Start {
            color: Color::Black,
            opponent_name,
            assigned_time_ms,
        } => Ok(State::MyTurn(Some(InitGame {
            opponent_name,
            assigned_time_ms,
        }))),
        RecvCommand::Start {
            color: Color::White,
            opponent_name,
            assigned_time_ms,
        } => Ok(State::OpTurn(Some(InitGame {
            opponent_name,
            assigned_time_ms,
        }))),
        r => Err(Error::Recv(r)),
    }
}

fn print_scores(logger: &mut Logger, stat: impl Iterator<Item = PlayerStat>) {
    let v: Vec<_> = stat.map(|player_stat| format!("{player_stat}\n")).collect();
    logger.log(v.concat());
}

fn format_move(m : usize) -> Move {
    match m {
        64 => Move::Pass,
        _ => {
            let x = (m & 7) as u32;
            let y = (m >> 3) as u32;
            Move::Mv {
                x_ah: x + 1,
                y_18: y + 1,
            }
        }
    }
}

fn my_move(
    reader: &mut BufReader<&TcpStream>,
    writer: &mut BufWriter<&TcpStream>,
    logger: &mut Logger,
    board: &mut Board,
    player_color: Color,
    assigned_time_ms: &mut i32,
) -> Result<State> {
    let mv = format_move(board.decide_move(*assigned_time_ms));
    board.do_move_interface(mv, true);

    //　以下変更しない
    send_command(writer, logger, &SendCommand::Move(mv))?;
    logger.log(board);

    match receive_command(reader, logger)? {
        RecvCommand::Ack {
            assigned_time_ms: updated,
        } => {
            *assigned_time_ms = updated;
            Ok(State::OpTurn(None))
        }
        RecvCommand::End {
            result,
            your_stone_count,
            opponent_stone_count,
            reason,
        } => Ok(State::EndGame {
            result,
            your_stone_count,
            opponent_stone_count,
            reason,
        }),
        r => Err(Error::Recv(r)),
    }
}

//変更しない
fn op_move(
    reader: &mut BufReader<&TcpStream>,
    logger: &mut Logger,
    board: &mut Board,
    _player_color: Color,
) -> Result<State> {
    // 初手かどうかをチェック（盤面にプレイヤーの手が1つ以下の場合は初手とみなす）
    let total_pieces = (board.my_board | board.opponent_board).count_ones();
    let is_early_game = total_pieces <= 4; // 初期の4石のみの場合は初手
    
    if is_early_game {
        // 初手の場合はバックグラウンド処理をスキップ
        //println!("Early game detected, skipping background thinking");
        match receive_command(reader, logger)? {
            RecvCommand::Move(m) => {
                board.do_move_interface(m, false);
                logger.log(board);
                Ok(State::MyTurn(None))
            }
            RecvCommand::End {
                result,
                your_stone_count,
                opponent_stone_count,
                reason,
            } => Ok(State::EndGame {
                result,
                your_stone_count,
                opponent_stone_count,
                reason,
            }),
            r => Err(Error::Recv(r)),
        }
    } else {
        // 通常の場合はバックグラウンド思考を実行
        let board_clone = board.clone();
        let stop_thinking = Arc::new(AtomicBool::new(false));
        let best_move = Arc::new(Mutex::new(None::<usize>));
        let node_count = Arc::new(Mutex::new(0u64));
        
        let stop_thinking_clone = stop_thinking.clone();
        let best_move_clone = best_move.clone();
        let node_count_clone = node_count.clone();
        
        // バックグラウンドで思考を開始
        let thinking_thread = thread::spawn(move || {
        // println!("Started background thinking...");
        
        // 相手の手を適用した後の盤面で思考
        let mut thinking_board = board_clone;
        
        // 可能な相手の手を列挙して、それぞれに対する応手を考える
        let opponent_moves = thinking_board.get_valid_moves();
        if opponent_moves == 0 {
            // パスの場合
            thinking_board.change_turn();
            
            // バックグラウンド思考実行（中断可能な版を使用）
            let my_move = thinking_board.decide_move_background(&stop_thinking_clone);
            
            {
                let mut result = best_move_clone.lock().unwrap();
                *result = Some(my_move);
            }
            
            // println!("Background thinking completed for pass case, move: {}", my_move);
            return;
        }
        
        // 最初の有効手に対する応手を計算
        let first_move = opponent_moves.trailing_zeros() as u8;
        thinking_board.do_move(first_move);
        thinking_board.change_turn();
        
        // バックグラウンド思考実行（中断可能な版を使用）
        // println!("Background thinking for opponent move: {}", first_move);
        let my_move = thinking_board.decide_move_background(&stop_thinking_clone);
        
        // 結果を更新
        {
            let mut result = best_move_clone.lock().unwrap();
            *result = Some(my_move);
        }
        
        if stop_thinking_clone.load(Ordering::Relaxed) {
            // println!("Background thinking interrupted, best move so far: {}", my_move);
        } else {
            // println!("Background thinking completed, best move: {}", my_move);
        }
        });
        
        // メインスレッドでI/O待ち
        let command_result = receive_command(reader, logger);
        
        // 思考を停止
        stop_thinking.store(true, Ordering::Relaxed);
        
        // スレッドの終了を待つ（タイムアウト付き）
        let _ = thinking_thread.join();
        
        // バックグラウンド思考の結果を取得
        let background_move = {
            let result = best_move.lock().unwrap();
            *result
        };
        
        let final_node_count = {
            let count = node_count.lock().unwrap();
            *count
        };
        
        if let Some(bg_move) = background_move {
            // println!("Background thinking found move: {} (evaluated {} nodes)", bg_move, final_node_count);
        }
        
        // 受信したコマンドを処理
        match command_result? {
            RecvCommand::Move(m) => {
                board.do_move_interface(m, false);
                logger.log(board);
                Ok(State::MyTurn(None))
            }
            RecvCommand::End {
                result,
                your_stone_count,
                opponent_stone_count,
                reason,
            } => Ok(State::EndGame {
                result,
                your_stone_count,
                opponent_stone_count,
                reason,
            }),
            r => Err(Error::Recv(r)),
        }
    }
}

fn proc_end(
    result: Wl,
    your_stone_count: u32,
    opponent_stone_count: u32,
    reason: &str,
    player_name: &str,
    opponent_name: &str,
    player_color: Color,
) {
    let result_str = match result {
        Wl::Win => "You win!",
        Wl::Lose => "You lose!",
        Wl::Tie => "Draw",
    };
    println!("{result_str} ({your_stone_count} vs. {opponent_stone_count}) -- {reason}.",);
    println!(
        "Your name: {player_name} ({player_color})  Opponent name: {opponent_name} ({opponent_color}).",
        opponent_color = player_color.opposite()
    );
}

fn client(options: &MyOptions) -> Result<()> {
    println!("{:?}", options.socket_addr);
    let stream = TcpStream::connect(options.socket_addr)?;
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    let mut logger = Logger::new(options);
    send_command(
        &mut writer,
        &mut logger,
        &(SendCommand::Open {
            player_name: &options.player,
        }),
    )?;

    let mut state = State::WaitStart;
    let mut board = None;
    let mut assigned_time_ms = 0i32;
    let mut opponent_name = String::new();
    let mut player_color = Color::Black;
    loop {
        match state {
            State::WaitStart => {
                state = wait_start(&mut reader, &mut logger)?;
            }
            State::MyTurn(Some(init_game)) => {
                assigned_time_ms = init_game.assigned_time_ms;
                opponent_name = init_game.opponent_name;
                board = Some(Board::new(false));
                state = State::MyTurn(None);
                player_color = Color::Black;
            }
            State::OpTurn(Some(init_game)) => {
                assigned_time_ms = init_game.assigned_time_ms;
                opponent_name = init_game.opponent_name;
                board = Some(Board::new(true));
                state = State::OpTurn(None);
                player_color = Color::White;
            }
            State::MyTurn(None) => {
                state = my_move(
                    &mut reader,
                    &mut writer,
                    &mut logger,
                    board.as_mut().expect("board must be initialized"),
                    player_color,
                    &mut assigned_time_ms,
                )?;
            }
            State::OpTurn(None) => {
                state = op_move(
                    &mut reader,
                    &mut logger,
                    board.as_mut().expect("board must be initialized"),
                    player_color,
                )?;
            }
            State::EndGame {
                result,
                your_stone_count,
                opponent_stone_count,
                reason,
            } => {
                proc_end(
                    result,
                    your_stone_count,
                    opponent_stone_count,
                    &reason,
                    &options.player,
                    &opponent_name,
                    player_color,
                );
                state = State::WaitStart;
            }
            State::Exit { stat } => {
                print_scores(&mut logger, stat.into_iter());
                break;
            }
        }
    }
    Ok(())
}

fn clear_log() {
    if let Ok(_) = std::fs::write("log.txt", "") {}
}

fn main() {
    let options = parse_args();
    clear_log();
    database::initialize_tables();
    database::init_book();
    client(&options).unwrap_or_else(|e| {
        eprintln!("{e}");
    });
}