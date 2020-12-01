extern crate exitcode;

struct Settings {
	letters: Vec<char>, // '_' are unknown characters
	minimum_number_of_words: u8,
	maximum_number_of_words: u8,
	minimum_word_length: u8,
	maximum_word_length: u8,
}

#[derive(Clone,Debug)]
struct AnagramObject {
	string: String,
	value: Option<Vec<AnagramObject>>,
}

impl std::string::ToString for AnagramObject {
	fn to_string(&self) -> String {
		let mut s = String::new();
		match &self.value {
			Some(v) => {
				s = format!("{}{}",s,format!("{}:[",self.string,));
				for x in v.iter() {
					s = format!("{}{}",s,&x.to_string());
				}
				s = format!("{}{}",s,"],");

			},
			None => {
				s = format!("{}{}",s,format!("{},",self.string));
			}
		}
		return s;
	}
}


fn main() {
	let initial_settings = Settings {
		letters: vec!['b', 'a', 'l', 'm', 's', '_', '_', '_', '_', '_',],
		minimum_number_of_words: 1,
		maximum_number_of_words: 2,
		minimum_word_length: 4,
		maximum_word_length: 6,
	};

	let valid_length_permutations = find_valid_permuations_of_word_lengths(&initial_settings);
	let valid_word_lengths = find_valid_word_lengths(&valid_length_permutations);
	let possible_words = load_all_words(&(initial_settings.letters), &valid_word_lengths);
	let anagram_trees = build_anagram_trees(&initial_settings,&possible_words,&valid_length_permutations);
	let anaragm_solutions = compile_anagram_solutions(anagram_trees, &String::from(" "));
}

fn find_valid_permuations_of_word_lengths(settings: &Settings) -> Vec<Vec<u8>> {
	match find_valid_permuations_of_word_lengths_recurser(settings, (*settings).letters.len() as u8) {
		Some(s) => {
			let mut results = Vec::new();
			for permutation in s.iter() {
				if (*permutation).len() as u8 >= settings.minimum_number_of_words && (*permutation).len() as u8 <= settings.maximum_number_of_words {
					results.push(permutation.clone());
				}
			}
			if results.is_empty() {
				println!("Unable to find any valid length permutations givin word size requirements.");
				std::process::exit(exitcode::CONFIG);
			} else {
				return results;
			}
			
		},
		None => {
			println!("Unable to find any valid length permutations givin word size requirements.");
			std::process::exit(exitcode::CONFIG);
		}
	}
}

fn find_valid_permuations_of_word_lengths_recurser(settings: &Settings, leftover_length: u8) -> Option<Vec<Vec<u8>>> {
	let mut results = Vec::new();
	if leftover_length >= settings.minimum_word_length {
		for i in settings.minimum_word_length..(std::cmp::min(settings.maximum_word_length,leftover_length)+1) {
			if leftover_length == i {
				let mut single_letter = Vec::new();
				single_letter.push(leftover_length);
				results.push(single_letter);
			} else {
				let sub_results = find_valid_permuations_of_word_lengths_recurser(settings, leftover_length - i);
				match sub_results {
					Some(s) => {
						for inner_vec in s.iter() {
							let mut temp_vec = Vec::new();
							temp_vec.push(i);
							for inner_value in inner_vec.iter() {
								temp_vec.push(*inner_value);
							}
							results.push(temp_vec)
						}
					},
					None => {}
				}
			}
		}
	}
	if results.is_empty(){
		return None
	} else {
		return Some(results)
	}
}

fn find_valid_word_lengths(length_permuations: &Vec<Vec<u8>>) -> Vec<u8> {
	let mut valid_word_lengths = Vec::new();
	for permuation in (*length_permuations).iter() {
		for value in permuation.iter() {
			match valid_word_lengths.iter().find(|&&x| x == *value) {
				Some(_) => { },
				None => {
					valid_word_lengths.push(*value);
				}
			}
		}
	}
	valid_word_lengths.sort();
	return valid_word_lengths;
}

fn load_all_words(letters: &Vec<char>, valid_word_lengths: &Vec<u8>) -> std::collections::HashMap<u8,Vec<String>> {
	// open file (just assuming it will work atm)
	let file = std::fs::File::open("src/words_small.txt").unwrap();
	// create a buffered reader
	let reader = std::io::BufReader::new(file);
	// load buffer read functions
	use std::io::BufRead;
	// create hashmap of vectors to read words into
	let mut hashmap: std::collections::HashMap<u8,Vec<String>> = std::collections::HashMap::new();
	// read line by line (just assuming it will work atm)
	for line in reader.lines() {
		let word = line.unwrap();
		let index = word.len() as u8;
		// preempetive word culling by length and then letters (optimized short circut order)
		if !validity_length(valid_word_lengths, index) || !validity_letters(&mut letters.clone(), &word) {
			continue
		}
		let result = hashmap.get_mut(&index);
		match result {
			Some(valid_vec) => {
				valid_vec.push(word);
			}
			None => {
				let mut vc = std::vec::Vec::new();
				vc.push(word);
				hashmap.insert(index, vc);
			}
		}
		
	}
	return hashmap;
}

