use std::{mem, panic, str};
use crate::page;

pub struct Table {
    buffer: Vec<page::Page>,
    page_size: usize,
    num_rows: usize,
    row_size: Option<usize>,
    max_string_len: Option<usize>,
}

impl Table {
    /// Constructs a new empty `Table` with specified `page_size` and 
    /// returns it, or `None` if `page_size` is 0. 
    pub fn build(page_size: usize) -> Option<Self> {
        if page_size == 0 {
            return None;
        } else {
            return Some(
                Self { 
                    buffer: Vec::new(), 
                    page_size, 
                    num_rows: 0, 
                    row_size: None, 
                    max_string_len: None 
                }
            );
        }
    }

    pub fn len(&self) -> usize {
        self.num_rows
    }

    pub fn push(&mut self, contents: &[u8]) {
        match self.row_size {
            Some(size) => if contents.len() != size { 
                panic!(
                    "Error: input size {} does not match table row size {}.",
                    contents.len(), 
                    size
                )
            },
            None => {
                if contents.len() > self.page_size { panic!(
                    "Error: row size greater than page size.") }
                self.row_size = Some(contents.len());
            },
        }
        let row_num = self.num_rows + 1;
        // Can unwrap here since self.row_size will never be None at
        // this point.
        let row_size = self.row_size.unwrap();
        let rows_per_page: usize = self.page_size / row_size;
        let page_num: usize = row_num / rows_per_page;
        if page_num >= self.buffer.len() {
            self.buffer.push(
                // Can unwrap here since self.page_size cannot be 0
                // which is the main cause for error.
                unsafe { page::Page::alloc_zeroed(self.page_size).unwrap() }
            );
        }
        // Page_num should always exist since it would have been 
        // allocated above if it didn't.
        let page = self.buffer.get_mut(page_num).unwrap();
        let write_point: usize = (self.num_rows - rows_per_page * page_num) * row_size;
        page.copy_from_slice(write_point, contents);
        self.num_rows = row_num;
    }

    /// Returns a reference to `Row` number `row_id` if it exists, 
    /// or `None` if it doesn't.
    pub fn get(&self, row_id: usize, max_string_len: usize) -> Option<Row> {
        if row_id >= self.num_rows {
            return None;
        }
        let rows_per_page = self.page_size/ self.row_size?;
        let page_num = row_id / rows_per_page;
        let read_point = (row_id - rows_per_page * page_num) * self.row_size?;
        let row_buffer = self.buffer
            .get(page_num)
            .unwrap()
            .read_from_index(read_point, self.row_size?)?;
        Row::deserialise(row_buffer, max_string_len).ok()
    }
}

#[derive(Debug)]
pub enum SerialiseError {
    NoContents,
    StringWriteError,
    StringReadError,
    BufferLenError
}

/// Temporary container for simple tabel rows.
#[derive(Clone)]
pub struct Row {
    pub id: Option<usize>,
    pub username: String,
    pub email: String,
    max_string_len: usize,
}

impl Row {

    /// Construct a `Row` with maximum length of internal strings given 
    /// by `max_string_len`.
    pub fn with_max_str_len(max_string_len: usize) -> Self {
        Self { 
            id: None, 
            username: String::new(), 
            email: String::new(), 
            max_string_len 
        }
    }

    /// Serialises contents and returns buffer, or `None` if `self.id` was 
    /// never set to `Some(value)`.
    pub fn serialise(&self) -> Result<Box<[u8]>, SerialiseError> {
        let id = self.id.map_or_else(|| Err(SerialiseError::NoContents), |x| Ok(x))?;
        let buffer_len = self.max_string_len * 2 + mem::size_of::<usize>();
        // Buffer must be zeroed since this is used to determine the  
        // length of each string during deserialisation.
        let mut buffer = vec![0u8; buffer_len].into_boxed_slice();
        let username = self.username.as_bytes();
        let email = self.email.as_bytes();
        panic::catch_unwind(panic::AssertUnwindSafe(|| {
            buffer[0..username.len()]
                .copy_from_slice(self.username.as_bytes());
            buffer[self.max_string_len..(self.max_string_len + email.len())]
                .copy_from_slice(self.email.as_bytes());
            buffer[self.max_string_len*2..(self.max_string_len * 2 + mem::size_of::<usize>())]
                .copy_from_slice(&id.to_le_bytes());
        })).map_err(|_| SerialiseError::StringWriteError)?;
        Ok(buffer)
    }

    pub fn deserialise(serial: Box<[u8]>, max_string_len: usize) -> Result<Self, SerialiseError> {
        if serial.len() != max_string_len * 2 + mem::size_of::<usize>() {
            return Err(SerialiseError::BufferLenError);
        }
        fn extract_string(buffer: &[u8]) -> Result<String, SerialiseError> {
            let string_len = find_first_zero(buffer.iter())
                .map_or_else(|| Err(SerialiseError::StringReadError), |x| Ok(x))?;
            let string = String::from(
                str::from_utf8(&buffer[0..string_len])
                    .map_err(|_| SerialiseError::StringReadError)?
            );
            Ok(string)
        }
        let username = extract_string(&serial[0..max_string_len])?;
        let email = extract_string(&serial[max_string_len..(max_string_len*2)])?;
        let id = usize::from_le_bytes(serial[(max_string_len*2)..].try_into().unwrap());
        Ok(Self { id: Some(id), username, email, max_string_len })
    }
}

/// Finds the location of the first zero byte and returns it, or `None` if 
/// all bytes are none zero.
fn find_first_zero<'a, T: Iterator<Item = &'a u8>>(x: T) -> Option<usize> {
    let mut counter: usize = 0;
    x.into_iter().find(|x| {
        counter += 1;
        **x == 0
    })?;
    counter -= 1;
    Some(counter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_first_zero_returns_correct_index() {
        let first_zero_id = (0..90).step_by(10);
        for id_expected in first_zero_id {
            let mut array = [1u8; 100];
            array[id_expected] = 0;
            let id = find_first_zero(array.iter()).unwrap();
            assert_eq!(id_expected, id);
        }
    }
    
    #[test]
    fn find_first_zero_returns_none_when_no_zeros() {
        let zero_id = find_first_zero([1;20].iter());
        assert!(matches!(zero_id, None));
    }

    #[test]
    fn assert_deserialised_serialised_row_is_unchanged() {
        let max_string_len = 100;
        let mut row = Row::with_max_str_len(max_string_len);
        row.id = Some(0);
        row.username = String::from("hello world");
        row.email = String::from("helloworld@something.fun");
        let row_serialised = row.serialise().unwrap();
        let row_deserialised = Row::deserialise(
            row_serialised, 
            max_string_len
        ).unwrap();
        assert_eq!(row.id, row_deserialised.id);
        assert_eq!(row.username, row_deserialised.username);
        assert_eq!(row.email, row_deserialised.email);
    }

    #[test]
    fn assert_data_written_and_read_from_table_is_correct() {
        let max_string_len = 100;
        let page_size = 1024;
        let mut table = Table::build(page_size).unwrap();
        let mut row = Row::with_max_str_len(max_string_len);
        row.id = Some(0);
        row.username = String::from("hello world");
        row.email = String::from("helloworld@funmail.com");
        table.push(&row.serialise().unwrap());
        let row_output = table.get(0, max_string_len).unwrap();
        assert_eq!(row.id, row_output.id);
        assert_eq!(row.username, row_output.username);
        assert_eq!(row.email, row_output.email);
    }
}