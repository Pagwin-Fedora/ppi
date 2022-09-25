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
    skeletons:HashMap<String, (String, String)>,
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
    let mut prog_copy = program.clone();
    program.build();
    let matches = program.get_matches();
    for (skelly_name, (skelly_src, skelly_branch)) in skeletons {
        if let Some(sub) = matches.subcommand_matches(&skelly_name){
            match sub.get_one::<String>("output_dir"){
                Some(loc)=>{

                    //cloning the repo
                    let repo = match git2::Repository::clone(skelly_src.as_ref(), loc){
                        Ok(repo)=>{
                            repo
                        }
                        Err(_)=>{
                            eprintln!("libgit2 failed clone falling back to cli");
                            cli_fallback(skelly_src,loc)?;
                            git2::Repository::open(loc)?
                        }
                    };
                    #[cfg(debug_assertions)]
                    {
                        eprintln!("repo cloned");
                        std::io::stdin().read_line(&mut String::new())?;
                    }

                    eprintln!("checking out appropriate branch");
                    handle_process(std::process::Command::new("git")
                        .args(["checkout", skelly_branch.as_str()]))?;
                    // let branch = repo.branches(Some(git2::BranchType::Remote))?.find(|branch|{
                    //     if let Ok(branch) = branch{
                    //         if let Ok(Some(name)) = branch.0.name(){
                    //             name == format!("origin/{}",skelly_branch)
                    //         }else{false}
                    //     }else{false}
                    // }).ok_or(git2::Error::from_str(format!("no branch with name {}",skelly_branch).as_str()))??;

                    // match repo.set_head(branch.0.into_reference().name().ok_or(git2::Error::from_str("the branch with a name somehow has a reference without a name"))?){
                    //     Err(e)=>{
                    //         panic!("{}",e.message());
                    //     }
                    //     _=>{}
                    // }

                    //#[cfg(debug_assertions)]
                    //{
                    //    eprintln!("head set");
                    //    std::io::stdin().read_line(&mut String::new())?;
                    //}

                    repo.remote_delete("origin")?;

                    #[cfg(debug_assertions)]
                    {
                        eprintln!("origin deleted");
                        std::io::stdin().read_line(&mut String::new())?;
                    }



                    let mut walk = repo.revwalk()?;
                    walk.set_sorting(git2::Sort::TIME)?;
                    walk.push_head()?;
                    //let head = walk.next().ok_or(git2::Error::from_str("no head"))?.map(|oid|repo.find_commit(oid))??;
                    let oldest_commit = repo.find_commit(walk.last().ok_or(git2::Error::from_str("No Oldest commit"))??)?;
                    //
                    #[cfg(debug_assertions)]
                    {
                        eprintln!("oldest commit found");
                        std::io::stdin().read_line(&mut String::new())?;
                    }
                    let pwd = repo.path().parent().expect("very bad cloning into the root dir happening");
                    eprintln!("rebasing skeleton's commits down into single commit {:?}", pwd);
                    // I give up git2 documentation/api is just too bad for me to do this with it
                    

                    handle_process(std::process::Command::new("git")
                        .current_dir(pwd)
                        .args(["reset", "--mixed" , oldest_commit.id().as_bytes().iter().map(|byte|format!("{:x}",byte)).collect::<String>().chars().take(7).collect::<String>().as_str()]))?;

                    handle_process(std::process::Command::new("git")
                        .current_dir(pwd)
                        .args(["add", "--all"]))?;

                    handle_process(std::process::Command::new("git")
                        .current_dir(pwd)
                        .args(["commit", "--amend",  "-am", format!("initialized from {} skeleton", skelly_name).as_str()]))?;
                    std::process::exit(0)
                }
                None=>{
                    eprintln!("failed to provide a path to clone the skeleton directory into");
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
    prog_copy.print_help()?;
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
    handle_process(&mut child)
}

fn handle_process<'proc,P:Into<&'proc mut std::process::Command>>(cmd:P) -> Result<(),CliError>{
    if !cmd.into().status()?.success() {
        Err(CliError::NonZero)
    }
    else{
        Ok(())
    }
}
