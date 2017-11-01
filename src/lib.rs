
#[macro_use]
extern crate lazy_static;
extern crate endian_type;
extern crate regex;

pub mod bf {

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
    
}

pub mod file_block {

  use super::bf::*;
  use std::str;

  #[derive(Debug)]
  pub struct FileBlockHeader {
    pub code: String,
    pub size: usize,
    pub content_offset: usize,
    pub offset: usize,
    pub sdna_index: usize,
    pub old_mem_adr: u64,
  }

  impl FileBlockHeader
  {

    pub fn new(bf: &BlenderFile) -> Vec<FileBlockHeader> {
      let mut out = Vec::new();

      let mut offset = 12;
      let endb = String::from("ENDB");
      loop {
        let fbh = FileBlockHeader::new_bfh(bf, offset);
        offset += 16 + bf.pointer_size + fbh.size as usize;
        let stop = fbh.code == endb;
        out.push(fbh);
        if stop {
          break;
        }
      }

      out
    }

    fn new_bfh(bf: &BlenderFile, offset: usize) -> FileBlockHeader {
      FileBlockHeader {
        code: String::from(str::from_utf8(&bf.content[offset..offset+4]).unwrap()),
        size: bf.u32(offset+4usize) as usize,
        content_offset: offset+bf.pointer_size+16usize,
        offset: offset,
        sdna_index: bf.u32(offset+bf.pointer_size+8usize) as usize,
        old_mem_adr: match bf.arch {
          Arch::Arch32 => bf.u32(offset+8usize) as u64,
          Arch::Arch64 => bf.u64(offset+8usize),
        },
      }
    }

  }
}

pub mod sdna {

  use super::bf::*;
  use super::file_block::*;
  use std::str;
  use regex::Regex;

  #[derive(Debug)]
  pub struct SDNA {
    pub types: Vec<Type>,
    pub structures: Vec<Structure>,
  }

  #[derive(Debug, Clone)]
  pub struct Type {
    pub name: String,
    pub size: usize,
    pub is_simple: bool,
    pub is_timer: bool,
  }

  impl Type {
    pub fn new((name, size): (String, usize)) -> Type {
      lazy_static! {
        static ref IS_SIMPLE_RE: Regex = Regex::new("^[a-z]").unwrap();
        static ref IS_TIMER_RE: Regex = Regex::new("Timer").unwrap();
      }
      Type {
        size: size,
        is_simple: IS_SIMPLE_RE.is_match(&name[..]),
        is_timer: IS_TIMER_RE.is_match(&name[..]),
        name: name,
      }
    }
  }

  #[derive(Debug)]
  pub struct Member {
    pub identifier: String,
    pub ty: Type,
    pub offset: usize,
    pub is_pointer: bool,
    pub is_pointer_pointer: bool,
    pub dimensions: Vec<usize>,
    pub size: usize,
  }

  #[derive(Debug)]
  pub struct Structure {
    pub ty: Type,
    pub members: Vec<Member>,
  }

  impl SDNA {

    pub fn new(bf: &BlenderFile, fbh: &Vec<FileBlockHeader>) -> SDNA {
      let mut out = SDNA {
        types: Vec::new(),
        structures: Vec::new(),
      };

      let mut offset = SDNA::find_dna1_offset(&fbh).unwrap();

      if !SDNA::compare_identifier("SDNA", offset, &bf) {
        panic!("cannot find SDNA signature");
      }
      offset += 4;
      if !SDNA::compare_identifier("NAME", offset, &bf) {
        panic!("cannot find NAME signature");
      }

      let mut names = Vec::<String>::new();
      SDNA::get_names(&mut names, &"TYPE", &mut offset, &bf);

      SDNA::add_types(&mut out.types, &mut offset, &bf);

      SDNA::add_structures(&mut out.structures, &mut offset, &names, &out.types, &bf);

      out
    }

    fn add_types(vec: &mut Vec<Type>, offset: &mut usize, bf: &BlenderFile) {
      let mut types = Vec::<String>::new();
      SDNA::get_names(&mut types, &"TLEN", offset, &bf);

      *offset += 4;
      let mut type_lens = Vec::<usize>::new();
      for _i in 0..types.len() {
        let len = bf.i16(*offset) as usize;
        type_lens.push(len);
        *offset += 2;
      }

      let it = types.into_iter().zip(type_lens.into_iter());

      for ty in it {
        vec.push(Type::new(ty));
      }
    }