// checks if the length of a word is valid
// 
// fairly optimized (uses returned lengths from possible_lenght_permutaions)
fn validity_length(valid_word_lengths: &Vec<u8>, length: u8) -> bool {
	match valid_word_lengths.iter().find(|&&x| x == length) {
		Some(_) => {
			return true;
		},
		None => {
			return false;
		}
	}
}

// checks if a word could be valid given the remaining letters
//
// fairly optimized (could be better if number of unknown letters was an u8 instead of using iter reverse position search for '_')
// consumes letters
fn validity_letters(letters: &mut Vec<char>, word: &String) -> bool {
	for letter in word.chars() {
		match letters.iter().position(|x| *x==letter) {
			Some(position) => {
				letters.remove(position);
			},
			None => {
				match letters.iter().rposition(|x| *x=='_') {
					Some(position) => {
						letters.remove(position);
					},
					None => {
						return false;
					}
				}
			}
		}
	}
	return true;
}

fn build_anagram_trees(settings: &Settings, possible_words: &std::collections::HashMap<u8,Vec<String>>, valid_length_permutations: &Vec<Vec<u8>>) -> Vec<Vec<AnagramObject>> {
	let mut result = Vec::new();
	for length_permuations in valid_length_permutations.iter() {
		let sub_result = build_anagram_trees_recursive(settings.letters.clone(), &mut length_permuations.clone(), possible_words);
		match sub_result {
			Some(s) => {
				result.push(s.clone());		
			},
			None => {}
		}
	}
	if result.is_empty(){
		println!("Unable to find any valid anagrams givin requirements.");
		std::process::exit(exitcode::CONFIG);
	} else {
		return result;
	}
	
}

fn build_anagram_trees_recursive<'a>(remaining_letters: Vec<char>, permuation: &mut Vec<u8>, possible_words: &std::collections::HashMap<u8,Vec<String>>) -> Option<Vec<AnagramObject>> {
	let mut results = Vec::new();
	let length = permuation.pop().unwrap();
	match possible_words.get(&length) {
		Some(s) => {
			for word in s {
				let mut remaining_letters_clone = remaining_letters.clone();	
				match validity_letters(&mut remaining_letters_clone, &word) {
					true => {
						if permuation.len() != 0 {
							match build_anagram_trees_recursive(remaining_letters_clone, &mut permuation.clone(), &possible_words) {
								Some(s) => {
									// s contains valid sub anagram word arrays
									results.push(AnagramObject{
										string: word.clone(),
										value: Some(s.clone())});
								},
								None => {
									// means there were no valid anagrams
								}
							}
						} else {
							results.push(AnagramObject{
								string: word.clone(),
								value: None});
						}
					},
					false => {
						continue;
					}
				}	
			}
		},
		None => {}
	}
	if !results.is_empty() {
		return Some(results);
	} else {
		return None;
	}
}

fn compile_anagram_solutions(anagram_trees: Vec<Vec<AnagramObject>>, sepetator: &String) -> Vec<String>{
	let mut results = Vec::new();
	for shape_group in anagram_trees.iter() {
		for word_group in shape_group.iter() {
			results.append(&mut compile_anagram_solutions_recursive(None,word_group,sepetator));
		}
	}
	return results;
}


fn compile_anagram_solutions_recursive(root: Option<String>, ao: &AnagramObject, sepetator: &String) -> Vec<String>{
	let rt;
	match root {
		Some(s) => {
			rt = format!("{}{}{}",s,sepetator,ao.string);
		},
		None => {
			rt = format!("{}",ao.string);
		}
	}
	match &ao.value {
		Some(v) => {
			// mush with root and recurse over v
			let mut results = Vec::new();
			for x in v.iter() {
				results.append(&mut compile_anagram_solutions_recursive(Some(rt.clone()),&x,sepetator));
			}
			return results;
		},
		None => {
			// mush with root and print
			let mut results = Vec::new();
			results.push(rt);
			return results;
		}
	}
}












// fn solve_puzzle(possibles :Vec<Vec<AnagramObject>>) {
// 	for shape_group in possibles.iter() {
// 		for word_group in shape_group.iter() {
// 			solve_puzzle_re(format!(""),word_group);
// 		}
// 	}
// }

// fn solve_puzzle_re(root: String, ao: &AnagramObject) {
// 	let rt = format!("{}{}",root,ao.string);
// 	match &ao.value {
// 		Some(v) => {
// 			// mush with root and recurse over v
// 			for x in v.iter() {
// 				solve_puzzle_re(rt.clone(),&x);
// 			}
// 		},
// 		None => {
// 			// mush with root and check site
// 			let stringg: String = format!("https://www.cephalofair.com/{}",rt);
// 			println!("!");
// 			// let body = reqwest::blocking::get(&stringg);
// 			// 	match body {
// 			// 		Ok(v) => {
// 			// 			let txxt = v.text().unwrap();
// 			// 			if txxt.contains("Page Not Found - Cephalofair Games"){
// 			// 				//println!("N : {}",rt);
// 			// 			} else {
// 			// 				println!("Y : {}",rt);
// 			// 			}
// 			// 		}
// 			// 		Err(_) => {
// 			// 			println!("redo: {}",rt);
// 			// 		}
// 			// 	}
// 		}
// 	}
// }