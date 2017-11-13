
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
  use std::collections::HashMap;

  #[derive(Debug)]
  pub struct FileBlockHeader {
    pub code: String,
    pub size: usize,
    pub content_offset: usize,
    pub offset: usize,
    pub sdna_index: usize,
    pub old_mem_adr: usize,
  }

  #[derive(Debug)]
  pub struct FileBlockHeaderMap {
    map: HashMap<usize, FileBlockHeader>
  }

  impl FileBlockHeader
  {

    fn new(bf: &BlenderFile, offset: usize) -> FileBlockHeader {
      let mut add = 0;
      while add < 4 && bf.content[offset+add] > 0 {
        add += 1;
      }
      FileBlockHeader {
        code: String::from(str::from_utf8(&bf.content[offset..offset+add]).unwrap()),
        size: bf.u32(offset+4usize) as usize,
        content_offset: offset+bf.pointer_size+16usize,
        offset: offset,
        sdna_index: bf.u32(offset+bf.pointer_size+8usize) as usize,
        old_mem_adr: match bf.arch {
          Arch::Arch32 => bf.u32(offset+8usize) as usize,
          Arch::Arch64 => bf.u64(offset+8usize) as usize,
        },
      }
    }
  }

  impl FileBlockHeaderMap
  {

    pub fn new(bf: &BlenderFile) -> FileBlockHeaderMap {
      let mut map = HashMap::new();

      let mut offset = 12;
      let endb = String::from("ENDB");
      loop {
        let fbh = FileBlockHeader::new(bf, offset);
        offset += 16 + bf.pointer_size + fbh.size as usize;
        let stop = fbh.code == endb;
        map.insert(fbh.old_mem_adr, fbh);
        if stop {
          break;
        }
      }

      FileBlockHeaderMap {
        map: map,
      }
    }

    pub fn map(&self) -> &HashMap<usize, FileBlockHeader> {
      &self.map
    }

    pub fn find(&self, code: &str) -> Vec<&FileBlockHeader> {
      let mut out = Vec::<&FileBlockHeader>::new();

      let test = String::from(code);

      for ref val in self.map.values() {

        if (val.code == test) {
          out.push(val);
        }
      }

      out
    }

    pub fn get(&self, ptr: &usize) -> Option<&FileBlockHeader> {
      self.map.get(ptr)
    }
  }
}

pub mod sdna {

  use std::str;

  use regex::Regex;

