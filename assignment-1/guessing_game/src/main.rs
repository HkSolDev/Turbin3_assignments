use rand::Rng;
use std::{cmp::Ordering, io};

fn main() {
    // The main fun is the entry point of the program this is where the program start
    println!("Guess the number!");
    let secret_no = rand::thread_rng().gen_range(1..=100);

    loop {
        println!("Please input your guess.");
        let mut guess = String::new();

        io::stdin()
            .read_line(&mut guess) // this call the read_line method on the standard input handle to get the input from the user and pass the guess to where to store the string
            .expect("Failed to read the line");

        let guess: u32 = match guess.trim().parse(){
            Ok(num) => num,
            Err(_) => continue,
        };
 println!("You guessed: {guess}");
        match guess.cmp(&secret_no) {
            Ordering::Less => println!("Too low"),
            Ordering::Greater => println!("Too high"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }
}
