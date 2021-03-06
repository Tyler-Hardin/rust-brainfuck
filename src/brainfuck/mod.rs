use std::collections::HashMap;
use std::io::{stdin,stdout,Read,Write};
use std::str::Chars;

// Used to represent data
pub type Data = i64;

// Used to represent a pointer to a memory cell on the data tape.
pub type DataPtr = usize;

// Used to represent a pointer to an instruction on the instruction tape.
pub type InstPtr = usize;

// Used to represent instructions in the instruction tape
#[derive(Clone,Copy,Debug)]
pub enum Inst {
	IncPtr,
	DecPtr,
	IncData,
	DecData,
	In,
	Out,
	Forward,
	Back,
	Null,
}

// Used to represent the state of a brainfuck machine
#[derive(Debug,Default)]
pub struct State {
	term : bool,
	depth : usize,
	data_ptr : DataPtr,
	inst_ptr : InstPtr,
	data_tape : HashMap<DataPtr, Data>,
	inst_tape : Vec<Inst>,
}

impl State {
	pub fn from_chars(c : &mut Chars) -> Self {
		// Parse instructions into a list of enums representing instructions.
		let mut inst_tape = vec!();
		Self::parse_chars(c, &mut inst_tape);
		
		// Return the initialized state.
		State {
			term : false,
			depth : 0,
			data_ptr : 0,
			inst_ptr : 0,
			data_tape : HashMap::new(),
			inst_tape : inst_tape,
		}
	}
	
	pub fn from_str(s : &str) -> Self {
		Self::from_chars(&mut s.chars())
	}
	
	/**
	 * Steps the state forward by executing the instruction currently at the 
	 * instruction pointer.
	 */
	pub fn step(&mut self) {
		match self.inst_tape.get(self.inst_ptr) {
			// Increment the data pointer.
			Some(&Inst::IncPtr) => {
				self.data_ptr += 1;
				self.inst_ptr += 1;
			}
			// Decrement the data pointer.
			Some(&Inst::DecPtr) => {
				self.data_ptr -= 1;
				self.inst_ptr += 1;
			}
			// Increment the data at the data pointer.
			Some(&Inst::IncData) => {
				let data_ptr = self.data_ptr;
				{
					let data = self.get_data_mut(&data_ptr);
					*data += 1;
				}
				self.inst_ptr += 1;
			}
			// Decrement the data at the data pointer.
			Some(&Inst::DecData) => {
				let data_ptr = self.data_ptr;
				{
					let data = self.get_data_mut(&data_ptr);
					*data -= 1;
				}
				self.inst_ptr += 1;
			}
			// Read input into the memory cell at the data pointer.
			Some(&Inst::In) => {
				let mut buf = [0;1];
				let data_ptr = self.data_ptr;
				{
					let data = self.get_data_mut(&data_ptr);
					*data = match stdin().read(&mut buf) {
						Ok(1) => {
							// Cast to u64 first so it doesn't sign extend.
							buf[0] as u64 as Data
						},
						Ok(0) => { // EOF?
						 	-1
						}
						_ => panic!("Read failed."),
					}
				}
				self.inst_ptr += 1;
			}
			// Print output from the memory cell at the data pointer.
			Some(&Inst::Out) => {
				let mut buf = [0;1];
				let data_ptr = self.data_ptr;
				{
					let data = self.get_data(&data_ptr);
					buf[0] = data as u8;
				}
				match stdout().write(&mut buf) {
					Ok(1) => { },
					_ => panic!("Write failed."),
				}
				self.inst_ptr += 1;
			}
			// Jump forward.
			Some(&Inst::Forward) => {
				let data_ptr = self.data_ptr;
				let jump = {
					self.get_data(&data_ptr) == 0
				};
				
				if jump {
					self.inst_ptr = self.find_matching_rbrace();
				} else {
					self.depth += 1;
				}
				self.inst_ptr += 1;
			}
			// Jump backward.
			Some(&Inst::Back) => {
				let data_ptr = self.data_ptr;
				let jump = {
					self.get_data(&data_ptr) != 0
				};
				
				if jump {
					self.inst_ptr = self.find_matching_lbrace();
				} else {
					self.depth -= 1;
				}
				self.inst_ptr += 1;
			}
			// Handle instruction. (Just increment instruction pointer.
			Some(&Inst::Null) => {
				self.inst_ptr += 1;
			}
			None => { self.term = true; }
		}
	}
	
