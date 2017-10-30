
extern crate endian_type;

use endian_type::types::*;

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
  pub pointer_size: usize,
  pub content: Vec<u8>,
  pub file_block_headers: Vec<FileBlockHeader>,
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

    let mut out = BlenderFile {
      file: String::from(filename),
      version: String::from(str::from_utf8(&vector[9..12]).unwrap()),
      content: vector,
      pointer_size: match &arch {
                      &Arch::Arch32 => 4,
                      &Arch::Arch64 => 8,
                    },
      arch: arch,
      endian: endian,
      file_block_headers: Vec::new(),
    };

    out.load_file_block_headers();

    out
  }

  pub fn u32(&self, offset: usize) -> u32 {
    match self.endian {
      Endian::LittleEndian => u32_le::from_bytes(&self.content[offset..]).into(),
      Endian::BigEndian    => u32_be::from_bytes(&self.content[offset..]).into(),
    }
  }

  pub fn u64(&self, offset: usize) -> u64 {
    match self.endian {
      Endian::LittleEndian => u64_le::from_bytes(&self.content[offset..]).into(),
      Endian::BigEndian    => u64_be::from_bytes(&self.content[offset..]).into(),
    }
  }

  pub fn i32(&self, offset: usize) -> i32 {
    match self.endian {
      Endian::LittleEndian => i32_le::from_bytes(&self.content[offset..]).into(),
      Endian::BigEndian    => i32_be::from_bytes(&self.content[offset..]).into(),
    }
  }

  pub fn i64(&self, offset: usize) -> i64 {
    match self.endian {
      Endian::LittleEndian => i64_le::from_bytes(&self.content[offset..]).into(),
      Endian::BigEndian    => i64_be::from_bytes(&self.content[offset..]).into(),
    }
  }

  fn load_file_block_headers(&mut self) {
    let mut offset = 12;
    let endb = String::from("ENDB");
    loop {
      let fbh = FileBlockHeader::new(self, offset);
      offset = offset + 16 + self.pointer_size + fbh.size as usize;
      let stop = fbh.code == endb;
      self.file_block_headers.push(fbh);
      if stop {
        break;
      }
    }
  }

}

#[derive(Debug)]
pub struct FileBlockHeader {
  pub offset: usize,
  pub code: String,
  pub size: u32,
  pub old_mem_adr: u64,
  pub sdna_index: u32,
  pub count: u32,
}

impl FileBlockHeader
{

  pub fn new(bf: &BlenderFile, offset: usize) -> FileBlockHeader {
    let out = FileBlockHeader {
      offset: offset,
      code: String::from(str::from_utf8(&bf.content[offset..offset+4]).unwrap()),
      size: bf.u32(offset+4usize),
      old_mem_adr: match bf.arch {
        Arch::Arch32 => bf.u32(offset+8usize) as u64,
        Arch::Arch64 => bf.u64(offset+8usize),
      },
      sdna_index: bf.u32(offset+bf.pointer_size+8usize),
      count: bf.u32(offset+bf.pointer_size+12usize),
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
