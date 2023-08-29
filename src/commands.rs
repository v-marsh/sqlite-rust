use std::process;
use crate::table;

const max_string_len: usize = 100;

pub enum ExitStatus {
    Success = 0,
    Failure,
}

pub enum StatementType {
    Insert(table::Row),
    Select,
}

pub struct InsertContents {
    pub id: usize,
    pub username: String,
    pub email: String,
}

impl InsertContents {
    pub fn new() -> Self {
        Self { id: 0, username: String::new(), email: String::new() }
    }
}

pub fn handle_meta_command(line: &str) -> Result<(), String> {
    let line = line.get(1..line.len()).unwrap();
    match line {
        "exit" => process::exit(ExitStatus::Success as i32),
        _ => Err(format!("unknown command or invalid arguments:  \"{line}\". Enter \".help\" for help")),
    }
}

pub fn prepare_statement(line: &str) -> Result<StatementType, String> {
     if line.starts_with("insert") {
        let print_error = |statement: &str| {
            let statement = statement.clone();
            format!("Unable to parse statement {statement}");
        };
        let columns = line
            .split(' ')
            .count();
        if columns != 4 {print_error(line)}
        let mut contents = table::Row::with_max_str_len(max_string_len);
        let mut elements = line.split(' ').skip(1);
        match elements.next().unwrap().parse::<usize>() {
            Ok(value) => contents.id = Some(value),
            Err(_) => print_error(line),
        }
        contents.username = String::from(elements.next().unwrap());
        contents.email = String::from(elements.next().unwrap());
        return Ok(StatementType::Insert(contents));        
    } else if line.starts_with("select") {
        return Ok(StatementType::Select);
    } else {
        return Err(format!("Unrecognised keyword at start of \'{line}\'\n"));
    }
}


pub fn execute_statement(statement: StatementType, table: &mut table::Table) {
    match statement {
        StatementType::Insert(contents) => {
            table.push(&contents.serialise().unwrap())
        },
        StatementType::Select => {
            for i in 0..table.len() {
                let row = table.get(i, max_string_len).unwrap();
                println!(
                    "({}, {}, {})", row.id.unwrap(), row.username, row.email
                );
            }
        }
    };
}