# Intro
这是一个为了练习Rust而Fork过来的仓库，没有其他意义  
Fork From https://github.com/bingryan/call  

Emoji is interesting
```Rust
use console::style;  
use indicatif::HumanDuration;
use crate::config::{CallSystemConfig, INIT_CONFIG, LOOKING_GLASS, SPARKLE};
// SPARKLE is a console::Emoji, which only print in std::env::var("LANG").to_uppercase().ends_with("UTF-8")
let started = Instant::now();  
...  
...
println!("{} Done in {}", SPARKLE, HumanDuration(started.elapsed()));  
```    
  
file operations
```Rust
// seem to the only way to move files is:
fs::rename(src,dest);

fs::copy(src, dest);

let mut file = File::create(&path)?;
// File::{open,create} only read and only write, if you want more options:
// use std:fs::OpenOptions
let mut file = OpenOptions::new()
.read(true)
.write(true)
.create(true)// create file if not exist, either open it
.truncate(true) // clear it
// .append(true) // add mode
.open(&path)?;
file.write_all(content.as_bytes())?;

```    


commands you can use:
```shell
./call -h
./call i
./call 
./call .
./call "."
./call '\-l'
./call '\-la'
./call '\-l -a'
./call '\-l'
./call '\-l' -c ls
./call '\-l \-a' -c ls
./call '\-a' -c ifconfig
./call "\-l"
./call "\-la"
./call "." --command ls
```