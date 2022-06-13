#![no_std]
use aidoku::{MangaContentRating, MangaViewer};
use mmrcms_template::{mmrcms, template::MMRCMSSource};

mmrcms! {
	MMRCMSSource {
		base_url: "https://zahard.xyz",
		category_parser: |_, categories| {
			let mut nsfw = MangaContentRating::Safe;
			for category in categories {
				match category.as_str() {
					"Adult" | "Smut" | "Mature" | "18+" | "Hentai" => {
						nsfw = MangaContentRating::Nsfw
					}
					"Ecchi" | "16+" => {
						nsfw = match nsfw {
							MangaContentRating::Nsfw => MangaContentRating::Nsfw,
							_ => MangaContentRating::Suggestive,
						}
					}
					_ => continue,
				}
			}
			(nsfw, MangaViewer::Scroll)
		},
		tags_mapper: |idx| {
			String::from(match idx {
				1 => "(", // (
				2 => "sdgsdg", // sdgsdg
				3 => "action", // Action
				4 => "fantasy", // Fantasy
				5 => "manhwa", // Manhwa
				6 => "martial-arts", // Martial Arts
				7 => "shounen", // Shounen
				8 => "webtoon", // Webtoon
				9 => "drama", // Drama
				10 => "isekai", // Isekai
				11 => "romance", // Romance
				12 => "webtoons", // Webtoons
				13 => "sekai", // sekai
				14 => "shoujo", // Shoujo
				15 => "returner", // Returner
				16 => "sub", // Sub
				17 => "comedy", // Comedy
				18 => "contract-relationship", // Contract Relationship
				19 => "contracts", // Contracts
				20 => "contractual-relationship", // Contractual Relationship
				21 => "from-being-haters-to-lovers", // From Being Haters To Lovers
				22 => "mature", // Mature
				23 => "office-workers", // Office Workers
				24 => "revenge", // Revenge
				25 => "tragic-past", // Tragic Past
				26 => "adventure", // Adventure
				27 => "apocalypse", // apocalypse
				28 => "supernatural", // Supernatural
				29 => "magic", // Magic
				30 => "time-travel", // Time Travel
				31 => "monsters", // Monsters
				32 => "psychological", // Psychological
				33 => "return", // Return
				34 => "tragedy", // Tragedy
				35 => "school", // School
				36 => "slice-of-life", // Slice of Life
				37 => "game", // Game
				38 => "rebirth", // Rebirth
				39 => "virtual-reality", // Virtual Reality
				40 => "manhua", // Manhua
				41 => "overpowered", // Overpowered
				42 => "sci-fi", // Sci-Fi
				43 => "video-game", // Video Game
				44 => "cultivation", // Cultivation
				45 => "murim", // Murim
				46 => "system", // System
				47 => "adaptation", // Adaptation
				48 => "historical", // Historical
				49 => "teen", // Teen
				50 => "demon", // Demon
				51 => "reincarnation", // Reincarnation
				52 => "harem", // Harem
				53 => "adult", // Adult
				54 => "monster", // Monster
				55 => "pokemon", // Pokemon
				56 => "battle-of-wits", // Battle of Wits
				57 => "mystery", // Mystery
				58 => "pretentious", // Pretentious
				59 => "superhero", // Superhero
				60 => "thriller", // Thriller
				61 => "gore", // Gore
				62 => "survival", // Survival
				63 => "ceo", // CEO
				64 => "urban", // Urban
				65 => "full-color", // Full Color
				66 => "korean", // Korean
				67 => "comic", // Comic
				68 => "josei", // Josei
				69 => "modern-romance", // Modern Romance
				70 => "hot-blood", // Hot blood
				71 => "school-life", // School Life
				72 => "chinese", // Chinese
				73 => "xuanhuan", // Xuanhuan
				74 => "doll", // doll
				75 => "female-protagonist", // Female Protagonist
				76 => "knight", // knight
				77 => "protect-me-knight", // protect me knight
				78 => "life", // Life
				79 => "video-games", // Video Games
				80 => "coming-soon", // Coming Soon
				81 => "actionm", // Actionm
				82 => "fantasym", // Fantasym
				83 => "murimm", // Murimm
				84 => "returner-system", // Returner System
				85 => "villain", // Villain
				86 => "fusion", // Fusion
				87 => "dungeons", // Dungeons
				88 => "manga", // Manga
				89 => "post-apocalyptic", // Post-Apocalyptic
				90 => "zombies", // Zombies
				91 => "military", // Military
				92 => "animals", // Animals
				93 => "story", // Story
				94 => "otherworld", // Otherworld
				95 => "ecchi", // Ecchi
				96 => "gaming", // Gaming
				_ => "",
			})
		},
		..Default::default()
	}
}
