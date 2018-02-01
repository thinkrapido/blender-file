
use std::fs::File;
use std::io::Read;
use std::str;
use endian_type::types::*;

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

    BlenderFile {
      file: String::from(filename),
      version: String::from(str::from_utf8(&vector[9..12]).unwrap()),
      content: vector,
      pointer_size: match &arch {
                      &Arch::Arch32 => 4,
                      &Arch::Arch64 => 8,
                    },
      arch: arch,
      endian: endian,
    }
  }

  pub fn u16(&self, offset: usize) -> u16 {
    match self.endian {
      Endian::LittleEndian => u16_le::from_bytes(&self.content[offset..]).into(),
      Endian::BigEndian    => u16_be::from_bytes(&self.content[offset..]).into(),
    }
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

  pub fn i16(&self, offset: usize) -> i16 {
    match self.endian {
      Endian::LittleEndian => i16_le::from_bytes(&self.content[offset..]).into(),
      Endian::BigEndian    => i16_be::from_bytes(&self.content[offset..]).into(),
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

}