  use super::bf::*;
  use super::file_block::*;

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
        static ref IS_TIMER_RE: Regex = Regex::new("Timer").unwrap();
      }
      Type {
        size: size,
        is_simple: SDNA::is_simple(&String::from(&name[..])),
        is_timer: IS_TIMER_RE.is_match(&name[..]),
        name: name,
      }
    }
  }

  #[derive(Debug)]
  pub struct Member {
    pub identifier: String,
    pub declaration: String,
    pub ty: Type,
    pub offset: usize,
    pub pointer_type: PointerType,
    pub structure_type: StructureType,
    pub dimensions: Vec<usize>,
    pub size: usize,
  }

  #[derive(Debug)]
  pub struct Structure {
    pub ty: Type,
    pub members: Vec<Member>,
  }

  impl Structure {

    pub fn name(&self) -> String {
      self.ty.name.clone()
    }

    pub fn member(&self, name: &String) -> Option<&Member> {
      let res: Vec<&Member> = self.members.iter().filter(|m| m.identifier == *name).collect();

      let len: usize = res.len();

      if len == 0 {
        None
      }
      else {
        Some(&res[0])
      }
    }

    pub fn pretty_print(&self) -> String {
      let mut out = String::new();

      out.push_str(&format!("{} {{\n", &self.ty.name)[..]);

      for ref m in self.members.iter() {
        let mut name = format!("{}                               ", &m.ty.name);
        name.truncate(20);
        let mut declaration = format!("{}                               ", &m.declaration);
        declaration.truncate(25);
        out.push_str(&format!("\t{}\t{}\t({})\t({})\t({:?}", &name, &declaration, &m.size, &m.offset, &m.structure_type)[..]);
        match m.pointer_type {
          PointerType::Pointer        => out.push_str(", pointer"),
          PointerType::PointerPointer => out.push_str(", pPointer"),
          PointerType::None => (),
        }
        if m.dimensions.len() > 0 {
          out.push_str(&format!(", {}-dim array", &m.dimensions.len())[..]);
        }
        out.push_str(");\n");
      }

      out.push_str("}\n");

      out
    }

  }

  #[derive(Debug)]
  pub enum PointerType {
    None,
    Pointer,
    PointerPointer,
  }

  #[derive(Debug)]
  pub enum StructureType {
    Complex(String),
    Char,
    Str,
    UChar,
    Short,
    UShort,
    Int,
    Long,
    ULong,
    Float,
    Double,
    Int64,
    UInt64,
    Void,
  }

  #[derive(Debug)]
  pub enum Value {
    None,
    Pointer(usize),         // pointer address
    PointerPointer(usize),  // pointer address
    Complex(usize),         // offset
    Char(i8),
    Str(String),
    UChar(u8),
    Short(i16),
    UShort(u16),
    Int(i32),
    Long(i32),
    ULong(i32),
    Float(f32),
    Double(f64),
    Int64(i64),
    UInt64(u64),
  }

  impl SDNA {

    pub fn new(bf: &BlenderFile, map: &FileBlockHeaderMap) -> SDNA {
      let mut out = SDNA {
        types: Vec::new(),
        structures: Vec::new(),
      };

      let dna1 = String::from("DNA1");
      let mut offset = map.map().values().filter(|fbh| fbh.code == dna1 ).next().unwrap().content_offset;

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

    pub fn structure(&self, name: &String) -> Option<&Structure> {
      let res: Vec<&Structure> = self.structures.iter().filter(|s| s.name() == *name).collect();

      let len: usize = res.len();

      if len == 0 {
        None
      }
      else {
        Some(&res[0])
      }
    }

    fn is_simple(source: &String) -> bool {
      lazy_static! {
        static ref SIMPLE: Vec<String> = vec![String::from("char"), String::from("uchar"), String::from("short"), String::from("ushort"), String::from("int"), String::from("long"), String::from("ulong"), String::from("float"), String::from("double"), String::from("int64_t"), String::from("uint64_t"), String::from("void")];
      }

      let res: Vec<&String> = SIMPLE.iter().filter(|s| *s == source).collect();

      res.len() > 0
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
            static ref IDENTIFIER_RE: Regex = Regex::new(r"[_a-zA-Z0-9]+").unwrap();
            static ref IS_POINTER_RE: Regex = Regex::new(r"^\*\w").unwrap();
            static ref IS_POINTER_POINTER_RE: Regex = Regex::new(r"^\*\*\w").unwrap();
            static ref DIMENSIONS_RE: Regex = Regex::new(r"\[([^\]]+)\]").unwrap();
          }

          let mut overall_size = 1;

          let mut identifier = String::from("");
          for cap in IDENTIFIER_RE.captures_iter(&name[..]) {
            identifier = String::from(&cap[0]);
            break;
          }

          let mut dimensions = Vec::new();
          for cap in DIMENSIONS_RE.captures_iter(&name[..]) {
            let size: usize = *(&cap[1].parse().unwrap());
            overall_size *= size;
            dimensions.push(size);
          }

          let pointer_type = {
            if IS_POINTER_POINTER_RE.is_match(&name[..]) {
              PointerType::PointerPointer
            }
            else if IS_POINTER_RE.is_match(&name[..]) {
              PointerType::Pointer
            }
            else {
              PointerType::None
            }
          };
          let structure_type = SDNA::get_structure_type(&ty);

          let member = Member {
            identifier: identifier,
            declaration: String::from(&name[..]),
            ty: ty.clone(),
            offset: member_offset,
            dimensions: dimensions,
            size: overall_size * match &pointer_type {
              &PointerType::Pointer => bf.pointer_size,
              &PointerType::PointerPointer => bf.pointer_size,
              _ => ty.size,
            },
            pointer_type: pointer_type,
            structure_type: structure_type,
          };

          structure.members.push(member);

        }

        vec.push(structure);
      }
    }

    fn get_structure_type(ty: &Type) -> StructureType {
      if ty.name == String::from("char") {
        StructureType::Char
      }
      else if ty.name == String::from("uchar") {
        StructureType::UChar
      }
      else if ty.name == String::from("short") {
        StructureType::Short
      }
      else if ty.name == String::from("ushort") {
        StructureType::UShort
      }
      else if ty.name == String::from("int") {
        StructureType::Int
      }
      else if ty.name == String::from("long") {
        StructureType::Long
      }
      else if ty.name == String::from("ulong") {
        StructureType::ULong
      }
      else if ty.name == String::from("float") {
        StructureType::Float
      }
      else if ty.name == String::from("double") {
        StructureType::Double
      }
      else if ty.name == String::from("int64_t") {
        StructureType::Int64
      }
      else if ty.name == String::from("uint64_t") {
        StructureType::UInt64
      }
      else if ty.name == String::from("void") {
        StructureType::Void
      }
      else {
        StructureType::Complex(ty.name.clone())
      }
    }

    pub fn get_structure_by_index(&self, idx: usize) -> Option<&Structure> {
      self.structures.get(idx)
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
