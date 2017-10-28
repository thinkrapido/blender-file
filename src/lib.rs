
use std::fs::File;
use std::io::Read;
use std::vec::Vec;
use std::str;


#[derive(Debug)]
pub struct BlenderFile {
  pub file: String,
  pub version: String,
  pub content: Vec<u8>,
}

impl BlenderFile {

  pub fn new(filename: &str) -> BlenderFile {

    let mut file = File::open(filename).expect(&format!("File {} cannot be opened.", filename));

    let mut vector = Vec::new();

    file.read_to_end(&mut vector).expect(&format!("File {:?} cannot be read.", file));

    let out = BlenderFile { 
      file: String::from(filename),
      version: String::from(str::from_utf8(&vector[..12]).unwrap()),
      content: vector,
    };


    out
  }

}


#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn create_new_blenderfile() {
    let bf = BlenderFile::new("resources/rust-cube.blend");
    let result = bf.version;
    let expected = String::from("BLENDER-v279");
    assert_eq!(expected, result);
  }

  #[test]
  #[should_panic]
  fn create_new_blenderfile_by_unkonwn_file() {
    BlenderFile::new("resources/rust-cube.blend.none");
  }

}
