
use std::str::FromStr;
use std::collections::HashSet;

use rand::{
	distributions::{Distribution, Standard},
	Rng,
};

use serde::{Serialize, Deserialize};

// ---

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Direction {
	East,
	South,
	West,
	North,
}

impl Default for Direction {
	fn default() -> Self {
		Self::East
	}
}

impl std::fmt::Display for Direction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			Self::East  => "east",
			Self::South => "south",
			Self::West  => "west",
			Self::North => "north",
		})
	}
}

impl FromStr for Direction {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"east"  => Ok(Self::East),
			"south" => Ok(Self::South),
			"west"  => Ok(Self::West),
			"north" => Ok(Self::North),
			_       => Err(()),
		}
	}
}

impl Distribution<Direction> for Standard {
	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
		match rng.gen_range(0..4) {
			0 => Direction::East,
			1 => Direction::South,
			2 => Direction::West,
			_ => Direction::North,
		}
	}
}

impl Direction {

	pub fn rotate_left(&self) -> Self {
		match self {
			Self::East  => Self::North,
			Self::South => Self::East,
			Self::West  => Self::South,
			Self::North => Self::West,
		}
	}

	pub fn rotate_right(&self) -> Self {
		match self {
			Self::East  => Self::South,
			Self::South => Self::West,
			Self::West  => Self::North,
			Self::North => Self::East,
		}
	}

	pub fn rotate_back(&self) -> Self {
		match self {
			Self::East  => Self::West,
			Self::South => Self::North,
			Self::West  => Self::East,
			Self::North => Self::South,
		}
	}
}

// ---

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Coordinate {
	pub x: i32,
	pub y: i32,
}

impl std::fmt::Display for Coordinate {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let x = self.x.to_string();
		let y = self.y.to_string();
		write!(f, "({x},{y})")
	}
}

impl FromStr for Coordinate {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {

		let (xs, ys) = s
			.strip_prefix('(')
			.and_then(|s| s.strip_suffix(')'))
			.and_then(|s| s.split_once(','))
			.ok_or(())?;

		let x: i32 = xs.parse().map_err(|_| ())?;
		let y: i32 = ys.parse().map_err(|_| ())?;
		Ok(Self{x, y})
	}
}

impl Distribution<Coordinate> for Standard {
	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Coordinate {
		Coordinate {
			x: rng.gen_range(0..Game::SIZE_X),
			y: rng.gen_range(0..Game::SIZE_Y),
		}
	}
}

impl Coordinate {

	pub const NOWHERE: Self = Self{x: -1, y: -1};
	pub const UNKNOWN: Self = Self{x: -2, y: -2};

	pub fn get_front(&self, direction: &Direction) -> Self {
		match direction {
			Direction::East  => Self{ x: self.x + 1, ..*self },
			Direction::South => Self{ y: self.y - 1, ..*self },
			Direction::West  => Self{ x: self.x - 1, ..*self },
			Direction::North => Self{ y: self.y + 1, ..*self },
		}
	}

	pub fn get_neighbours(&self) -> HashSet<Self> {
		HashSet::from([
			Self{ x: self.x + 1, ..*self },
			Self{ y: self.y - 1, ..*self },
			Self{ x: self.x - 1, ..*self },
			Self{ y: self.y + 1, ..*self },
		])
	}

	pub fn get_cluster(&self) -> HashSet<Self> {
		HashSet::from([
			*self,
			Self{ x: self.x + 1, ..*self },
			Self{ y: self.y - 1, ..*self },
			Self{ x: self.x - 1, ..*self },
			Self{ y: self.y + 1, ..*self },
		])
	}

	pub fn get_relative_direction(&self, location: &Coordinate) -> Option<Direction> {
		if      *location == self.get_front(&Direction::East)  { return Some(Direction::East);  }
		else if *location == self.get_front(&Direction::North) { return Some(Direction::North); }
		else if *location == self.get_front(&Direction::West)  { return Some(Direction::West);  }
		else if *location == self.get_front(&Direction::South) { return Some(Direction::South); }
		else { return None; }
	}

}

