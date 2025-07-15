use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

use crate::util::Reader;

/// A struct that represents a matrix
pub struct Matrix {
    pub data: Vec<f32>,
    pub rows: usize,
    pub cols: usize,
    pub indexes: Vec<Vec<u32>>,
}

impl Matrix {
    /// Create a new matrix using an EMME .mtx or .mtx.gz file
    ///
    /// # Arguments
    /// * `file_path` - A string slice that holds the path to the file
    ///
    /// # Returns
    /// * A Result containing the Matrix if successful, or an io::Error if there was an issue reading the file
    pub fn from_emme_file(file_path: &str) -> io::Result<Matrix> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut reader = if file_path.ends_with(".gz") {
            Reader::Gzip(GzDecoder::new(reader))
        } else {
            Reader::Plain(reader)
        };

        let magic_number = reader.read_u32::<LittleEndian>()?;

        if magic_number != 0xC4D4F1B2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid header"));
        }

        let _version = reader.read_u32::<LittleEndian>()?;
        // float32 = 1, float64 = 2, int32 = 3, int64 = 4, but we are going to assume float32
        let _data_type = reader.read_u32::<LittleEndian>()?;
        let dimensions = reader.read_u32::<LittleEndian>()? as usize;

        // There should be 2 dimensions with indexes are are the same length and contain the same values

        if dimensions != 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid dimensions",
            ));
        }

        // This is only valid because we check for the number of dimensions above.
        let mut index_length: [usize; 2] = [0; 2];
        for index in index_length.iter_mut().take(dimensions) {
            let len = reader.read_u32::<LittleEndian>()? as usize;
            *index = len;
        }

        let mut indexes: Vec<Vec<u32>> = Vec::new();
        // In this case we need a copy of the dimensions
        for index in index_length.iter() {
            let data = reader.read_into_vector(*index)?;
            indexes.push(data);
        }

        // Read the data data payload
        let size = index_length[0] * index_length[1];
        let data: Vec<f32> = reader.read_into_vector(size)?;

        let rows = index_length[0];
        let cols = index_length[1];

        Ok(Matrix {
            data,
            rows,
            cols,
            indexes,
        })
    }

    // Get all the values in a row
    ///
    /// # Arguments
    /// * `row` - The row index
    ///
    /// # Returns
    /// * A Option containing the values at the given row or None if the indexes are out
    pub fn get_row(&self, row: usize) -> Option<&[f32]> {
        if row < self.rows {
                let start = row * self.cols;
                let end = start + self.cols;
                Some(&self.data[start..end])
        } else {
            None
        }
    }

    /// Create a new matrix that has all of the same values as the given matrix
    /// # Arguments
    ///
    /// * `writer` - A stream to write to.
    ///
    /// # Returns
    ///
    /// * A Result if the write was successful, or an io::Error if there was an issue writing the file
    pub fn write_csv_square(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        let col_indexes = &self.indexes[0];
        let row_indexes = &self.indexes[1];
        // Write the header for columns
        write!(writer, "Row\\Col")?;
        for item in col_indexes.iter() {
            write!(writer, ",{item}")?;
        }
        writeln!(writer)?;
        for (row_index, row) in row_indexes.iter().enumerate() {
            // Write the row index
            write!(writer, "{row}")?;
            let row_data = self.get_row(row_index).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid row index")
            })?;

            // Write the row values
            for item in row_data.iter() {
                write!(writer, ",{item:.5}")?;
            }
            writeln!(writer)?;
        }
        Ok(())
    }

    pub fn write_csv_column(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        let col_indexes = &self.indexes[0];
        let row_indexes = &self.indexes[1];
        let number_of_columns = col_indexes.len();
        // Write the header
        writeln!(writer, "Origin,Destination,Value")?;
        for (row_index, row) in row_indexes.iter().enumerate() {
            // Write the row index
            let row_data = self.get_row(row_index).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid row index")
            })?;
            for col_index in 0..number_of_columns {
                let value = row_data[col_index];
                let col = col_indexes[col_index];
                writeln!(writer, "{row},{col},{value:.5}")?;
            }
        }
        Ok(())
    }
}
