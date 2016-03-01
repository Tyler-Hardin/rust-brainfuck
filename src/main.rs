mod brainfuck;

use brainfuck::State;

fn main() {
	let mut state = State::from_str("++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>\
		+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.");
	state.run();
}
