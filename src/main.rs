extern crate clap;
extern crate serde;
extern crate toml;
extern crate git2;

use std::{path::{PathBuf, Path}, io::{Read, Cursor}, collections::{HashMap, HashSet}, ffi::OsStr};

use clap::{Command, Arg};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize,Default)]
struct Subcommands{
    //a hashmap of subcommands which can be created via cloning a skeleton repository
    skeletons:HashMap<String, String>,
    // a hashmap of subcommands which can be created via running an external executable
    scripts: HashMap<String,PathBuf>
}
#[derive(Serialize, Deserialize,Default)]
struct Config{
    subcommands: Subcommands
}


#[derive(Debug)]
enum Errors{IoErr(std::io::Error), GitErr(git2::Error), CliErr(CliError), Unknown}
impl From<std::io::Error> for Errors {
    fn from(e: std::io::Error) -> Self {
        Self::IoErr(e)
    }
}
impl From<git2::Error> for Errors {
    fn from(e: git2::Error) -> Self {
        Self::GitErr(e)
    }
}
impl From<CliError> for Errors{
    fn from(e: CliError) -> Self{
        Self::CliErr(e)
    }
}

fn main() -> Result<(),Errors> {

    let config:Config = toml::from_str({
        let mut buf = String::new();
        let _ = std::fs::File::open(
            {let mut n = dirs::config_dir().expect("no config dir so edit the source code to make it work buddy");
                n.push(clap::crate_name!().to_owned() + "/config.toml");
                n
            }).expect("error opening file").read_to_string(&mut buf);
        buf
    }.as_str()).unwrap_or_default();

    let Config {subcommands:Subcommands { skeletons, scripts }}  = config;

    // Making sure the skeletons and scripts don't have overlap that will cause issue later
    {
        let scripts = scripts.keys().map(String::clone).collect::<HashSet<String>>();
        let skeletons = skeletons.keys().map(String::clone).collect::<HashSet<String>>();
        if scripts.intersection(&skeletons).count() > 0 {
            panic!("Overlap between skeletons and scripts");
        }
    }

    let mut program = Command::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .subcommands(skeletons.keys().map(|v|Command::new(v).arg(Arg::new("output_dir").takes_value(true).action(clap::ArgAction::Append))))
        .subcommands(scripts.keys().map(|v|Command::new(v).arg(Arg::new("script_args").takes_value(true))));
    let mut short_help = String::new();
    let mut long_help = String::new();
    {
        let mut short_help_buf = Vec::new();
        let mut long_help_buf = Vec::new();
        program.write_help(&mut Cursor::new(&mut short_help_buf))?;
        program.write_help(&mut Cursor::new(&mut long_help_buf))?;
        short_help_buf.as_slice().read_to_string(&mut short_help)?;
        long_help_buf.as_slice().read_to_string(&mut long_help)?;
    }
    program.build();
    let matches = program.get_matches();
    for skelly in skeletons {
        if let Some(sub) = matches.subcommand_matches(skelly.0){
            match sub.get_one::<String>("output_dir"){
                Some(loc)=>{
                    let repo = match git2::Repository::clone(skelly.1.as_ref(), loc){
                        Ok(repo)=>{
                            repo
                        }
                        Err(_)=>{
                            eprintln!("libgit2 failed clone falling back to cli");
                            cli_fallback(skelly.1,loc)?;
                            git2::Repository::open(loc)?
                        }
                    };
                    repo.remote_delete("origin")?;
                    return Ok(())
                }
                None=>{
                    println!("failed to provide a path to clone the skeleton directory into");
                    std::process::exit(1);
                }
            }
        }
    }
    for script in scripts {
        if let Some(sub) = matches.subcommand_matches(script.0){
            std::process::exit(std::process::Command::new(script.1).args(sub.get_many::<String>("script_args").map(Iterator::collect).unwrap_or(Vec::new())).spawn()?.wait()?.code().ok_or(Errors::Unknown)?);
        }
    }
    println!("{}", short_help);
    Ok(())
}
#[derive(Debug)]
enum CliError{Io(std::io::Error), NonZero}
impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
fn cli_fallback<S:AsRef<OsStr>, P:AsRef<Path>>(source:S, dest: P)->Result<(),CliError>{

    let mut child = std::process::Command::new("git");
    child
        .arg("clone")
        .arg(source)
        .arg(dest.as_ref());
    if !child.spawn()?.wait()?.success() {
        Err(CliError::NonZero)
    }
    else{
        Ok(())
    }
}
