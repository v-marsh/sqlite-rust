use std::{process, io};
use sqlite::user_io::*;
use sqlite::commands::*;
use sqlite::table;

const MAX_BUFFER_CAPACITY: usize = 4096; 
const PAGE_SIZE: usize = 4096;

fn main() {
    if let Err(e) = configure_env(io::stdout()) {
        eprintln!("{}", e);
        process::exit(ExitStatus::Failure as i32);
    };
    let mut input_buffer = InputBuffer::with_capacity(MAX_BUFFER_CAPACITY);
    let mut table = table::Table::build(PAGE_SIZE).unwrap();
    loop {
        if let Err(e) = prompt_user_input(io::stdin().lock(), io::stdout(), &mut input_buffer) {
            eprintln!("{}", e);
            process::exit(ExitStatus::Failure as i32);
        };
        if input_buffer.buffer().is_empty() {
        continue
        }
        if input_buffer.buffer().get(0..1).unwrap() == "." {
            if let Err(e) = handle_meta_command(input_buffer.buffer()) {
                eprintln!("{}", e);
            };
        } else {
            let statement = prepare_statement(input_buffer.buffer());
            let statement = match statement {
                Ok(val) => val,
                Err(msg) => {
                    if let Err(e) = display_bad_statement_message(io::stdout(), msg) {
                        eprintln!("{}", e);
                        process::exit(ExitStatus::Failure as i32);
                    };
                    continue;
                }
            };
            execute_statement(statement, &mut table);
        }
    }
}