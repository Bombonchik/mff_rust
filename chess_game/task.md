Create a prototype of a chess game library that can be used with local or remote players or even game AI.

Your library will be based on the Tokio asynchronous runtime. It will not provide any asynchronous traites. Instead it will provide a game object and two player interfaces that will be used by the testing code.

## The game interface

Design the interface of your library to allow for the following usage patterns. You can use shared state via standard library or Tokio mutexes, message queues via Tokio channels. To find out what you need is your main job during this task.

The user of your library provided in `solution.rs` will be able to create a game and run it. The user of the library provides the main function and the Tokio scheduler as follows.

```
use solution::Game;
use tokio::task::spawn;

#[tokio::main]
async fn main() {
    todo!();
}
```

The main program needs to create the game and move it into a Tokio task to let it run in the background.

```
let game = Game::new();
let task = spawn(async move { game.run().await; });
task.await
    .expect("game task crashed");
```

Before the program spawns the task, it uses the game object to create the player interfaces. Make sure these are independent, so that the game can be moved to its own task. Use the tools provided by Tokio and the standard library.

```
let mut white = game.create_player();
let mut black = game.create_player();
```

Notice that all the objects are marked mutable. This is part of the API of your library and you can require a mutable self reference for the methods.

The order is simple. The first call creates the white player interface, the second one creates the black player and the library doesn't allow you to successfully run the method for the third time.

## The player interface

In our design it is the player who talks to the game object via the player interface. The player can ask about his color. The color function is immediate.

```
use solution::Color;

let mut player = game.create_player();
match player.color() {
    Color::White => todo!(),
    Color::Black => todo!(),
}
```

The white player can make a move immediately and then wait. The black player would first wait for a turn and only then make its own turn. Both playing and waiting is asynchronous. The reason in the error can be a piece of text. The testing code doesn't care. The same applies to the bad move error.

```
use solution::Error;

let their_move = match white.wait().await {
    Ok(their_move) => their_move,
    Err(Error::OpponentGone(reason)) => todo!(),
    _ => panic!("unexpected error"),
};

match white.play(my_move).await {
    Ok(()) => todo!(),
    Err(Error::BadMove(bad_move)) => todo!(),
    _ => panic!("unexpected error"),
};
```

You can see that we use a broad error domain. The running code needs to handle all possible errors. That's why the default case is covered to avoid handing errors that we don't expect in that situation. This shows you how powerful is Rust in error handling.

The library is supposed to check for good and bad moves. However, it is up to you if you want to implement a complete move correctness checker or do just enough to make the tests happy.

The move is passed and returned as an owned string in a simplified notation that includes the source coordinates, a hyphen and the destination coordinates.

```
let my_move = "e2-e4".to_string();
```

Keep things simple and this will be an easy task. Or you can experiment and make it challenging if you have enough time and motivation.

## The player implementation

Add one player implementation that plays the first moves of a game either for white or for black depending on the color reported by the game. We don't use class inheritance in Rust and we didn't introduce a polymorphic trait for this task. Therefore the player implementation will have to encapsulate the player interface and provide an asynchronous function to perform all tasks.

Your player will just play the moves of the Sicilian Defence, Najdorf Variation. It can quit after the five moves.

```
use solution::Najdorf;
let player = Najdorf::new(game.create_player());
let task = spawn(player.run());
task.await()
    .expect("player task crashed")
    .expect("an error occured when playing or waiting");
```

It will run as a separate task from the game. Search for the actor model for more information.  