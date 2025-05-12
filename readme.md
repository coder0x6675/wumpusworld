
# WumpusWorld

> A CLI-based, client-server minigame written in Rust.

## The Game

The rules of the game:

- The map is a 4x4 grid.
- Each location has one of four different classes: *empty*, *treasure*, *wumpus* or *pit*.
- The map contains 2 treasures, 1 wumpus and 3 pits, with the rest being empty.
- Each treasure is surrounded by *glitter*.
- Each wumpus is surrounded by *stench*.
- Each pit is surrounded by *breeze*.
- A location is *undiscovered* until the player has visited that location. While it's undiscovered, no information can be gained from it.
- Initially, the player starts at (0,0) facing east. This location is always empty.

Actions:

- The player can turn *left* or *right*, and *walk* forward.
- The player gets 1 arrow to optionally try to *shoot* the wumpus with. The arrow is shot onto the location in front of the player.
- The treasures are burried, and cannot be seen on the map. To collect a treasure, the player has to *dig* at the right spot.

Objective:

- The objective is to gain the highest score possible.
- The game is won once all the treasures have been found.
- The game is lost if the player encounters a wumpus.

## Running The Game

In order to run the game, perform the following steps:

1. Assert that `git` and `cargo` are installed on the system.
1. Clone the repository.
1. Start the server by entering the root directory and running: `cargo run -r --bin server`
1. In another terminal, run the client and connect to the server: `cargo run -r --bin client HOSTNAME:6666 MODEL`
	- Replace `MODEL` with one of the models below.
	- Alternatively you can run this from any other device, as long as you have a network connection to the server.

Available models:

- *manual*: The manual model allows the user to play the game manually.
- *random*: The random model makes random actions. **This model might never finish a game.**
- *bayes*: The bayes model is based on bayesian statistics and can finish games with a decent score.

While playing, the following actions are available:

- *walk*: Walk forward 1 tile.
- *left*: Turn left.
- *left*: Turn right.
- *shoot*: Shoot an arrow onto the tile ahead.
- *dig*: Dig at the current location for a treasure.

The following events affect the final score:

- Walking: -1
- Turning left or right: -1
- Shooting an arrow: -10
- Digging for a treasure: -50
- Finding a treasure: +250
- Walking into a wumpus: -200
- Falling into a pit: -100

## Architecture

Prominent features of this project include but is not limited to:

- Multithreaded server able to serve thousands of clients simltaneously
- Selectable client model via CLI arguments
- Client-server communication over a JSON API
- Randomized map generation
- Highscore progress tracking
- Automatic pathfinding using a modified version Dijkstra's algorithm
- Advanced bot using bayesian statistics to obtain the optimal action, concistently achieving a 250+ score.

## Potential Future Improvements

The following improvements might be implemented in future versions:

- Add a CLI flag to the client enabling the selection of map generation seed.
- Improve the decision-making algorithm for the *Bayes* model to compare each possible action against each other.
- Randomize and hide the map size from the client.
- Add TUI interface for client.
- Generalize the `Coordinate` into a tuple struct, and expand the game to 3+ dimensions.
- Remove dependancies on 3rd-party libraries.
- Increase the error handling of the math equations.

Implement more model types such as:

- Neural Network
- Decision Tree
- A-Star Algorithm

