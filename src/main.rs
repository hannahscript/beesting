use std::io;

fn read() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    return Result::Ok(input);
}

fn eval(text: String) -> String {
    return text;
}

fn print(text: String) -> String {
    return text;
}

fn rep() -> io::Result<String> {
    let input = read()?;
    let result = eval(input);
    return Ok(print(result));
}

fn main() {
    loop {
        print!("user> ");
        let output_result = rep();
        match output_result {
            Ok(output) => println!("{}", output),
            Err(err) => eprintln!("Error occurred: {:?}", err),
        }
    }
}
