// SPDX-License-Identifier: MIT
//
// Parsing of openapi spec
//

use std::io::Read;

extern crate openapirs;

#[allow(dead_code)]
#[derive(Debug)]
enum Error {
    IoError(std::io::Error),
    SerdeYmlError(serde_yaml::Error),
}

fn main() -> Result<(), Error> {
    let mut file = std::fs::File::open("openapi.yaml").map_err(Error::IoError)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(Error::IoError)?;
    let spec: openapirs::schema::Description =
        serde_yaml::from_str(&contents).map_err(Error::SerdeYmlError)?;
    //println!("{spec:?}");
    if let Some(paths) = spec.components.and_then(|v| v.schemas) {
        for (path, item) in paths.into_iter() {
            println!("");
            println!("{path:?}: {item:?}");
        }
    }
    Ok(())
}
