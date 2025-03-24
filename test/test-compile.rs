// SPDX-License-Identifier: MIT
//
// Parsing of openapi spec
//
extern crate openapirs;

use openapirs::compile;
use std::io::Read;

#[allow(dead_code)]
#[derive(Debug)]
enum Error {
    Io(std::io::Error),
    SerdeYml(serde_yaml::Error),
    ParameterNeeded,
    Compile(compile::Error),
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage:");
        println!(" {} <openapi yaml file>", args[0]);
        return Err(Error::ParameterNeeded);
    }
    let mut file = std::fs::File::open(args[1].clone()).map_err(Error::Io)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(Error::Io)?;
    let spec: openapirs::schema::Description =
        serde_yaml::from_str(&contents).map_err(Error::SerdeYml)?;
    let result = compile::compile(&spec).map_err(Error::Compile)?;
    for v in result.operations.iter() {
        println!("{v:?}");
    }
    Ok(())
}
