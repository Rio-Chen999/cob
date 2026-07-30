use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::io::Write;
use std::thread;
use std::time::{ Duration, UNIX_EPOCH };
use std::collections::HashMap;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type FileStore = HashMap<PathBuf, Duration>;

pub struct Instruction<'a> {
    command: Option<&'a str>,
    args: Vec<&'a str>,
    file_store: FileStore,
}

impl<'a> Instruction<'a> {
    pub fn new(command: &'a str) -> Self {
        Self {
            command: Some(command),
            args: vec![],
            file_store: HashMap::new(),
        }
    }
    pub fn arg(&mut self, arg: &'a str) -> &mut Self {
        self.args.push(arg);
        self
    }
    pub fn args(&mut self, args: &[&'a str]) -> &mut Self {
        self.args.extend(args);
        self
    }
    fn build(&self) -> Result<()> {
        if let Some(command) = self.command {
            let output = Command::new(command)
                .args(&self.args)
                .output()?;
            // "\x1B[2J" will clean screen
            // "\x1B[1;1H" will move cursor to left-top corner
            print!("\x1B[2J\x1B[1;1H");
            std::io::stdout().flush().unwrap();
            println!("Watching: {:?}", std::env::current_dir().unwrap());
            println!("[INFO]: {}", format!("{} {}", command, self.args.join(" ")));
            if !output.stdout.is_empty() {
                match String::from_utf8(output.stdout) {
                    Ok(s) => println!("{}", s),
                    Err(e) => println!("Failed to convert stdout to string: {}", e),
                }
            }
            if !output.stderr.is_empty() {
                match String::from_utf8(output.stderr) {
                    Ok(s) => println!("{}", s),
                    Err(e) => println!("Failed to convert stderr to string: {}", e),
                }
            }
        }
        Ok(())
    }
    fn handle_file(&mut self, path: PathBuf, metadata: &std::fs::Metadata) -> Result<bool> {
        if let Some(ext) = path.extension() {
            if ext == "rs" {
                let sys_time = metadata.modified()?;
                let cur_modified = sys_time.duration_since(UNIX_EPOCH).unwrap();
                if let Some(last_modified) = self.file_store.get_mut(&path) {
                    if *last_modified != cur_modified {
                        *last_modified = cur_modified;
                        self.build()?;
                        return Ok(true);
                    }
                } else {
                    self.file_store.insert(path, cur_modified);
                }
            }
        }
        Ok(false)
    }
    fn traversal_proj_bfs(&mut self, root: PathBuf) -> Result<()> {
        let entries = fs::read_dir(&root).unwrap_or_else(|_| {
            eprintln!("ERROR: Failed to read directory {:?}", root);
            panic!();
        });
        let mut dir_vec = vec![];
        for entry in entries {
            let entry = entry?;
            let path_b = entry.path();
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                dir_vec.push(path_b);
            } else {
                if self.handle_file(path_b, &metadata)? { break };
            }
        }

        for dir in dir_vec {
            self.traversal_proj_bfs(dir)?;
        }

        Ok(())
    }
    // fn traversal_proj_dfs(&mut self, root: PathBuf) -> Result<()> {
    //     let entries = fs::read_dir(&root).unwrap_or_else(|_| {
    //         eprintln!("ERROR: Failed to read directory {:?}", root);
    //         panic!();
    //     });
    //     for entry in entries {
    //         let entry = entry?;
    //         let path_b = entry.path();
    //         let metadata = entry.metadata()?;
    //         if metadata.is_dir() {
    //             self.traversal_proj_dfs(entry.path())?;
    //         } else {
    //             if self.handle_file(path_b, &metadata)? { break };
    //         }
    //     }
    //
    //     Ok(())
    // }
    pub fn watch(&mut self) -> ! {
        if let Err(_) = self.build() {
            println!("ERROR: Something wrong happended in build stage");
            panic!();
        }
        loop {
            let _ = self.traversal_proj_bfs(PathBuf::from("."));
            thread::sleep(Duration::from_secs(2));
        }
    }
}