// ---

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Class {
	Empty,
	Treasure,
	Wumpus,
	Pit,
}

impl Class {

	pub const VALUES: [Self; 4] = [Self::Empty, Self::Treasure, Self::Wumpus, Self::Pit];

}

// ---

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ClassField<T> {
	pub empty:    T,
	pub treasure: T,
	pub wumpus:   T,
	pub pit:      T,
}

impl<T: ToString> std::fmt::Display for ClassField<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let t = self.treasure.to_string();
		let w = self.wumpus.to_string();
		let p = self.pit.to_string();
		write!(f, "({t},{w},{p})")
	}
}

// ---

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Action {
	Walk,
	Left,
	Right,
	Dig,
	Shoot,
}

impl std::fmt::Display for Action {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			Self::Walk  => "walk",
			Self::Left  => "left",
			Self::Right => "right",
			Self::Dig   => "dig",
			Self::Shoot => "shoot",
		})
	}
}

impl FromStr for Action {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"walk"  => Ok(Self::Walk),
			"left"  => Ok(Self::Left),
			"right" => Ok(Self::Right),
			"dig"   => Ok(Self::Dig),
			"shoot" => Ok(Self::Shoot),
			_       => Err(()),
		}
	}
}

impl Distribution<Action> for Standard {
	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Action {
		match rng.gen_range(0..5) {
			0 => Action::Walk,
			1 => Action::Left,
			2 => Action::Right,
			3 => Action::Dig,
			_ => Action::Shoot,
		}
	}
}

// ---

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Events {
	pub treasure : bool, // The player dug up a treasure.
	pub wumpus   : bool, // The player is on a wumpus.
	pub pit      : bool, // The player is on a pit.
	pub glitter  : bool, // The player is 1 block from the treasure.
	pub stench   : bool, // The player is 1 block from the wumpus.
	pub breeze   : bool, // The player is 1 block from a pit.
	pub bonked   : bool, // The player walked into a wall.
	pub scream   : bool, // The player killed the wumpus.
	pub gameover : bool, // The player found the treasure.
}

impl std::fmt::Display for Events {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut s = Vec::default();
		if self.treasure { s.push("treasure") }
		if self.wumpus   { s.push("wumpus") }
		if self.pit      { s.push("pit") }
		if self.glitter  { s.push("glitter") }
		if self.stench   { s.push("stench") }
		if self.breeze   { s.push("breeze") }
		if self.bonked   { s.push("bonked") }
		if self.scream   { s.push("scream") }
		if self.gameover { s.push("gameover") }
		write!(f, "{}", s.join(","))
	}
}

impl FromStr for Events {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let words: Vec<&str> = s.split(',').collect();
		Ok(Self {
			treasure : words.contains(&"treasure"),
			wumpus   : words.contains(&"wumpus"),
			pit      : words.contains(&"pit"),
			glitter  : words.contains(&"glitter"),
			stench   : words.contains(&"stench"),
			breeze   : words.contains(&"breeze"),
			bonked   : words.contains(&"bonked"),
			scream   : words.contains(&"scream"),
			gameover : words.contains(&"gameover"),
		})
	}
}

// ---

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Map {
	pub size       : Coordinate,
	pub treasures  : HashSet<Coordinate>,
	pub wumpuses   : HashSet<Coordinate>,
	pub pits       : HashSet<Coordinate>,
	pub glitters   : HashSet<Coordinate>,
	pub stenches   : HashSet<Coordinate>,
	pub breezes    : HashSet<Coordinate>,
	pub discovered : HashSet<Coordinate>,
}

impl Default for Map {
	fn default() -> Self {
		Self {
			size       : Coordinate{x: 3, y: 3},
			treasures  : Default::default(),
			wumpuses   : Default::default(),
			pits       : Default::default(),
			glitters   : Default::default(),
			stenches   : Default::default(),
			breezes    : Default::default(),
			discovered : Default::default(),
		}
	}
}

