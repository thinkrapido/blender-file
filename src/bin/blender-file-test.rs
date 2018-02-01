extern crate blenderfile;

use blenderfile::file::*;
use blenderfile::file_block::*;
use blenderfile::sdna::*;

fn main() {
  let bf = BlenderFile::new(&"resources/rust-cube.blend");

  let fbh_map = FileBlockHeaderMap::new(&bf);

  let sdna = SDNA::new(&bf, &fbh_map);

  if let Some(s) = sdna.structure(&String::from("Object")) {
    println!("{}", &s.pretty_print());
  }

  if let Some(s) = sdna.structure(&String::from("Mesh")) {
    println!("{}", &s.pretty_print());
  }

  let me = fbh_map.find("ME");
  println!("{:?}", me);

  let out = bf.u64(me[0].content_offset + 232) as usize;
  println!("{}", out);

  let vert = &fbh_map.get(&out);
  println!("{:?}", vert);

  let structure = sdna.structure_by_index(vert.unwrap().sdna_index);
  println!("{}", structure.unwrap().pretty_print());

  let vert = sdna.query(&bf, &fbh_map, "OB", "data");
  println!("{:?}", vert);

}

