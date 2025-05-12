
use std::{
	net::{TcpListener, TcpStream},
	sync::{Arc, Mutex},
};

use wumpusworld::wumpus;
use wumpusworld::algorithms;

use serde::Deserialize;


fn handle_client(stream: TcpStream, high_score: Arc<Mutex<i32>>) {

	// Show the connected client.
	let mut de = serde_json::Deserializer::from_reader(&stream);
	let client_address = stream.peer_addr().expect("Could not determine client address");
	println!("Client {client_address} connected");

	// Initialize the game and send the state.
	let mut game = wumpus::Game::new_random();
	println!("{}", algorithms::visualize_map(&game.map, &game.location, &game.direction, &true));

	loop {

		// Hide undiscovered information from the client.
		let mut hidden_game = game.clone();
		algorithms::hide_map(&mut hidden_game.map);

		// Send the game state to the client.
		if let Err(_) = serde_json::to_writer(&stream, &hidden_game) {
			break;
		}

		// Receive and perform action from the client.
		if let Ok(action) = wumpus::Action::deserialize(&mut de) {
			println!("- {client_address} performs: {action}");
			game.do_action(action);
		}
	}

	// Update the global high score if applicable.
	let player_score = game.score;
	let mut high_score = high_score.lock().unwrap();
	if player_score > *high_score {
		*high_score = player_score;
		println!("Client {client_address} disconnected with a new high score of {player_score}!");
	}
	else {
		println!("Client {client_address} disconnected with a score of {player_score}");
	}
}


fn main() {

	let address = concat!("127.0.0.1:", 6666);
	let listener = TcpListener::bind(address).expect("Failed to bind to port");
	println!("Server listening on {address}...");

	let high_score = Arc::new(Mutex::new(std::i32::MIN));

	for stream in listener.incoming() {
		let stream = stream.unwrap();
		let high_score = Arc::clone(&high_score);
		std::thread::spawn(move || handle_client(stream, high_score));
	}
}