impl Map {

	pub fn encompass(&self, location: &Coordinate) -> bool {
		(location.x >= 0 && location.x <= self.size.x)
		&&
		(location.y >= 0 && location.y <= self.size.y)
	}

	pub fn add_treasure(&mut self, location: Coordinate) {
		self.treasures.insert(location);
		self.glitters.extend(location.get_neighbours());
	}

	pub fn add_wumpus(&mut self, location: Coordinate) {
		self.wumpuses.insert(location);
		self.stenches.extend(location.get_neighbours());
	}

	pub fn add_pit(&mut self, location: Coordinate) {
		self.pits.insert(location);
		self.breezes.extend(location.get_neighbours());
	}

	pub fn remove_treasure(&mut self, location: Coordinate) {
		self.treasures.remove(&location);
		let mut glitters_to_remove = location.get_neighbours();
		glitters_to_remove.retain(|neighbour|
			!neighbour.get_neighbours().iter().any(|neighbours_neighbour|
				self.treasures.contains(neighbours_neighbour)
			)
		);
		self.glitters = self.glitters.difference(&glitters_to_remove).cloned().collect();
	}

	pub fn remove_wumpus(&mut self, location: Coordinate) {
		self.wumpuses.remove(&location);
		let mut stenches_to_remove = location.get_neighbours();
		stenches_to_remove.retain(|neighbour|
			!neighbour.get_neighbours().iter().any(|neighbours_neighbour|
				self.wumpuses.contains(neighbours_neighbour)
			)
		);
		self.stenches = self.stenches.difference(&stenches_to_remove).cloned().collect();
	}

	pub fn remove_pit(&mut self, location: Coordinate) {
		self.pits.remove(&location);
		let mut breezes_to_remove = location.get_neighbours();
		breezes_to_remove.retain(|neighbour|
			!neighbour.get_neighbours().iter().any(|neighbours_neighbour|
				self.pits.contains(neighbours_neighbour)
			)
		);
		self.breezes = self.breezes.difference(&breezes_to_remove).cloned().collect();
	}

	pub fn apply_classes(&mut self, locations: &[Coordinate], classes: &[Class]) {
		debug_assert_eq!(locations.len(), classes.len());
		for (location, class) in std::iter::zip(locations, classes) {
			self.treasures.remove(&location);
			self.wumpuses.remove(&location);
			self.pits.remove(&location);
			match class {
				Class::Treasure => { self.treasures.insert(*location); },
				Class::Wumpus   => { self.wumpuses.insert(*location); },
				Class::Pit      => { self.pits.insert(*location); },
				_               => (),
			}
		}
	}

	pub fn get_frontier(&self) -> HashSet<Coordinate> {
		self.discovered
			.iter()
			.flat_map(|&location| location.get_neighbours())
			.filter(|&neighbour| self.encompass(&neighbour) && ! self.discovered.contains(&neighbour))
			.collect()
	}

}

// ---

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
	pub map        : Map,
	pub location   : Coordinate,
	pub direction  : Direction,
	pub events     : Events,
	pub game_over  : bool,
	pub score      : i32,
	pub arrows     : i32,
}

impl Default for Game {
	fn default() -> Self {
		Self {
			map       : Default::default(),
			location  : Default::default(),
			direction : Default::default(),
			events    : Default::default(),
			game_over : Default::default(),
			score     : Default::default(),
			arrows    : 1,
		}
	}
}

impl Game {

	pub const SIZE_X: i32 = 4;
	pub const SIZE_Y: i32 = 4;

	pub const SPAWN_LOCATION  : Coordinate = Coordinate{x: 0, y: 0};
	pub const SPAWN_DIRECTION : Direction  = Direction::East;
	pub const SPAWN_ARROWS    : i32        = 1;

	pub const COUNT_TREASURES : i32 = 2;
	pub const COUNT_WUMPUSES  : i32 = 1;
	pub const COUNT_PITS      : i32 = 3;