    fn add_structures(vec: &mut Vec<Structure>, offset: &mut usize, 
                      names: &Vec<String>, types: &Vec<Type>, bf: &BlenderFile) {
      if !SDNA::compare_identifier("STRC", *offset, &bf) {
        panic!("cannot find {} signature", "STRC");
      }

      *offset += 4;
      let len = bf.i32(*offset) as usize;
      *offset += 4;
      for _i in 0..len {
        let structure_idx = bf.i16(*offset) as usize;
        let mut structure = Structure {
          ty: types[structure_idx].clone(),
          members: Vec::new(),
        };
        *offset += 2;

  //      print!("{} {{\n", structure);

        let fields = bf.i16(*offset) as usize;
        *offset += 2;
        let mut member_offset = 0;
        for f in 0..fields {
          let type_idx = bf.i16(*offset) as usize;
          let ty = types[type_idx].clone();
          *offset += 2;

          let name_idx = bf.i16(*offset) as usize;
          let name = &names[name_idx];
          *offset += 2;

          if f > 0 {
            member_offset += structure.members[f-1].size;
          }

          lazy_static! {
            static ref IS_POINTER_RE: Regex = Regex::new(r"^\*\w").unwrap();
            static ref IS_POINTER_POINTER_RE: Regex = Regex::new(r"^\*\*\w").unwrap();
            static ref DIMENSIONS_RE: Regex = Regex::new(r"\[([^\]]+)\]").unwrap();
          }

          let mut dimensions = Vec::new();
          let mut overall_size = 1;
          for cap in DIMENSIONS_RE.captures_iter(&name[..]) {
            let size: usize = *(&cap[1].parse().unwrap());
            overall_size *= size;
            dimensions.push(size);
          }

          let is_pointer_pointer = IS_POINTER_POINTER_RE.is_match(&name[..]);
          let is_pointer = IS_POINTER_RE.is_match(&name[..]) || is_pointer_pointer;

          let member = Member {
            identifier: String::from(&name[..]),
            ty: ty.clone(),
            offset: member_offset,
            is_pointer: is_pointer,
            is_pointer_pointer: is_pointer_pointer,
            dimensions: dimensions,
            size: overall_size * match is_pointer {
              true => bf.pointer_size,
              _ => ty.size,
            },
          };

          structure.members.push(member);

  /*        if DIMENSIONS_RE.is_match(&name[..]) {
            println!("{:?}", member);
          }

  *///        println!("{:?}", member);

  //        print!("\t{}\t{};\n", ty, name);
        }

  //      print!("{:?}\n\n", structure);
        vec.push(structure);
      }
    }

    fn find_dna1_offset(fbh: &Vec<FileBlockHeader>) -> Option<usize> {
      let dna1 = String::from("DNA1");

      for ref fb in fbh.iter() {
        if fb.code == dna1 {
          return Some(fb.content_offset);
        }
      }

      None
    }

    fn get_names(names: &mut Vec<String>, breaker: &str, offset: &mut usize, bf: &BlenderFile) {
      let len = bf.i32((*offset) + 4) as usize;

      *offset += 8;
      while !SDNA::compare_identifier(breaker, *offset, &bf) {
        let (name, new_offset) = SDNA::get_name(*offset, &bf);
        *offset = new_offset;
        if name != "" {
          names.push(name);
        }
      }
      if !SDNA::compare_identifier(breaker, *offset, &bf) {
        panic!("cannot find {} signature", breaker);
      }
      assert_eq!(len, names.len());
    }

    fn compare_identifier(source: &str, offset: usize, bf: &BlenderFile) -> bool {
        let mut out = true;
        for (i, c) in source.chars().enumerate() {
          out = out && (c == bf.content[offset + i] as char);
        }
        out
    }

    fn get_name(offset: usize, bf: &BlenderFile) -> (String, usize) {
        let mut search = offset;
        while bf.content[search] != 0 {
          search = search + 1;
        }
        (String::from(str::from_utf8(&bf.content[offset..search]).unwrap()), search + 1)
    }
    
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
