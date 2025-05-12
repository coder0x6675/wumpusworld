
use wumpusworld::wumpus;
use wumpusworld::models;
use wumpusworld::algorithms;

use serde::Deserialize;


fn main() {

	// Select model to use.
	let model = std::env::args()
		.nth(1)
		.expect("No model specified")
		.to_lowercase()
		;
	let mut model: Box<dyn models::Model> = match model.as_str() {
		"random" => Box::new(models::ModelRandom{}),
		"manual" => Box::new(models::ModelManual{}),
		"bayes" => Box::new(models::ModelBayes{.. Default::default()}),
		_ => panic!("Unknown model type"),
	};

	// Connect to the game server.
	let stream = std::net::TcpStream::connect(concat!("127.0.0.1:", 6666)).expect("Unable to connect to the server");
	let mut de = serde_json::Deserializer::from_reader(&stream);

	let mut game: wumpus::Game;
	loop {

		// Receive the state of the game.
		game = wumpus::Game::deserialize(&mut de).expect("Error while deserializing game from server");

		// Print the game and events.
		println!("{}", algorithms::visualize_map(&game.map, &game.location, &game.direction, &false));
		if game.events.bonked   { println!("> You hit your head against the wall. Ouch!"); }
		if game.events.scream   { println!("> A terrible scream echoes throughout the cave..."); }
		if game.events.treasure { println!("> You found a treasure! Congratulations!"); }
		if game.events.pit      { println!("> Oh no, you fell into a pit :("); }

		// Print ending game statement.
		if game.game_over {
			if game.events.wumpus {
				println!("> You walked into a wumpus den. GG");
			}
			else {
				println!("> All treasures have been found. GG");
			}
			break;
		}

		// Show the status bar.
		println!("Position: {} facing {}, arrows: {}, score: {}",
			game.location,
			game.direction,
			game.arrows,
			game.score,
		);

		// Let the model choose an action.
		let action = model.run(&game);

		// Send that action to the server.
		serde_json::to_writer(&stream, &action).expect("Error while sending action to server");
		std::thread::sleep(std::time::Duration::from_secs(1));
	}

	// Print the final score.
	println!();
	println!("GAME OVER");
	println!("Final score: {}", game.score);
}

