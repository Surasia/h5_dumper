//! Extensions to BufReader.
//!
//! Implements `read_cstring` and `read_fixed_string` that are not present in the regular BufReader.
//! * `read_cstring:` Given a buffer, reads a character until `0x00` is encountered (null termination), returns the `String` collected. Errors when given invalid UTF-8.
//! * `read_fixed_string:` Given a buffer and size, reads characters and collects them into a `String` abd returns it. Ignores unknown UTF-8 encoding, errors out when encountering end-of-file or other IO related errors.
//!
//! These functions are implemented as traits in generics. Requires `<BufReaderExt + Read>` to be satisfied.
//!

use std::io::{self, BufRead, BufReader, Read};

/// Extension trait for BufReader to add custom reading methods.
pub trait BufReaderExt: BufRead {
    /// Reads a UTF-8 encoded C-style string from the reader until a null terminator (0x00) is encountered.
    ///
    /// # Returns
    ///
    /// Returns an `io::Result<String>` containing the read string.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * There's an I/O error while reading from the reader.
    /// * The read bytes are not valid UTF-8.
    fn read_cstring(&mut self) -> io::Result<String> {
        let mut buffer = Vec::new();
        self.read_until(0, &mut buffer)?;
        if buffer.ends_with(&[0]) {
            buffer.pop(); // remove null terminator from buffer
        }
        String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Reads a fixed-length UTF-8 encoded string from the reader.

    /// and trims any null bytes found at the end of the string.
    ///
    /// # Arguments
    ///
    /// * `length` - The number of bytes to read.
    ///
    /// # Returns
    ///
    /// Returns an `io::Result<String>` containing the read string.
    ///
    /// # Errors
    ///
    /// Returns `Ok(())` if the read operation is successful, or an `Err` containing
    /// the I/O error if any reading operation fails.
    fn read_fixed_string(&mut self, length: usize) -> io::Result<String> {
        let mut buffer = vec![0; length];
        self.read_exact(&mut buffer)?;
        Ok(String::from_utf8_lossy(&buffer)
            .trim_end_matches('\0')
            .to_string())
    }
}

impl<R: Read> BufReaderExt for BufReader<R> {}
