#![deny(clippy::all)]

use dicom_object::open_file;
use dicom_dump::dump_file;
use napi::JsError;

pub mod storescp;
pub mod storescu;

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}

#[napi]
pub fn dicom_dump(file: String) -> Result<(), JsError> {
  let obj_result = open_file(file).unwrap();
  let _ = dump_file(&obj_result);
  Ok(())
}