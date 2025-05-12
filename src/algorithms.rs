
use std::collections::{VecDeque, HashMap};

use crate::wumpus::{
	Coordinate,
	Direction,
	Action,
	Class,
	ClassField,
	Map,
	Game,
};

use itertools::Itertools;


pub fn hide_map(map: &mut Map) {
	map.treasures.clear();
	map.wumpuses = map.discovered.intersection(&map.wumpuses).cloned().collect();
	map.pits     = map.discovered.intersection(&map.pits).cloned().collect();
	map.glitters = map.discovered.intersection(&map.glitters).cloned().collect();
	map.stenches = map.discovered.intersection(&map.stenches).cloned().collect();
	map.breezes  = map.discovered.intersection(&map.breezes).cloned().collect();
}


pub fn is_map_valid(map: &Map, blacklist: &HashMap<Coordinate, Class>) -> bool {

	// All used tiles must be within the map.
	if std::iter::empty()
		.chain(map.treasures.iter())
		.chain(map.wumpuses.iter())
		.chain(map.pits.iter())
		.any(|&x| ! map.encompass(&x))
	{ return false; }

	// A blacklisted location cannot have the blacklisted class.
	for (location, class) in blacklist {
		match class {
			Class::Treasure => if map.treasures.contains(location) { return false; },
			Class::Wumpus   => if map.wumpuses.contains(location)  { return false; },
			Class::Pit      => if map.pits.contains(location)      { return false; },
			Class::Empty    => if false
				|| map.treasures.contains(location)
				|| map.wumpuses.contains(location)
				|| map.pits.contains(location)
			{ return false; },
		}
	}

	// The number of classes cannot be more than possible.
	if map.treasures.len() > Game::COUNT_TREASURES as usize { return false; }
	if map.wumpuses.len()  > Game::COUNT_WUMPUSES  as usize { return false; }
	if map.pits.len()      > Game::COUNT_PITS      as usize { return false; }

	// All treasures must be surrounded with glitters.
	if ! map.treasures
		.iter()
		.flat_map(|&x| x.get_neighbours())
		.filter(|&x| map.discovered.contains(&x))
		.all(|x| map.glitters.contains(&x))
	{ return false; }

	// All wumpuses must be surrounded with stenches.
	if ! map.wumpuses
		.iter()
		.flat_map(|&x| x.get_neighbours())
		.filter(|&x| map.discovered.contains(&x))
		.all(|x| map.stenches.contains(&x))
	{ return false; }

	// All pits must be surrounded with breezes.
	if ! map.pits
		.iter()
		.flat_map(|&x| x.get_neighbours())
		.filter(|&x| map.discovered.contains(&x))
		.all(|x| map.breezes.contains(&x))
	{ return false; }

	// All glitters must have at least one adjacent treasure.
	if ! map.glitters
		.iter()
		.all(|&glitter| glitter
			.get_neighbours()
			.iter()
			.filter(|&neighbour| map.encompass(&neighbour))
			.find(|&neighbour| map.treasures.contains(&neighbour))
			.is_some()
		)
	{ return false; }

	// All stenches must have at least one adjacent wumpus.
	if ! map.stenches
		.iter()
		.all(|&stench| stench
			.get_neighbours()
			.iter()
			.filter(|&neighbour| map.encompass(&neighbour))
			.find(|&neighbour| map.wumpuses.contains(&neighbour))
			.is_some()
		)
	{ return false; }

	// All breezes must have at least one adjacent pit.
	if ! map.breezes
		.iter()
		.all(|&breeze| breeze
			.get_neighbours()
			.iter()
			.filter(|&neighbour| map.encompass(&neighbour))
			.find(|&neighbour| map.pits.contains(&neighbour))
			.is_some()
		)
	{ return false; }

	return true;
}


pub fn pathfind(initial_location: &Coordinate, initial_direction: &Direction, map: &Map) -> (HashMap<Coordinate, Coordinate>, HashMap<Coordinate, i32>) {

	let mut links: HashMap<Coordinate, Coordinate> = HashMap::from([(*initial_location, *initial_location)]);
	let mut dirs: HashMap<Coordinate, Direction> = HashMap::from([(*initial_location, *initial_direction)]);
	let mut costs: HashMap<Coordinate, i32> = HashMap::from([(*initial_location, 0)]);

	let mut queue: VecDeque<Coordinate> = VecDeque::from([*initial_location]);

	while let Some(current_location) = queue.pop_front() {
		for new_location in current_location.get_neighbours() {

			// Ignore locations outside the map.
			if ! map.encompass(&new_location) {
				continue;
			}

			// If this location is available and hasn't been evaluated yet, add to queue.
			if map.discovered.contains(&new_location) && ! links.contains_key(&new_location) {
				queue.push_back(new_location);
			}

			// Get the currently known cost of getting to the location.
			let known_cost = *costs
				.get(&new_location)
				.unwrap_or(&std::i32::MAX)
				;

			// Calculate the new cost of getting to the location.
			let mut new_cost: i32 = costs[&current_location];
			if map.wumpuses.contains(&new_location) { new_cost -= Game::SCORE_WUMPUS; }
			if map.pits.contains(&new_location) { new_cost -= Game::SCORE_PIT; }

			let relative_direction = current_location.get_relative_direction(&new_location).unwrap();
			if relative_direction == dirs[&current_location].rotate_right() { new_cost += 1; }
			if relative_direction == dirs[&current_location].rotate_left() { new_cost += 1; }
			if relative_direction == dirs[&current_location].rotate_back() { new_cost += 2; }
			new_cost += 1;

			// If the new cost is lower, set this path as the preferred one.
			if new_cost < known_cost {
				links.insert(new_location, current_location);
				dirs.insert(new_location, relative_direction);
				costs.insert(new_location, new_cost);
			}
		}
	}

	return (links, costs);
}


