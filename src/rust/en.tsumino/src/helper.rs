use aidoku::{error::Result, std::String, std::ValueRef};
use alloc::string::ToString;

pub fn get_id(value: ValueRef) -> Result<String> {
	let id = value.as_int().unwrap_or(0) as i32;
	Ok(if id != 0 {
		id.to_string()
	} else {
		value.as_string()?.read()
	})
}
