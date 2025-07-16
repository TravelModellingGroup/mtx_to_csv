# mtx_to_csv

This console program is designed to convert mtx / mtx.gz files to CSV files.  This is typically
done when a person wants to manually inspect the values.

## Compiling

mtx_to_csv requires compiles with Rust's Cargo system.

```cmd
cargo build --release;
```

## Running

Running the code is also possible from Cargo.  Doing so without any parameters will give you
a prompt of how to use the program.

```cmd
cargo run --release;
```

There are two output formats, square (default) and column based for the CSV. To output
in the column format use the -c flag.


For a square CSV:

```cmd
cargo run --release -- path_to_matrix.mtx.gz
```

For a column CSV:

```cmd
cargo run --release -- -c path_to_matrix.mtx.gz
```

When deploying the code you would replace `cargo run --release` with `mtx_to_csv -c path_to_matrix.mtx.gz` if you've added it to
the system path.

### Square CSV

Here is a simple example of a 3x3 matrix where the TAZ are 1,2,3 with some sample data.

|Origin\Destination|	1|	2|	3|
-------------------|-----|----|---|
|1	|0.1	|0.2	|0.3|
|2	|0.4	|0.5	|0.6|
|3	|0.7	|0.8	|0.9|


### Column CSV

Here is the same example as a column based CSV.

|Origin |Destination |Value|
|-------|------------|-----|
|1|1|0.1|
|1|2|0.2|
|1|3|0.3|
|2|1|0.4|
|2|2|0.5|
|2|3|0.6|
|3|1|0.7|
|3|2|0.8|
|3|3|0.9|

Depending on the size of your zone system column based CSV files might have too many rows
for use within Excel, so please take that into consideration.
