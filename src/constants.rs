// Constants used in the project. These are "convention over configuration" for now.

pub const DATABASE_DIR: &str = "./data/db";
pub const DATABASE_SUFFIX: &str = ".v4.sqlite";

pub const FILES_DIR: &str = "./data/files";
/// Directory where fully uploaded and hash-checked files are stored
/// (in future, the files should also be s3-uploaded).
pub const FILES_FINAL_SUBDIR: &str = "final";
