use super::url::{Charset, Url};
use aidoku::{
	error::Result,
	prelude::{format, println},
	std::defaults::defaults_get,
};
use base64::{engine::general_purpose, Engine as _};

pub fn change_charset() {
	let charset = defaults_get("isTC")
		.and_then(|value| {
			if value.as_bool()? {
				return Ok(Charset::Traditional);
			}

			Ok(Charset::Simplified)
		})
		.unwrap_or_default();

	Url::Charset { charset }.get().send();
}

pub fn sign_in() -> Result<()> {
	let captcha = defaults_get("captcha")?.as_string()?.read();

	let is_wrong_captcha_format = captcha.parse::<u16>().is_err() || captcha.chars().count() != 4;
	if is_wrong_captcha_format {
		let sign_in_page = Url::SignInPage.get().html()?;

		let captcha_img_path = sign_in_page.select("img#verifyImg").attr("src").read();
		let captcha_img = Url::Abs {
			path: &captcha_img_path,
		}
		.get()
		.data();
		let base64_img = general_purpose::STANDARD_NO_PAD.encode(captcha_img);

		return Ok(println!("{}", base64_img));
	}

	let username = defaults_get("username")?.as_string()?.read();
	let password = defaults_get("password")?.as_string()?.read();
	let sign_in_data = format!(
		"username={}&password={}&vfycode={}&type=login",
		username, password, captcha
	);

	let response_json = Url::SignIn.post(sign_in_data).json()?;
	let reponse_obj = response_json.as_object()?;
	let info = reponse_obj.get("info").as_string()?;

	Ok(println!("{}", info))
}
