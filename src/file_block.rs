
use super::file::*;
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

      if val.code == test {
        out.push(val);
      }
    }

    out
  }

  pub fn get(&self, ptr: &usize) -> Option<&FileBlockHeader> {
    self.map.get(ptr)
  }
}