	/**
	 * Run until termination, i.e. the end of the data tape.
	 */
	pub fn run(&mut self) {
		while !self.term {
			self.step();
		}
	}
	
	/**
	 * Returns the data in a data cell.
	 *
	 * @param ptr	the address of the data cell
	 */
	pub fn get_data(&self, ptr : &DataPtr) -> Data {
		if let Some(data) = self.data_tape.get(ptr) {
			*data
		} else {
			0
		}
	}
	
	/**
	 * Get a mutable reference to a data cell.
	 *
	 * @param ptr	the address of the data cell
	 */
	pub fn get_data_mut(&mut self, ptr : &DataPtr) -> &mut Data {
		self.data_tape.entry(*ptr).or_insert(0)
	}
	
	/**
	 * Get a instruction from the instruction tape or Null if the tape isn't
	 * initialized at this position.
	 *
	 * @param ptr	the address of the intruction cell
	 */
	pub fn get_inst(&mut self, ptr : &InstPtr) -> Inst {
		if let Some(inst) = self.inst_tape.get(*ptr) {
			*inst
		} else {
			Inst::Null
		}
	}
	
	/**
	 * Get a mutable reference to an instruction cell. If the pointer is past the end, the tape
	 * is grown by appending Null's until it is long enough.
	 *
	 * @param ptr	the address of the intruction cell
	 */
	pub fn get_inst_mut(&mut self, ptr : &InstPtr) -> &mut Inst {
		if self.inst_tape.len() < *ptr {
			self.inst_tape.resize(*ptr + 1, Inst::Null);
		}
		self.inst_tape.get_mut(*ptr).expect("Inst tape not extended correctly.")
	}
	
	/**
	 * Parses a (potentially partial) script, appending instructions to inst_tape.
	 */
	fn parse_chars(c : &mut Chars, inst_tape : &mut Vec<Inst>) {
		// Parse instructions into a list of enums representing instructions.
		for inst in c.filter_map(|c| {
			match c {
				'>' => Some(Inst::IncPtr),
				'<' => Some(Inst::DecPtr),
				'+' => Some(Inst::IncData),
				'-' => Some(Inst::DecData),
				',' => Some(Inst::In),
				'.' => Some(Inst::Out),
				'[' => Some(Inst::Forward),
				']' => Some(Inst::Back),
				_ => Some(Inst::Null) // BF skips non-instruction char's.
			}
		}) {
			inst_tape.push(inst);
		}
	}
	
	/**
	 * Finds the matching [ to the current ].
	 */
	fn find_matching_lbrace(&self) -> InstPtr {
		let mut cur_depth = self.depth;
		let mut i = self.inst_ptr;
		let mut found = false;
		while i > 0 && !found {
			i -= 1;
			match self.inst_tape.get(i) {
				Some(&Inst::Forward) => {
					if cur_depth == self.depth {
						found = true;
					} else {
						cur_depth -= 1;
					}
				}
				Some(&Inst::Back) => {
					cur_depth += 1;
				}
				_ => { }
			}
		}
		
		if found {
			i
		} else {
			panic!("Unmatched braces.");
		}
	}
	
	
	/**
	 * Finds the matching ] to the current [.
	 */
	fn find_matching_rbrace(&self) -> InstPtr {
		let mut cur_depth = self.depth;
		let mut i = self.inst_ptr;
		let mut found = false;
		while i > 0 && !found {
			i += 1;
			match self.inst_tape.get(i) {
				Some(&Inst::Forward) => {
					cur_depth += 1;
				}
				Some(&Inst::Back) => {
					if cur_depth == self.depth {
						found = true;
					} else {
						cur_depth -= 1;
					}
				}
				_ => { }
			}
		}
		
		if found {
			i
		} else {
			panic!("Unmatched braces.");
		}
	}
}
