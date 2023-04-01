use aidoku::{
	prelude::format,
	std::{html::Node, String, Vec},
	MangaStatus,
};

// MARK: Mappings
pub fn get_tag_id(genre: i64) -> String {
	String::from(match genre {
		1 => "1",     // R18
		2 => "113",   // Action
		3 => "114",   // Adventure
		4 => "115",   // Comedy
		5 => "116",   // Demon
		6 => "117",   // Drama
		7 => "118",   // Ecchi
		8 => "119",   // Fantasy
		9 => "120",   // Game
		10 => "121",  // Gender Bender
		11 => "122",  // Harem
		12 => "123",  // Historical
		13 => "124",  // Horror
		14 => "125",  // Martial Arts
		15 => "126",  // Mecha
		16 => "127",  // Military
		17 => "128",  // Music
		18 => "129",  // Mystery
		19 => "130",  // Parody
		20 => "131",  // Psychological
		21 => "132",  // Romance
		22 => "133",  // School Life
		23 => "134",  // Sci-Fi
		24 => "137",  // Slice of  Life
		25 => "138",  // Sports
		26 => "139",  // Supernatural
		27 => "140",  // Vampire
		28 => "141",  // Violence
		29 => "142",  // Tragedy
		30 => "143",  // Adult Life
		31 => "144",  // Isekai
		32 => "145",  // College
		33 => "146",  // Manhua
		34 => "147",  // Manhwa
		35 => "148",  // Full Color
		36 => "149",  // 4-koma
		37 => "150",  // No Text
		38 => "151",  // Yuri
		39 => "152",  // Hints
		40 => "153",  // Lỗi: không tìm thấy trai
		41 => "156",  // Glasses
		42 => "157",  // Blushing
		43 => "158",  // Body Swap
		44 => "159",  // Reversal
		45 => "160",  // Het
		46 => "161",  // Excuse me WTF?
		47 => "162",  // Pay for Gay
		48 => "163",  // FBI Warning!!
		49 => "164",  // Moe Paradise
		50 => "165",  // Science Babies
		51 => "166",  // Student x Teacher
		52 => "167",  // Siscon
		53 => "168",  // Mahou Shoujo
		54 => "169",  // Idol
		55 => "170",  // Tomboy
		56 => "171",  // Yankee
		57 => "172",  // Maid
		58 => "173",  // Monster Girl
		59 => "174",  // Office Lady
		60 => "175",  // Animal Ears
		61 => "176",  // Bisexual
		62 => "177",  // Tsundere
		63 => "178",  // Yandere
		64 => "179",  // Age Gap
		65 => "180",  // Co-worker
		66 => "181",  // Roommates
		67 => "182",  // Childhood Friends
		68 => "183",  // Love Triangle
		69 => "184",  // Threesome
		70 => "185",  // Polyamory
		71 => "186",  // Twins
		72 => "187",  // Incest
		73 => "188",  // Marriage
		74 => "189",  // Christmas
		75 => "190",  // Halloween
		76 => "191",  // New Year's
		77 => "192",  // Valentine
		78 => "193",  // Thất Tịch
		79 => "194",  // Birthday
		80 => "195",  // Big Breasts
		81 => "196",  // Butts
		82 => "197",  // Loli
		83 => "198",  // Netorare
		84 => "199",  // BDSM
		85 => "200",  // Toys
		86 => "201",  // Futanari
		87 => "202",  // Tentacles
		88 => "203",  // Rape
		89 => "204",  // Massage
		90 => "205",  // Masturbation
		91 => "206",  // Guro
		92 => "208",  // Dark Skin
		93 => "209",  // Anal
		94 => "210",  // Boob Sex
		95 => "211",  // Ahegao
		96 => "212",  // Pocky Game
		97 => "214",  // Anime
		98 => "215",  // School Girl
		99 => "216",  // Light Novel
		100 => "218", // Oneshot
		101 => "219", // Drunk
		102 => "220", // Creepy
		103 => "222", // Official
		104 => "223", // Spin-off
		105 => "226", // Bath
		106 => "227", // Mangaka
		107 => "228", // Yuri Crush
		108 => "229", // NSFW
		109 => "230", // Yaoi
		110 => "231", // Subtext
		111 => "232", // Food
		112 => "234", // Mermaid
		113 => "235", // Kuudere
		114 => "236", // Drugs
		115 => "237", // Tailsex
		116 => "238", // Zombies
		117 => "239", // Childification
		118 => "240", // Prostitution
		119 => "241", // Bullying
		120 => "242", // Amnesia
		121 => "243", // Time Travel
		122 => "244", // Ghost
		123 => "245", // Exhibitionism
		124 => "246", // Gyaru
		125 => "249", // Sleeping
		126 => "251", // Sequel
		127 => "252", // Disability
		128 => "254", // Hypnosis
		129 => "255", // Autobiographical
		130 => "256", // Feet
		131 => "257", // Player
		132 => "258", // Delinquent
		133 => "260", // Lactation
		134 => "261", // Orgy
		135 => "262", // Alien
		136 => "263", // Swimsuits
		137 => "264", // Robot
		138 => "265", // Deity
		139 => "266", // Stalking
		140 => "267", // Cheating
		141 => "268", // Moderate amounts of sex
		142 => "269", // Lots of sex
		143 => "270", // Biting
		144 => "271", // Clones
		145 => "272", // Prequel
		146 => "273", // Post-Apocalyptic
		147 => "274", // Philosophical
		148 => "276", // Omegaverse
		149 => "277", // Amputee
		150 => "278", // Watersports
		151 => "279", // Wholesome
		152 => "280", // Blackmail
		153 => "281", // Height Gap
		154 => "282", // Idiot Couple
		155 => "283", // Assassin
		156 => "284", // Transgender
		157 => "285", // Biographical
		158 => "286", // Introspective
		159 => "287", // Ninja
		160 => "288", // Cross-dressing
		161 => "289", // Beach
		162 => "290", // Depressing as fuck
		163 => "291", // Space
		164 => "292", // Hardcore
		165 => "293", // Witch
		166 => "296", // Insane Amounts of Sex
		167 => "298", // Angel
		168 => "299", // Spanking
		169 => "300", // Abuse
		170 => "301", // Miko
		171 => "302", // Non-moe art
		172 => "303", // Furry
		173 => "304", // BHTT
		174 => "305", // Web Novel
		175 => "306", // >
		_ => "",
	})
}

