extern crate rlua;
extern crate clap;
extern crate serde;
extern crate toml;
extern crate git2;

use std::{path::{Path, PathBuf}, io::Read, collections::{HashMap, HashSet}};

use clap::{Command, AppSettings, Arg};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize,Default)]
struct Config{
    //a hashmap of subcommands which can be created via cloning a skeleton repository
    skeletons:HashMap<String, Box<str>>,
    // a hashmap of subcommands which can be created via running an external executable
    scripts: HashMap<String,PathBuf>
}
pub fn main() {
    //
    let config:Config = toml::from_str({
        let mut buf = String::new();
        let _ = std::fs::File::open(
            {let mut n = dirs::config_dir().expect("no config dir so edit the source code to make it work buddy");
                n.push("pag-project-init/config.toml");
                n
            }).expect("error opening file").read_to_string(&mut buf);
        buf
    }.as_str()).unwrap_or_default();

    let Config {scripts,skeletons} = config;

    // Making sure the skeletons and scripts don't have overlap that will cause issue later
    {
        let scripts = scripts.keys().map(String::clone).collect::<HashSet<String>>();
        let skeletons = skeletons.keys().map(String::clone).collect::<HashSet<String>>();
        if scripts.union(&skeletons).count() > 0 {
            panic!("Overlap between skeletons and scripts");
        }
    }

    let mut program = Command::new("project-init")
        .subcommands(skeletons.keys().map(|v|Command::new(v).arg(Arg::new("output_dir").takes_value(true))))
        .subcommands(scripts.keys().map(|v|Command::new(v).arg(Arg::new("script_args").takes_value(true))));
    program.build();
    let matches = program.get_matches();
    for skelly in skeletons {
        if let Some(sub) = matches.subcommand_matches(skelly.0){
            match sub.get_one::<String>(""){
                Some(loc)=>{
                    
                }
                None=>{
                    println!("failed to provide a path to clone the skeleton directory into");
                    std::process::exit(1);
                }
            }
        }
    }
}
