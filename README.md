### Continuation Of Building
This is a rust file that automatically compiles when the .rs file in the listening working directory changes, intended for personal use

### Quick Start

```rust
// build.rs in working directory
mod cob;
use cob::Instruction;

fn main() -> ! {
    let mut instruction = Instruction::new("rustc");
    instruction
        .args(&["-o", "main"])
        .arg("main.rs")
        .watch();
}
```
```bash
./build
```
