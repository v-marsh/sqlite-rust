use std::io::{self, Write, BufRead};


pub struct InputBuffer {
    buffer: String,
    buffer_length: usize,
}

impl InputBuffer {
    pub fn with_capacity(capacity: usize) -> Self {
        InputBuffer {
            buffer: String::with_capacity(capacity), 
            buffer_length: capacity, 
        }
    }

    pub fn buffer(&mut self) -> &mut String {
        &mut self.buffer
    }

    pub fn buffer_length(&self) -> &usize {
        &self.buffer_length
    }
}

pub fn configure_env(mut stream: impl Write) -> io::Result<()> {
    stream.write(format!("SQLite rust clone version {}\n", 
        env!("CARGO_PKG_VERSION")).as_bytes())?;
    stream.write(b"Enter \".help\" for instructions\n")?;
    stream.flush()?;
    Ok(())
}

pub fn prompt_user_input(input_stream: impl BufRead, mut output_stream: impl Write, input_buffer: &mut InputBuffer) -> io::Result<()>{
    input_buffer.buffer().clear();
    output_stream.write(b"db> ")?;
    output_stream.flush()?;
    input_stream
        .take(input_buffer.buffer_length().clone() as u64)
        .read_line(input_buffer.buffer())?;
    input_buffer.buffer().pop();
    Ok(())
}

pub fn display_bad_statement_message(mut steam: impl Write, statement: String) -> io::Result<()>{
    steam.write(statement.as_bytes())?;
    Ok(())

}