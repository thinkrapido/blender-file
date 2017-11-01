# blender-file
rust blender-file interpreter

## Example Code

```rust


extern crate blenderfile;

use blenderfile::bf::*;
use blenderfile::file_block::*;
use blenderfile::sdna::*;


fn main() {

  let bf = BlenderFile::new(&"resources/rust-cube.blend");

  let fbh = FileBlockHeader::new(&bf);

  let sdna = SDNA::new(&bf, &fbh);

//  println!("{:?}\n\n{:?}", &sdna.structures[0].members, &sdna.structures[0].member(&String::from("next")));

/*  for ref s in sdna.structures.iter() {
    println!("{}", &s.pretty_print());
  }
*/

  if let Some(s) = sdna.structure(&String::from("Object")) {
    println!("{}", &s.pretty_print());
  }

}


```
