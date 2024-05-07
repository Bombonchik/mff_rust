use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::VecDeque;

pub enum Key {
    Left,
    Right,
    Up,
    Down, 
    Quit  
}

pub struct Keyboard {
    game: Arc<Mutex<Game>>,
}

impl Keyboard {
    pub async fn push(&mut self, key: Key) {
        let mut game = self.game.lock().await;
        game.process_key(key).await;
    }
}

pub enum LogRecord {
    Started(usize, usize),
    Moved(usize, usize),
    Stayed,
    Finished,
}

pub struct Logger {
    queue: Arc<Mutex<VecDeque<LogRecord>>>,
}

impl Logger {
    pub fn new() -> Self {
        Logger {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn log(&self, record: LogRecord) {
        let mut queue = self.queue.lock().await;
        queue.push_back(record);
    }

    pub async fn next(&self) -> Option<LogRecord> {
        let mut queue = self.queue.lock().await;
        queue.pop_front()
    }
}

struct Coordinate {
    x: i64,
    y: i64,
}

pub struct Game {
    coordinate: Coordinate,
    board_size: (usize, usize),
    logger: Arc<Logger>,
    is_started: bool,
}

impl Game {
    pub fn new(x: usize, y: usize) -> (Arc<Mutex<Self>>, Keyboard, Arc<Logger>)  {
        let logger = Arc::new(Logger::new());

        let game = Arc::new(Mutex::new(Game {
            coordinate: Coordinate { x: 0, y: 0 },
            board_size: (x, y),
            logger: Arc::clone(&logger),
            is_started: false,
        }));

        let keyboard = Keyboard { game: Arc::clone(&game) };
        (game, keyboard, logger)
    }

    async fn start(&mut self) {
        if !self.is_started {
            self.is_started = true;
            self.logger.log(LogRecord::Started(self.coordinate.x as usize, self.coordinate.y as usize)).await;
        }
    }

    async fn process_key(&mut self, key: Key) {
        self.start().await;
        match key {
            Key::Left => {
                if self.coordinate.x > 0 {
                    self.coordinate.x -= 1;
                    self.logger.log(LogRecord::Moved(self.coordinate.x as usize, self.coordinate.y as usize)).await;
                } else {
                    self.logger.log(LogRecord::Stayed).await;
                }
            },
            Key::Right => {
                if self.coordinate.x < self.board_size.0 as i64 - 1 {
                    self.coordinate.x += 1;
                    self.logger.log(LogRecord::Moved(self.coordinate.x as usize, self.coordinate.y as usize)).await;
                } else {
                    self.logger.log(LogRecord::Stayed).await;
                }
            },
            Key::Up => {
                if self.coordinate.y > 0 {
                    self.coordinate.y -= 1;
                    self.logger.log(LogRecord::Moved(self.coordinate.x as usize, self.coordinate.y as usize)).await;
                } else {
                    self.logger.log(LogRecord::Stayed).await;
                }
            },
            Key::Down => {
                if self.coordinate.y < self.board_size.1 as i64 - 1 {
                    self.coordinate.y += 1;
                    self.logger.log(LogRecord::Moved(self.coordinate.x as usize, self.coordinate.y as usize)).await;
                } else {
                    self.logger.log(LogRecord::Stayed).await;
                }
            },
            Key::Quit => {
                self.logger.log(LogRecord::Finished).await;
            },
            
        }
    }
    
}