pub fn path_to_actions(target: &Coordinate, initial_direction: &Direction, pathmap: &HashMap<Coordinate, Coordinate>) -> Option<Vec<Action>> {

	// If no path to the location is known, return None.
	if ! pathmap.contains_key(&target) {
		return None;
	}

	let mut actions: Vec<Action> = Default::default();

	// Build the chain of locations to get to the target.
	let mut path: Vec<Coordinate> = vec![*target];

	let mut location = target;
	while let Some(new_location) = pathmap.get(location) {
		if new_location != location {
			path.push(*new_location);
			location = new_location;
		}
		else {
			break;
		}
	}
	path.reverse();

	// Turn the location chain into actions.
	let mut direction: Direction = *initial_direction;
	let mut location: Coordinate = path[0];
	for new_location in path {
		if new_location == location { continue; }

		let new_direction = location.get_relative_direction(&new_location).unwrap();
		if new_direction == direction.rotate_back()  { actions.push(Action::Right);  actions.push(Action::Right); }
		if new_direction == direction.rotate_right() { actions.push(Action::Right); }
		if new_direction == direction.rotate_left()  { actions.push(Action::Left); }

		location = new_location;
		direction = new_direction;
		actions.push(Action::Walk);
	}

	return Some(actions);
}


pub fn visualize_map(map: &Map, player_location: &Coordinate, player_direction: &Direction, show_undiscovered: &bool) -> String {

	const SEPARATOR_X: &str = "    ";
	const SEPARATOR_Y: &str = "\n";
	let mut minimap = String::new();

	minimap.push_str(SEPARATOR_Y);
	for y in (0..=map.size.y).rev() {

		// On first line, print player position and location attributes
		for x in 0..=map.size.x {
			minimap.push_str(SEPARATOR_X);

			let location = Coordinate{x, y};
			if !show_undiscovered && !map.discovered.contains(&location) {
				minimap.push_str("xxxx");
				continue;
			}

			// Print the player location and direction
			if location == *player_location {
				minimap.push(match player_direction {
					Direction::North => '^',
					Direction::East  => '>',
					Direction::South => 'v',
					Direction::West  => '<',
				});
			}
			else {
				minimap.push('-');
			}

			// Print the attributes on this location
			if map.treasures.contains(&location) { minimap.push('T') } else { minimap.push('-') }
			if map.wumpuses.contains(&location)  { minimap.push('W') } else { minimap.push('-') }
			if map.pits.contains(&location)      { minimap.push('P') } else { minimap.push('-') }
		}

		minimap.push('\n');

		// On the second row, print player location and location events
		for x in 0..=map.size.x {
			minimap.push_str(SEPARATOR_X);

			let location = Coordinate{x, y};
			if !show_undiscovered && ! map.discovered.contains(&location) {
				minimap.push_str("xxxx");
				continue;
			}

			// Print the player location and direction
			if location == *player_location {
				minimap.push(match player_direction {
					Direction::North => '^',
					Direction::East  => '>',
					Direction::South => 'v',
					Direction::West  => '<',
				});
			}
			else {
				minimap.push('-');
			}

			// Print the events on this location
			if map.glitters.contains(&location) { minimap.push('G') } else { minimap.push('-') }
			if map.stenches.contains(&location) { minimap.push('S') } else { minimap.push('-') }
			if map.breezes.contains(&location)  { minimap.push('B') } else { minimap.push('-') }
		}

		minimap.push('\n');
		minimap.push_str(SEPARATOR_Y);
	}

	minimap.pop();
	return minimap;
}


pub fn calculate_map_possibilities(
	frontier: &[Coordinate],
	possible_treasures: &[Coordinate],
	map: &Map,
	blacklist: &HashMap<Coordinate, Class>,
) -> (i32, HashMap<Coordinate, ClassField<i32>>) {

	let length = frontier.len() + possible_treasures.len();
	let locations: Vec<Coordinate> = [frontier, possible_treasures].concat();
	let mut class_counts: Vec<ClassField<i32>> = vec![Default::default(); length];

	let mut tmp_map = map.clone();
	let mut total_possibilities = 0;
	let mut classes: Vec<Class> = Vec::with_capacity(length);

	// Generate all possible map permutations.
	for perm_treasures in itertools::repeat_n([Class::Empty, Class::Treasure], possible_treasures.len()).multi_cartesian_product() {
		for perm_frontier in itertools::repeat_n(Class::VALUES, frontier.len()).multi_cartesian_product() {

			// Construct a map from the current permutation.
			classes.clear();
			classes.extend_from_slice(&perm_frontier);
			classes.extend_from_slice(&perm_treasures);
			tmp_map.apply_classes(&locations, &classes);

			// Verify that the permutation upholds the game logic.
			if ! is_map_valid(&tmp_map, &blacklist) {
				continue;
			}

			// Count the number of map possibilities, as well as the class count for each location.
			total_possibilities += 1;
			for (i, class) in classes.iter().enumerate() {
				match class {
					Class::Empty    => { class_counts[i].empty    += 1; },
					Class::Treasure => { class_counts[i].treasure += 1; },
					Class::Wumpus   => { class_counts[i].wumpus   += 1; },
					Class::Pit      => { class_counts[i].pit      += 1; },
				}
			}
		}
	}

	// Collect the locations and class counts into a hashmap.
	let mut counts: HashMap<Coordinate, ClassField<i32>> = HashMap::with_capacity(length);
	for (location, class_count) in std::iter::zip(locations.iter(), class_counts.iter()) {
		counts.insert(*location, *class_count);
	}

	return (total_possibilities, counts)
}

