use flate2::read::GzDecoder;
use std::io::{self, Read, Seek};

/// An internal representation of a reader that can read from a plain file or a gzip file
pub enum Reader<R: Read> {
    Plain(R),
    Gzip(GzDecoder<R>),
}

#[doc(hidden)]
impl<R: Read> Read for Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Reader::Plain(r) => r.read(buf),
            Reader::Gzip(r) => r.read(buf),
        }
    }
}

impl<R: Read> Reader<R> {
    /// Read the given number of elements into a new vector
    ///
    /// # Arguments
    /// * `size` - The number of elements to read
    ///
    /// # Returns
    /// * A vector containing the read elements
    ///
    /// # Example
    /// ```
    /// use flate2::read::GzDecoder;
    /// use std::fs::File;
    /// use std::path::Path;
    /// use crate::util::Reader;
    ///
    /// fn example() {
    ///     let path = Path::new("tests/data/test.mtx.gz");
    ///     let file = File::open(&path).unwrap();
    ///     let reader = Reader::Gzip(GzDecoder::new(file));
    ///     let mut reader = Reader::Gzip(GzDecoder::new(file));
    ///     let data: Vec<f32> = reader.read_into_vector(10).unwrap();
    /// }
    /// ```
    ///
    /// # Exceptions
    /// * If there is an issue reading the data or if there is not enough data, an `io::Error` will be returned.
    pub fn read_into_vector<T>(&mut self, size: usize) -> io::Result<Vec<T>> {
        let mut data = Vec::with_capacity(size);
        // Directly read into the vector sized number of elements in a single call
        unsafe {
            //data.set_len(size);
            let data_ptr = data.as_mut_ptr();
            let data_slice = std::slice::from_raw_parts_mut(
                data_ptr as *mut u8,
                size * std::mem::size_of::<T>(),
            );
            self.read_exact(data_slice)?;
            data.set_len(size);
        }

        Ok(data)
    }
}

#[doc(hidden)]
impl<R: Seek + Read> Seek for Reader<R> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match self {
            Reader::Plain(r) => r.seek(pos),
            Reader::Gzip(r) => {
                // Since you can't directly seek on a GzDecoder, we need to read through the data
                // until we reach the desired position.
                match pos {
                    io::SeekFrom::Start(_) => Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Cannot seek to an absolute position in a Gzip file",
                    )),
                    io::SeekFrom::End(_) => Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Cannot seek from the end of a Gzip file",
                    )),
                    io::SeekFrom::Current(pos) => {
                        if pos < 0 {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidInput,
                                "Cannot seek to a negative position in a GZip file",
                            ));
                        }
                        let pos = pos as usize;
                        // Create a small fixed sized buffer of 4kb and iteratively read from the GzDecoder
                        // until we reach the desired position.
                        const MAX_SIZE: usize = 4096;
                        let mut buffer = [0; MAX_SIZE];
                        let mut total_read: usize = 0;
                        loop {
                            let remaining = pos - total_read;
                            let read = r.read(&mut buffer[..min(remaining, MAX_SIZE)])?;
                            if read == 0 {
                                break;
                            }
                            total_read += read;
                            if total_read >= pos {
                                break;
                            }
                        }
                        Ok(0)
                    }
                }
            }
        }
    }
}

#[doc(hidden)]
fn min(a: usize, b: usize) -> usize {
    if a < b { a } else { b }
}

/// Check if the given file name ends with the specified suffix
/// # Arguments
/// * `file_name` - The stem to check
/// * `suffix` - The suffix to check for
/// # Returns
/// * A boolean indicating if the file name ends with the suffix
/// # Example
/// ```
/// use std::ffi::OsStr;
/// use crate::util::ends_with;
/// fn example() {
///    let stem = OsStr::new("example.mtx");
///   let suffix = "mtx";   
///   assert_eq!(ends_with(stem, suffix), true);    
pub fn ends_with(file_name: &std::ffi::OsStr, suffix: &str) -> bool {
    file_name.to_str()
        .is_some_and(|s| s.ends_with(suffix))
}
