
#[macro_use]
extern crate lazy_static;
extern crate endian_type;
extern crate regex;

pub mod file;
pub mod file_block;
pub mod sdna;

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
