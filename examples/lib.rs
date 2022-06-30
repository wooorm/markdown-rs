extern crate micromark;
use micromark::micromark;

fn main() {
    env_logger::init();

    println!("{:?}", micromark("[](irc:///help)"));
}
