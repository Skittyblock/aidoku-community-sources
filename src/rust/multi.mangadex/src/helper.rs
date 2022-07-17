use aidoku::std::{current_date, net::Request, ValueRef};

#[link(wasm_import_module = "net")]
extern "C" {
	#[link_name = "send"]
	fn request_send(rd: i32);
	#[link_name = "json"]
	fn request_json(rd: i32) -> i32;
	#[link_name = "close"]
	fn request_close(rd: i32);
	#[link_name = "get_data_size"]
	fn request_get_data_size(rd: i32) -> i32;

	#[link_name = "get_status_code"]
	fn request_get_status_code(rd: i32) -> i32;
}

/// Helper for automatically retrying a rate-limited request.
///
/// This works on the assumption that Aidoku rate-limited requests
/// don't have a body and returns the 429 status code.
///
/// Hardcoded constants:
/// * This retries after 1 second
/// * Max retries is 5
/// * Assumes that status code 429 means we're being ratelimited
pub trait SendRatelimited {
	fn send_rl(&self);
	fn json_rl(self) -> ValueRef;
}

impl SendRatelimited for Request {
	fn send_rl(&self) {
		unsafe {
			request_send(self.0);
		}

		let mut attempts = 1;
		let mut size = unsafe { request_get_data_size(self.0) };
		while unsafe { request_get_status_code(self.0) } == 429 && size < 0 && attempts <= 5 {
			let now = current_date();
			while current_date() - now < 1.0 {}
			unsafe {
				request_send(self.0);
				size = request_get_data_size(self.0);
				attempts += 1;
			}
		}
	}

	/// Get the data as JSON
	fn json_rl(self) -> ValueRef {
		self.send_rl();
		let rid = unsafe { request_json(self.0) };
		unsafe { request_close(self.0) };
		ValueRef::new(rid)
	}
}
