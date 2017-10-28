
use std::fs::File;
use std::io::Read;
use std::vec::Vec;
use std::str;

#[derive(Debug)]
pub enum Endian {
  BigEndian,
  LittleEndian,
}

#[derive(Debug)]
pub enum Arch {
  Arch32,
  Arch64,
}

#[derive(Debug)]
pub struct BlenderFile {
  pub file: String,
  pub version: String,
  pub endian: Endian,
  pub arch: Arch,
  pub content: Vec<u8>,
}

impl BlenderFile {

  pub fn new(filename: &str) -> BlenderFile {

    let mut file = File::open(filename).expect(&format!("File {} cannot be opened.", filename));

    let mut vector = Vec::new();

    file.read_to_end(&mut vector).expect(&format!("File {:?} cannot be read.", file));

    assert_eq!([0x42u8, 0x4cu8, 0x45u8, 0x4eu8, 0x44u8, 0x45u8, 0x52u8], vector[..7], "It is not a blender file.");

    let arch = match vector[7] as char {
        '-' => Arch::Arch64,
        '_' => Arch::Arch32,
        _ => panic!("architecture marker doesn't match"),
      };

    let endian = match vector[8] as char {
        'v' => Endian::LittleEndian,
        'V' => Endian::BigEndian,
        _ => panic!("endian marker doesn't match"),
      };

    let out = BlenderFile {
      file: String::from(filename),
      version: String::from(str::from_utf8(&vector[9..12]).unwrap()),
      content: vector,
      arch: arch,
      endian: endian,
    };


    out
  }

}


#[cfg(test)]
mod tests {

  use super::*;

  macro_rules! matches{
    ($e: expr, $p: pat) => (
      match $e {
        $p => true,
        _ => false,
      }
    )
  }

  #[test]
  fn create_new_blenderfile() {
    let bf = BlenderFile::new("resources/rust-cube.blend");
    let result = bf.version;
    let expected = String::from("279");
    assert_eq!(expected, result);
  }

  #[test]
  #[should_panic]
  fn create_new_blenderfile_by_unkonwn_file() {
    BlenderFile::new("resources/rust-cube.blend.none");
  }

  #[test]
  fn verify_endian() {
    let bf = BlenderFile::new("resources/rust-cube.blend");
    assert!(matches!(bf.endian, Endian::LittleEndian));
  }

  #[test]
  fn verify_architecture() {
    let bf = BlenderFile::new("resources/rust-cube.blend");
    assert!(matches!(bf.arch, Arch::Arch64));
  }

}
