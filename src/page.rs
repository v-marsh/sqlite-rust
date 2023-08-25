use std::{alloc::{self, GlobalAlloc}, ptr};

#[derive(Debug)]
pub enum PageAllocationError{
    LayoutError(alloc::LayoutError),
    MemoryAllocationError,
}

/// A block of raw heap-allocated system memory.
pub struct Page {
    // Note: buffer must be u8 (one byte) since many of the implemented
    // methods rely on one unit of buffer to equal one byte for things 
    // such as offset calculations.
    buffer: *mut u8,
    layout: alloc::Layout,
}

impl Page {
    /// Constructs a `Page` from an allocated block of 0-initialised heap memory with `size` bytes
    /// and double word (8 byte) alignment.
    /// 
    /// # Errors
    /// 
    /// Returns [`Err`] if `size` is 0 or the [`GlobalAlloc::alloc`] 
    /// method fails.
    /// 
    /// # Safety
    /// 
    /// This function is unsafe becuase undefined behaviour can occur if 
    /// the allocator registered with the `#[global_allocator]` is 
    /// changed before the object is dropped.
    pub unsafe fn alloc_zeroed(size: usize) -> Result<Self, PageAllocationError> {
        // from_size_align is required to avoid attempting a 0 size 
        // allocation in unsafe block
        let layout = alloc::Layout::from_size_align(size, 8)
            .map_err(|e| PageAllocationError::LayoutError(e))?;
        let buffer;
        unsafe {
            // alloc_zeroed used for ease of debugging
            buffer = alloc::alloc_zeroed(layout.clone());
            if buffer.is_null() {
                return Err(PageAllocationError::MemoryAllocationError);
            } else {
                return Ok(Self {buffer, layout});
            }           
        }
    }

    /// Get the size of the allocated buffer in bytes.
    pub fn size(&self) -> usize {
        self.layout.size()
    }

    /// Copies all elements from `src` to `self`. The contents of `src`
    /// are written into `self` starting at index `loc`.
    /// 
    /// The length of `src` + `loc` must be smaller than [`size`].
    /// 
    /// [`size`]: Self::size
    /// 
    /// # Panics
    /// Panics if the length of `src` + `loc` is greater than [`size`].
    pub fn copy_from_slice(&mut self, loc: usize, src: &[u8] ) {
        if src.len() + loc > self.size() {
            panic!(
                "source slice is too large to write from location {}",
                loc
            );
        }
        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(), self.buffer, src.len())
        }
    }

    /// Reads `count` bytes from `self` starting at `loc`, copies them 
    /// into an array and returns the copied contents.
    /// 
    /// * If `count` = 0, returns `None`.
    /// * If `loc` + `count` is geater than [`size`], returns `None`.
    /// 
    /// [`size`]: Self::size
    pub fn read_from_index(& self, loc: usize, count: usize) -> Option<Box<[u8]>> {
        if count  == 0 {
            return None;
        }
        if loc + count > self.size() {
            return None;
        }
        let mut output = vec![0; count];
        unsafe {
            ptr::copy_nonoverlapping(self.buffer.add(loc), output.as_mut_ptr(), count);
        }
        Some(output.into_boxed_slice())
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        unsafe {
            alloc::dealloc(self.buffer, self.layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;
    
    #[test]
    fn page_size_returns_correct_value() {
        let size_expected: usize = 8;
        let page;
        unsafe {
            page = Page::alloc_zeroed(size_expected).unwrap();
        }
        assert_eq!(size_expected, page.size()) 
    }

    #[test]
    fn write_and_read_from_page_returns_original_values() {
        let page_size: usize = 16;
        let mut page = unsafe {
            Page::alloc_zeroed(page_size).unwrap()
        };
        let contents = "hello world".as_bytes();
        page.copy_from_slice(0, contents);
        let contents_read = page.read_from_index(0, contents.len()).unwrap();
        let contents = str::from_utf8(contents).unwrap();
        let contents_read = str::from_utf8(&contents_read).unwrap();
        assert_eq!(contents, contents_read);
    }
}