pub fn listing_map(listing: String) -> String {
	String::from(match listing.as_str() {
		"Random" => "random",
		_ => "",
	})
}

pub fn status_map(status: i64) -> MangaStatus {
	match status {
		1 => MangaStatus::Unknown, // "Chưa ra mắt" => Not released
		2 => MangaStatus::Completed,
		3 => MangaStatus::Unknown, // "Sắp ra mắt" => Upcoming
		4 => MangaStatus::Ongoing,
		5 => MangaStatus::Cancelled, // "Ngừng dịch" => source not translating it anymomre
		6 => MangaStatus::Hiatus,
		7 => MangaStatus::Cancelled, // "Ngừng xuất bản" => No more publications
		_ => MangaStatus::Unknown,
	}
}

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

pub fn extract_f32_from_string(title: String, text: String) -> f32 {
	text.replace(&title, " ")
		.chars()
		.filter(|a| (*a >= '0' && *a <= '9') || *a == ' ' || *a == '.')
		.collect::<String>()
		.split(' ')
		.collect::<Vec<&str>>()
		.into_iter()
		.map(|a| a.parse::<f32>().unwrap_or(0.0))
		.find(|a| *a > 0.0)
		.unwrap_or(0.0)
}

pub fn get_search_url(base_url: String, query: String, tag: String, page: i32) -> String {
	if !query.is_empty() {
		format!("{base_url}/search?query={query}&page={page}")
	} else if !tag.is_empty() {
		return format!("{base_url}/searchType?type=tag&id={tag}&page={page}");
	} else {
		return format!("{base_url}/lastest2?page={page}");
	}
}

pub fn text_with_newlines(html: String) -> Option<String> {
	if !String::from(html.trim()).is_empty() {
		if let Ok(node) = Node::new_fragment(
			html.replace("<br>", "{{ .LINEBREAK }}")
				.replace("</p><p>", "{{ .LINEBREAK }}")
				.as_bytes(),
		) {
			Some(node.text().read().replace("{{ .LINEBREAK }}", "\n"))
		} else {
			None
		}
	} else {
		None
	}
}
