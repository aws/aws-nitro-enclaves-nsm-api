// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

extern crate cbindgen;

use cbindgen::Config;
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let crate_env = env::var("CARGO_MANIFEST_DIR").unwrap();
    let crate_path = Path::new(&crate_env);
    let config = Config::from_root_or_default(crate_path);
    let out_path = output_dir();
    cbindgen::Builder::new()
        .with_crate(crate_path.to_str().unwrap())
        .with_config(config)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("nsm.h"));
}

/// Sets target to target/$PROFILE/
fn output_dir() -> PathBuf {
    env::var("OUT_DIR")
        .map(PathBuf::from)
        .map(|dir| dir.ancestors().nth(3).unwrap().to_owned())
        .unwrap()
}