	pub const SCORE_ACTION   : i32 = -1;   // When a movement is poerformed.
	pub const SCORE_SHOT     : i32 = -10;  // When shooting an arrow.
	pub const SCORE_DUG      : i32 = -50;  // When digging for a treasure.
	pub const SCORE_TREASURE : i32 =  250; // When finding a treasure.
	pub const SCORE_WUMPUS   : i32 = -200; // When players walks into a wumpus.
	pub const SCORE_PIT      : i32 = -100; // When falling into a pit.


	pub fn new_random() -> Self {

		// Create a new map
		let mut map: Map = Default::default();
		map.discovered.insert(Self::SPAWN_LOCATION);

		// Generate special locations
		let special_location_count = Self::COUNT_TREASURES + Self::COUNT_WUMPUSES + Self::COUNT_PITS;
		let mut special_locations: Vec<Coordinate> = vec![Self::SPAWN_LOCATION];

		while special_locations.len() <= special_location_count as usize {
			let random_location: Coordinate = rand::random();
			if ! special_locations.contains(&random_location) {
				special_locations.push(random_location);
			}
		}

		// Insert special locations into map
		let mut iter = special_locations.iter().skip(1);
		for location in iter.by_ref().take(Self::COUNT_TREASURES as usize) { map.add_treasure(*location); }
		for location in iter.by_ref().take(Self::COUNT_WUMPUSES as usize)  { map.add_wumpus(*location); }
		for location in iter.by_ref().take(Self::COUNT_PITS as usize)      { map.add_pit(*location); }

		// Build the game struct
		let mut game = Self {
			map       : map,
			direction : Self::SPAWN_DIRECTION,
			arrows    : Self::SPAWN_ARROWS,
			.. Default::default()
		};

		// Initialize the game
		game.place_player(&Self::SPAWN_LOCATION);
		game.update_senses();
		return game;
	}


	pub fn update_senses(&mut self) {
		self.events.glitter  = self.map.glitters.contains(&self.location);
		self.events.stench   = self.map.stenches.contains(&self.location);
		self.events.breeze   = self.map.breezes.contains(&self.location);
	}


	pub fn place_player(&mut self, new_location: &Coordinate) {

		self.map.discovered.insert(*new_location);

		if self.map.wumpuses.contains(&new_location) {
			self.game_over = true;
			self.events.gameover = true;
			self.events.wumpus = true;
			self.score += Self::SCORE_WUMPUS;
		}

		if self.map.pits.contains(&new_location) {
			self.events.pit = true;
			self.score += Self::SCORE_PIT;
		}

		self.location = *new_location;
	}


	pub fn do_action(&mut self, action: Action) {

		if self.game_over {
			self.events.gameover = true;
			return;
		}

		self.events = Default::default();
		self.score += Self::SCORE_ACTION;
		
		match action {

			Action::Walk => {
				let new_location = self.location.get_front(&self.direction);
				if self.map.encompass(&new_location) {
					self.place_player(&new_location);
				}
				else {
					self.events.bonked = true;
				}
			},

			Action::Left => {
				self.direction = self.direction.rotate_left();
			},

			Action::Right => {
				self.direction = self.direction.rotate_right();
			},

			Action::Dig => {
				self.score += Self::SCORE_DUG;
				if self.map.treasures.contains(&self.location) {
					self.map.remove_treasure(self.location);
					self.score += Self::SCORE_TREASURE;
					self.events.treasure = true;
				}
				if self.map.treasures.is_empty() {
					self.events.gameover = true;
					self.game_over = true;
				}
			},

			Action::Shoot => {
				if self.arrows > 0 {
					self.arrows -= 1;
					self.score += Self::SCORE_SHOT;
					let front_location = self.location.get_front(&self.direction);
					if self.map.wumpuses.contains(&front_location) {
						self.map.remove_wumpus(front_location);
						self.events.scream = true;
					}
				}
			},

		}

		self.update_senses();
	}

}

