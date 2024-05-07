

use core::convert::TryFrom;
use core::convert::TryInto;
use tokio::sync::{Mutex, mpsc};
use std::sync::Arc; 
use std::error::Error as StdError;
use std::fmt;


use Color::*;
#[derive(Copy, Clone, PartialEq)]
pub enum Color {
    White,
    Black,
}   

use PieceType::*;
#[derive(Copy, Clone)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

use Piece::{Black, White};
#[derive(Copy, Clone)]
pub enum Piece {
    White(PieceType),
    Black(PieceType),
}

impl Piece {
    fn get_color(&self) -> Color {
        match self {
            White(_) => Color::White,
            Black(_) => Color::Black,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Position {
    row: usize,    // 0-7 for rows 1-8 on the chessboard
    column: usize, // 0-7 for columns a-h on the chessboard
}

use Turn::*;
#[derive(Copy, Clone)]
pub enum Turn {
    WhitePlays,
    BlackPlays
}

#[derive(Debug)]
pub enum Error {
    OpponentGone(String),
    BadMove(String),
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::OpponentGone(msg) => write!(f, "Opponent gone: {}", msg),
            Error::BadMove(msg) => write!(f, "Bad move: {}", msg),
            Error::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl StdError for Error {}

struct ChessBoard {
    state: [[Option<Piece>; 8]; 8]
}

impl ChessBoard {
    fn new() -> Self {
        // Initialize an empty board
        let mut state: [[Option<Piece>; 8]; 8] = Default::default();

        // Place black pieces
        state[0] = [
            Some(White(Rook)),
            Some(White(Knight)),
            Some(White(Bishop)),
            Some(White(Queen)),
            Some(White(King)),
            Some(White(Bishop)),
            Some(White(Knight)),
            Some(White(Rook)),
        ];
        for i in 0..8 {
            state[1][i] = Some(White(Pawn));
            state[6][i] = Some(Black(Pawn));
        }

        // Place white pieces
        state[7] = [
            Some(Black(Rook)),
            Some(Black(Knight)),
            Some(Black(Bishop)),
            Some(Black(Queen)),
            Some(Black(King)),
            Some(Black(Bishop)),
            Some(Black(Knight)),
            Some(Black(Rook)),
        ];

        ChessBoard { state }
    }

    fn get_field(&self, position: Position) -> Option<Piece> {
        if position.is_valid() {
            self.state[position.row][position.column]
        }
        else {
            None
        }
    }

    fn set_field(&mut self, position: Position, piece: Option<Piece>) {
        self.state[position.row][position.column] = piece;
    }
}

impl Position {
    pub fn is_valid(&self) -> bool {
        self.row < 8 && self.column < 8
    }
}

impl TryFrom<&str> for Position {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 2 {
            return Err(Error::Other("Invalid position".to_string()));
        }
        let col = value.chars().nth(0).unwrap();
        let row = value.chars().nth(1).unwrap();

        if col >= 'a' && col <= 'h' && row >= '1' && row <= '8' {
            let column = col as usize - 'a' as usize; // Convert letter to 0-7
            let row = row.to_digit(10).unwrap() as usize - 1; // Convert number to 0-7
            Ok(Position { row, column })
        } else {
            Err(Error::Other("Invalid position".to_string()))
        }
    }
}

impl Turn {
    fn get_color(&self) -> Color {
        match self {
            WhitePlays => Color::White,
            BlackPlays => Color::Black,
        }
    }

    fn change(&mut self) {
        *self = match self {
            WhitePlays => BlackPlays,
            BlackPlays => WhitePlays,
        }
    }
}

pub struct Game {
    white_move_sender: Option<mpsc::Sender<String>>,
    black_move_sender: Option<mpsc::Sender<String>>,
    white_move_receiver: mpsc::Receiver<String>,
    black_move_receiver: mpsc::Receiver<String>,
    white_update_sender: mpsc::Sender<String>,
    black_update_sender: mpsc::Sender<String>,
    white_update_receiver: Option<mpsc::Receiver<String>>,
    black_update_receiver: Option<mpsc::Receiver<String>>,
    game_state: Arc<Mutex<GameState>>,
    player_created: u8, 
}

struct GameState {
    pub board: ChessBoard, 
    current_turn: Turn,
}

impl GameState {
    pub fn get_field(&self, position: Position) -> Option<Piece> {  
        self.board.get_field(position)
    }
    fn set_field(&mut self,  position: Position, piece: Option<Piece>) {
        self.board.set_field(position, piece);
    }

    fn move_piece(&mut self, position_from: Position, position_to: Position) {
        self.set_field(position_to, self.get_field(position_from));
        self.set_field(position_from, None);
        self.current_turn.change();
    }
    pub async fn make_move (&mut self, position_from: Position, position_to: Position) -> Result<Option<Piece>, Error> {
        if !position_from.is_valid() || !position_to.is_valid() {
            return Err(Error::BadMove("Invalid position".to_string()));
        }
        let field_from = self.get_field(position_from);
        let field_to = self.get_field(position_to);
        let piece_from = match field_from {
            Some(piece) => piece,
            None => return Err(Error::BadMove("No piece at position".to_string())),
        };
        
        let piece_from_color = piece_from.get_color();
        if piece_from_color != self.current_turn.get_color() {
            return Err(Error::BadMove("Not your turn".to_string()));
        }
        let piece_to = match field_to {
            Some(piece) => piece,
            None => {
                self.move_piece(position_from, position_to);
                return Ok(None);
            }
        };
        let piece_to_color = piece_to.get_color();
        if piece_from_color == piece_to_color {
            return Err(Error::BadMove("Cannot take your own piece".to_string()));
        }
        self.move_piece(position_from, position_to);
        Ok(Some(piece_to))
    }
    pub fn current_player(&self) -> Turn {
        self.current_turn
    }
}

pub struct Player {
    pub sender: mpsc::Sender<String>,
    pub receiver: mpsc::Receiver<String>,
    color: Color,
}

impl Player {
    pub async fn wait(&mut self) -> Result<String, Error> {
        match self.receiver.recv().await {
            Some(message) => {
                println!("{} player received: {}", match self.color { Color::White => "White", Color::Black => "Black" }, message);
                Ok(message)
            }
            None => Err(Error::OpponentGone("Opponent disconnected".to_string())),
        }
    }

    pub async fn play(&mut self, move_str: String) -> Result<(), Error> {
        println!("{} player sending: {}", match self.color { Color::White => "White", Color::Black => "Black" }, move_str);
        self.sender.send(move_str).await.map_err(|_| Error::BadMove("Failed to send move".to_string()))?;
        match self.receiver.recv().await {
            Some(response) => {
                if response == "Move accepted" {
                    Ok(())
                } else {
                    Err(Error::BadMove(response))  // Assuming response is the error message directly
                }
            },
            _ => Err(Error::Other("Failed to receive response from the game".to_string()))
        }
    }

    pub fn color(&self) -> Color {
        self.color
    }
}


impl Game {

    pub fn new() -> Self {
        let (wms, wmr) = mpsc::channel::<String>(32);  // white move sender, receiver
        let (bms, bmr) = mpsc::channel::<String>(32);  // black move sender, receiver
        let (wus, wur) = mpsc::channel::<String>(32);  // white update sender, receiver
        let (bus, bur) = mpsc::channel::<String>(32);  // black update sender, receiver
        let game_state = Arc::new(Mutex::new(GameState {
            board: ChessBoard::new(),  
            current_turn: WhitePlays,
        }));

        Game {
            white_move_sender: Some(wms),
            black_move_sender: Some(bms),
            white_move_receiver: wmr,
            black_move_receiver: bmr,
            white_update_sender: wus,
            black_update_sender: bus,
            white_update_receiver: Some(wur),
            black_update_receiver: Some(bur),
            game_state,
            player_created: 0,
        }
    }

    pub fn create_player(&mut self) -> Player {
        self.player_created += 1;
        match self.player_created {
            1 => {
                Player {
                    sender: self.white_move_sender.take().expect("White move sender already taken"),
                    receiver: self.white_update_receiver.take().expect("White update receiver already taken"),
                    color: Color::White,
                }
            },
            2 => {
                Player {
                    sender: self.black_move_sender.take().expect("Black move sender already taken"),
                    receiver: self.black_update_receiver.take().expect("Black update receiver already taken"),
                    color: Color::Black,
                }
            },
            _ => panic!("All players have already been created"),
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                Some(move_str) = self.white_move_receiver.recv() => {
                    println!("White move: {}", move_str);
                    let result = self.handle_move(move_str.clone()).await;
                    match result {
                        Ok(_) => {
                            // If the move is valid, send it to the black player
                            let _ = self.white_update_sender.send("Move accepted".to_string()).await;
                            let _ = self.black_update_sender.send(move_str).await;
                        },
                        Err(e) => {
                            // Send error back to white player
                            let _ = self.white_update_sender.send(e.to_string()).await;
                        }
                    }
                },
                Some(move_str) = self.black_move_receiver.recv() => {
                    println!("Black move: {}", move_str);
                    let result = self.handle_move(move_str.clone()).await;
                    match result {
                        Ok(_) => {
                            // If the move is valid, send it to the white player
                            let _ = self.black_update_sender.send("Move accepted".to_string()).await;
                            let _ = self.white_update_sender.send(move_str).await;
                        },
                        Err(e) => {
                            // Send error back to black player
                            let _ = self.black_update_sender.send(e.to_string()).await;
                        }
                    }
                },
            }
        }
    }
    

    async fn handle_move(&self, move_str: String) -> Result<(), Error> {
        println!("Handling move: {}", move_str);
        let parts: Vec<&str> = move_str.split('-').collect();
        if parts.len() != 2 {
            return Err(Error::Other("Invalid move format".to_string()));
        }

        let from_pos = parts[0].try_into().map_err(|_| Error::Other("Invalid start position".to_string()))?;
        let to_pos = parts[1].try_into().map_err(|_| Error::Other("Invalid end position".to_string()))?;

        let mut game_state = self.game_state.lock().await;  // Await the lock here
        game_state.make_move(from_pos, to_pos).await.map(|_| ())
    }
}

#[tokio::main]
async fn main() {
    let mut game = Game::new();
    let mut white = game.create_player();
    let mut black = game.create_player();

    let task = tokio::spawn(async move {
        game.run().await;
    });

    let my_white_move = "e2-e4".to_string();   
    match white.play(my_white_move).await {
        Ok(()) => println!("1 Move played"),
        Err(Error::BadMove(bad_move)) => {
            println!("Bad move: {}", bad_move);
        }
        _ => panic!("unexpected error"),
    };
    let black_move = match white.wait().await {
        Ok(their_move) => their_move,
        Err(Error::OpponentGone(reason)) => {
            println!("Opponent gone: {}", reason);
            return;
        }
        _ => panic!("unexpected error"),
    };
    let white_move = match black.wait().await {
        Ok(their_move) => their_move,
        Err(Error::OpponentGone(reason)) => {
            println!("Opponent gone: {}", reason);
            return;
        },
        _ => panic!("unexpected error"),
    };

    let my_black_move = "e7-e5".to_string();
    match black.play(my_black_move).await {
        Ok(()) => print!("2 Move played"),
        Err(Error::BadMove(bad_move)) => {
            println!("Bad move: {}", bad_move);
        }
        _ => panic!("unexpected error"),
    };
        


    // Implementation for playing moves, handling errors, etc.

    task.await.expect("Game task crashed");
}
