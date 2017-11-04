# blender-file
rust blender-file interpreter

## Prepare

Create an executable project in sibling folder.

Execute ```cp -r ../blender-file/resources . && cargo watch -x run -w . -w ../blender-file -s 'mkdir -p target/debug && cp -r resources target/debug'```

## Example Code

```rust


  let bf = BlenderFile::new(&"resources/rust-cube.blend");

  let fbh_map = FileBlockHeaderMap::new(&bf);

  let sdna = SDNA::new(&bf, &fbh_map);

  if let Some(s) = sdna.structure(&String::from("Object")) {
    println!("{}", &s.pretty_print());
  }

  if let Some(s) = sdna.structure(&String::from("Mesh")) {
    println!("{}", &s.pretty_print());
  }

  let out = bf.u64(350868 + 232) as usize;
  println!("{}", out);

  let vert = &fbh_map.get(&out);
  println!("{:?}", vert);

  let structure = sdna.get_structure_by_index(vert.unwrap().sdna_index);
  println!("{}", structure.unwrap().pretty_print());


```
