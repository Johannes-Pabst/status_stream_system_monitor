use std::{fmt::Display, io::{self, Write}};

pub fn ask_yn<T>(question: T, stdy: bool) -> bool
where
    T: Display,
{
    let mut input = String::new();
    loop {
        print!("{} ", question);
        if stdy {
            print!("[Y/n]: ");
        } else {
            print!("[y/N]: ");
        }
        io::stdout().flush().unwrap();
        input.clear();
        io::stdin().read_line(&mut input).unwrap();
        let lowercase = input.trim().to_lowercase();
        match lowercase.as_str() {
            "y" => return true,
            "n" => return false,
            "" => return stdy,
            _ => eprintln!("Invalid input, please enter Y or N."),
        }
    }
}