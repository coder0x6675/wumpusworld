
use std::io::{BufRead, Write};
use std::str::FromStr;
use std::collections::{HashSet, HashMap};

use crate::algorithms;
use crate::wumpus::{
	Coordinate,
	Class,
	ClassField,
	Action,
	Game,
};


pub trait Model {
	fn run(&mut self, game: &Game) -> Action;
}

// ---

pub struct ModelRandom{
}

impl Model for ModelRandom {
	fn run(&mut self, _game: &Game) -> Action {
		rand::random()
	}
}

// ---

pub struct ModelManual {
}

impl Model for ModelManual {
	fn run(&mut self, _game: &Game) -> Action {
		loop {

			const PROMPT: &str = "What do you want to do? ";

			print!("{}", PROMPT);
			std::io::stdout()
				.flush()
				.expect("Error while flushing stdout")
				;

			let mut line = String::new();
			std::io::stdin()
				.lock()
				.read_line(&mut line)
				.expect("Error while reading from stdin")
				;
			line = line
				.trim()
				.to_lowercase()
				;

			if let Ok(action) = Action::from_str(line.as_str()) { break action }
			println!("Unrecognized action. Try again.");
		}
	}
}

// ---

#[derive(Default)]
pub struct ModelBayes {
	pub treasures_found: i32,
	pub wumpuses_killed: i32,
	pub blacklist: HashMap<Coordinate, Class>,
	pub action_queue: std::collections::VecDeque<Action>,
}

impl ModelBayes {

	fn naive_bayes_classifier(&self, c: f64, n: f64, i: f64, a: f64) -> f64{

		/* Naive Bayes equation: https://en.wikipedia.org/wiki/Naive_Bayes_classifier
		1. Calculate P(A)
		2. Calculate P(!A)
		3. Calculate P(B | A)
		4. Calculate P(B | !A)
		5. Calculate P(A | B) = ( P(B | A) * P(A) ) / ( P(B | A) * P(A) + P(B | !A) * P(!A) )
		*/

		let prior      : f64 = c / n;
		let likelihood : f64 = i / a;
		let evidence   : f64 = prior * likelihood + ((1.0 - prior) * (1.0 - likelihood));
		let posterior  : f64 = (prior * likelihood) / evidence;
		return posterior;
	}

}

impl Model for ModelBayes {
	fn run(&mut self, game: &Game) -> Action {

		// Remember important events
		if game.events.treasure { self.treasures_found += 1; }
		if game.events.scream   { self.wumpuses_killed += 1; }

		// Finish performing the chosen abstract action
		if ! self.action_queue.is_empty() {
			return self.action_queue.pop_front().unwrap();
		}

		// Calculate paths and cost to locations
		let (path_map, path_costs) = algorithms::pathfind(
			&game.location,
			&game.direction,
			&game.map
		);

		// Identify the locations which class is uncertain.
		let frontier: Vec<Coordinate> = game.map.get_frontier().into_iter().collect();
		let possible_treasures: Vec<Coordinate> = game.map.glitters
			.iter()
			.flat_map(|&location| location.get_neighbours())
			.filter(|&location| true
				&& ! game.map.wumpuses.contains(&location)
				&& ! game.map.pits.contains(&location)
				&&   game.map.discovered.contains(&location)
			)
			.collect::<HashSet<Coordinate>>()
			.into_iter()
			.collect()
			;

		let (total_map_possibilities, class_counts) = algorithms::calculate_map_possibilities(
			&frontier,
			&possible_treasures,
			&game.map,
			&self.blacklist,
		);

		// Calculate general class statistics.
		let map_size          : i32 = Game::SIZE_X * Game::SIZE_Y;
		let undiscovered_left : i32 = map_size - game.map.discovered.len() as i32;
		let treasures_left    : i32 = Game::COUNT_TREASURES - game.map.treasures.len() as i32 - self.treasures_found;
		let wumpuses_left     : i32 = Game::COUNT_WUMPUSES - game.map.wumpuses.len() as i32 - self.wumpuses_killed;
		let pits_left         : i32 = Game::COUNT_PITS - game.map.pits.len() as i32;
		let empties_left      : i32 = undiscovered_left - treasures_left - wumpuses_left - pits_left;

		// Calculate class probabilities.
		let mut classes: HashMap<Coordinate, ClassField<f64>> = Default::default();
		for (location, class_count) in class_counts {
			classes.insert(location, ClassField{

				empty: self.naive_bayes_classifier(
					empties_left.into(),
					undiscovered_left.into(),
					class_count.empty.into(),
					total_map_possibilities.into(),
				),

				treasure: self.naive_bayes_classifier(
					treasures_left.into(),
					undiscovered_left.into(),
					class_count.treasure.into(),
					total_map_possibilities.into(),
				),

				wumpus: self.naive_bayes_classifier(
					wumpuses_left.into(),
					undiscovered_left.into(),
					class_count.wumpus.into(),
					total_map_possibilities.into(),
				),

				pit: self.naive_bayes_classifier(
					pits_left.into(),
					undiscovered_left.into(),
					class_count.pit.into(),
					total_map_possibilities.into(),
				),
			});
		}

		// If treasure is known, dig it up
		if let Some(treasure) = classes
			.iter()
			.filter(|&(_, c)| c.treasure >= 0.25)
			.min_by_key(|&(l, _)| path_costs[l])
		{
			let treasure = treasure.0;
			self.blacklist.insert(*treasure, Class::Treasure);
			let actions = algorithms::path_to_actions(&treasure, &game.direction, &path_map).unwrap();
			self.action_queue.extend(actions);
			self.action_queue.push_back(Action::Dig);
			return self.action_queue.pop_front().unwrap();
		}

		// If wumpus is known, shoot it
		if let Some(wumpus) = classes
			.iter()
			.filter(|&(_, c)| c.wumpus == 1.0)
			.min_by_key(|&(l, _)| path_costs[l])
		{
			let wumpus = wumpus.0;
			self.blacklist.insert(*wumpus, Class::Wumpus);
			let actions = algorithms::path_to_actions(&wumpus, &game.direction, &path_map).unwrap();
			self.action_queue.extend(actions);
			self.action_queue.pop_back();
			self.action_queue.push_back(Action::Shoot);
			return self.action_queue.pop_front().unwrap();
		}

		// Discover the most rewarding location
		let get_score = |location: &Coordinate| -> f64 {
			let class = classes[&location];
			let safety = 1.0 - if class.wumpus != 0.0 {0.9999} else {class.pit};
			let cost = 1.0 / (path_costs[&location] + 20) as f64;
			let score = cost * safety;
			return score;
		};

		if let Some(location) = classes
			.iter()
			.filter(|&(l, _)| ! game.map.discovered.contains(&l) )
			.map(|(l, _)| (l, get_score(&l)) )
			.max_by(|(_, s1), (_, s2)| s1.partial_cmp(&s2).unwrap())
		{
			let location = location.0;
			let actions = algorithms::path_to_actions(&location, &game.direction, &path_map).unwrap();
			self.action_queue.extend(actions);
			return self.action_queue.pop_front().unwrap();
		}

		unreachable!();
	}
}

