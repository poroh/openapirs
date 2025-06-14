// SPDX-License-Identifier: MIT
//
// Parsing of openapi spec
//
extern crate openapirs;

use openapirs::compile;
use openapirs::model;
use std::io::Read;

#[allow(dead_code)]
#[derive(Debug)]
enum Error {
    Io(String, std::io::Error),
    SerdeYml(String, serde_yaml::Error),
    ParameterNeeded,
    Compile(String, String),
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage:");
        println!(" {} <openapi yaml file>", args[0]);
        return Err(Error::ParameterNeeded);
    }
    let fname = args[1].clone();
    let mut file =
        std::fs::File::open(args[1].clone()).map_err(|err| Error::Io(fname.clone(), err))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|err| Error::Io(fname.clone(), err))?;
    let spec: openapirs::schema::Description =
        serde_yaml::from_str(&contents).map_err(|err| Error::SerdeYml(fname.clone(), err))?;
    let compiled = compile::compile(&spec)
        .map_err(|err| Error::Compile(args[1].clone(), format!("{err:?}")))?;
    let result = model::build(&compiled);
    println!("Result:");
    for (name, v) in result.iter() {
        println!("{name:?}");
        println!("{v:?}");
    }
    Ok(())
}
