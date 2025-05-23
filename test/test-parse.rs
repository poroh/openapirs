// SPDX-License-Identifier: MIT
//
// Parsing of openapi spec
//

use std::io::Read;

extern crate openapirs;

#[allow(dead_code)]
#[derive(Debug)]
enum Error {
    Io(std::io::Error),
    SerdeYml(serde_yaml::Error),
    ParameterNeeded,
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
    //println!("{spec:?}");
    if let Some(paths) = spec.paths {
        for (path, item) in paths.into_iter() {
            println!();
            println!("{path:?}: {item:?}");
        }
    }
    if let Some(components) = spec.components.and_then(|v| v.schemas) {
        for (component, item) in components.into_iter() {
            println!();
            println!("{component:?}: {item:?}");
        }
    }
    Ok(())
}
