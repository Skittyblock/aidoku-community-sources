use aidoku::std::{String, Vec};

pub fn i32_to_string(mut integer: i32) -> String {
	if integer == 0 {
		return String::from("0");
	}
	let mut string = String::with_capacity(11);
	let pos = if integer < 0 {
		string.insert(0, '-');
		1
	} else {
		0
	};
	while integer != 0 {
		let mut digit = integer % 10;
		if pos == 1 {
			digit *= -1;
		}
		string.insert(pos, char::from_u32((digit as u32) + ('0' as u32)).unwrap());
		integer /= 10;
	}
	string
}

pub fn urlencode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789abcdef".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if curr.is_ascii_lowercase() || curr.is_ascii_uppercase() || curr.is_ascii_digit() {
			result.push(curr);
		} else {
			result.push(b'%');
			result.push(hex[curr as usize >> 4]);
			result.push(hex[curr as usize & 15]);
		}
	}

	String::from_utf8(result).unwrap_or_default()
}

pub fn lang_encoder(lang: String) -> String {
	let lang = match lang.to_lowercase().as_str() {
		"abkhaz" => String::from("ab"),
		"afar" => String::from("aa"),
		"afrikaans" => String::from("af"),
		"akan" => String::from("ak"),
		"albanian" => String::from("sq"),
		"amharic" => String::from("am"),
		"arabic" => String::from("ar"),
		"aragonese" => String::from("an"),
		"armenian" => String::from("hy"),
		"assamese" => String::from("as"),
		"avaric" => String::from("av"),
		"avestan" => String::from("ae"),
		"aymara" => String::from("ay"),
		"azerbaijani" => String::from("az"),
		"bambara" => String::from("bm"),
		"bashkir" => String::from("ba"),
		"basque" => String::from("eu"),
		"belarusian" => String::from("be"),
		"bengali; bangla" => String::from("bn"),
		"bihari" => String::from("bh"),
		"bislama" => String::from("bi"),
		"bosnian" => String::from("bs"),
		"breton" => String::from("br"),
		"bulgarian" => String::from("bg"),
		"burmese" => String::from("my"),
		"catalan; valencian" => String::from("ca"),
		"chamorro" => String::from("ch"),
		"chechen" => String::from("ce"),
		"chichewa; chewa; nyanja" => String::from("ny"),
		"chinese" => String::from("zh"),
		"chuvash" => String::from("cv"),
		"cornish" => String::from("kw"),
		"corsican" => String::from("co"),
		"cree" => String::from("cr"),
		"croatian" => String::from("hr"),
		"czech" => String::from("cs"),
		"danish" => String::from("da"),
		"divehi; dhivehi; maldivian;" => String::from("dv"),
		"dutch" => String::from("nl"),
		"dzongkha" => String::from("dz"),
		"english" => String::from("en"),
		"esperanto" => String::from("eo"),
		"estonian" => String::from("et"),
		"ewe" => String::from("ee"),
		"faroese" => String::from("fo"),
		"fijian" => String::from("fj"),
		"finnish" => String::from("fi"),
		"french" => String::from("fr"),
		"fula; fulah; pulaar; pular" => String::from("ff"),
		"galician" => String::from("gl"),
		"georgian" => String::from("ka"),
		"german" => String::from("de"),
		"greek, modern" => String::from("el"),
		"guaranã\u{AD}" => String::from("gn"),
		"gujarati" => String::from("gu"),
		"haitian; haitian creole" => String::from("ht"),
		"hausa" => String::from("ha"),
		"hebrew (modern)" => String::from("he"),
		"herero" => String::from("hz"),
		"hindi" => String::from("hi"),
		"hiri motu" => String::from("ho"),
		"hungarian" => String::from("hu"),
		"interlingua" => String::from("ia"),
		"indonesian" => String::from("id"),
		"interlingue" => String::from("ie"),
		"irish" => String::from("ga"),
		"igbo" => String::from("ig"),
		"inupiaq" => String::from("ik"),
		"ido" => String::from("io"),
		"icelandic" => String::from("is"),
		"italian" => String::from("it"),
		"inuktitut" => String::from("iu"),
		"japanese" => String::from("ja"),
		"javanese" => String::from("jv"),
		"kalaallisut, greenlandic" => String::from("kl"),
		"kannada" => String::from("kn"),
		"kanuri" => String::from("kr"),
		"kashmiri" => String::from("ks"),
		"kazakh" => String::from("kk"),
		"khmer" => String::from("km"),
		"kikuyu, gikuyu" => String::from("ki"),
		"kinyarwanda" => String::from("rw"),
		"kyrgyz" => String::from("ky"),
		"komi" => String::from("kv"),
		"kongo" => String::from("kg"),
		"korean" => String::from("ko"),
		"kurdish" => String::from("ku"),
		"kwanyama, kuanyama" => String::from("kj"),
		"latin" => String::from("la"),
		"luxembourgish, letzeburgesch" => String::from("lb"),
		"ganda" => String::from("lg"),
		"limburgish, limburgan, limburger" => String::from("li"),
		"lingala" => String::from("ln"),
		"lao" => String::from("lo"),
		"lithuanian" => String::from("lt"),
		"luba-katanga" => String::from("lu"),
		"latvian" => String::from("lv"),
		"manx" => String::from("gv"),
		"macedonian" => String::from("mk"),
		"malagasy" => String::from("mg"),
		"malay" => String::from("ms"),
		"malayalam" => String::from("ml"),
		"maltese" => String::from("mt"),
		"mäori" => String::from("mi"),
		"marathi (maräá¹\u{AD}hä«)" => String::from("mr"),
		"marshallese" => String::from("mh"),
		"mongolian" => String::from("mn"),
		"nauru" => String::from("na"),
		"navajo, navaho" => String::from("nv"),
		"norwegian bokmã¥l" => String::from("nb"),
		"north ndebele" => String::from("nd"),
		"nepali" => String::from("ne"),
		"ndonga" => String::from("ng"),
		"norwegian nynorsk" => String::from("nn"),
		"norwegian" => String::from("no"),
		"nuosu" => String::from("ii"),
		"south ndebele" => String::from("nr"),
		"occitan" => String::from("oc"),
		"ojibwe, ojibwa" => String::from("oj"),
		"old church slavonic, church slavic, church slavonic, old bulgarian, old slavonic" => {
			String::from("cu")
		}
		"oromo" => String::from("om"),
		"oriya" => String::from("or"),
		"ossetian, ossetic" => String::from("os"),
		"panjabi, punjabi" => String::from("pa"),
		"päli" => String::from("pi"),
		"persian (farsi)" => String::from("fa"),
		"polish" => String::from("pl"),
		"pashto, pushto" => String::from("ps"),
		"portuguese" => String::from("pt"),
		"quechua" => String::from("qu"),
		"romansh" => String::from("rm"),
		"kirundi" => String::from("rn"),
		"romanian, [])" => String::from("ro"),
		"russian" => String::from("ru"),
		"sanskrit (saá¹ská¹›ta)" => String::from("sa"),
		"sardinian" => String::from("sc"),
		"sindhi" => String::from("sd"),
		"northern sami" => String::from("se"),
		"samoan" => String::from("sm"),
		"sango" => String::from("sg"),
		"serbian" => String::from("sr"),
		"scottish gaelic; gaelic" => String::from("gd"),
		"shona" => String::from("sn"),
		"sinhala, sinhalese" => String::from("si"),
		"slovak" => String::from("sk"),
		"slovene" => String::from("sl"),
		"somali" => String::from("so"),
		"southern sotho" => String::from("st"),
		"spanish; castilian" => String::from("es"),
		"sundanese" => String::from("su"),
		"swahili" => String::from("sw"),
		"swati" => String::from("ss"),
		"swedish" => String::from("sv"),
		"tamil" => String::from("ta"),
		"telugu" => String::from("te"),
		"tajik" => String::from("tg"),
		"thai" => String::from("th"),
		"tigrinya" => String::from("ti"),
		"tibetan standard, tibetan, central" => String::from("bo"),
		"turkmen" => String::from("tk"),
		"tagalog" => String::from("tl"),
		"tswana" => String::from("tn"),
		"tonga (tonga islands)" => String::from("to"),
		"turkish" => String::from("tr"),
		"tsonga" => String::from("ts"),
		"tatar" => String::from("tt"),
		"twi" => String::from("tw"),
		"tahitian" => String::from("ty"),
		"uyghur, uighur" => String::from("ug"),
		"ukrainian" => String::from("uk"),
		"urdu" => String::from("ur"),
		"uzbek" => String::from("uz"),
		"venda" => String::from("ve"),
		"vietnamese" => String::from("vi"),
		"volapã¼k" => String::from("vo"),
		"walloon" => String::from("wa"),
		"welsh" => String::from("cy"),
		"wolof" => String::from("wo"),
		"western frisian" => String::from("fy"),
		"xhosa" => String::from("xh"),
		"yiddish" => String::from("yi"),
		"yoruba" => String::from("yo"),
		"zhuang, chuang" => String::from("za"),
		"zulu" => String::from("zu"),
		_ => String::new(),
	};
	lang
}
