#![no_std]
use aidoku::{MangaContentRating, MangaViewer};
use mmrcms_template::{mmrcms, template::MMRCMSSource};

mmrcms! {
	MMRCMSSource {
		base_url: "https://manhwas.men",
		category_parser: |_, _| (MangaContentRating::Nsfw, MangaViewer::Scroll),
		tags_mapper: |idx| {
			String::from(match idx {
				1 => "", // #동거
				2 => "four-sisters", // Four sisters
				3 => "in-laws", // in-laws
				4 => "raws", // raws
				5 => "adult", // #Adult
				6 => "raw", // #Raw
				7 => "drama", // #drama
				8 => "romance", // #romance
				9 => "manhwa", // Manhwa
				10 => "mature", // #Mature
				11 => "sub-english", // Sub English
				12 => "seinen", // Seinen
				13 => "smut", // #Smut
				14 => "harem", // #harem
				15 => "toptoon", // #Toptoon
				16 => "slice-of-life", // Slice of Life
				17 => "full-color", // full color
				18 => "milf", // milf
				19 => "ntr", // #NTR
				20 => "rape", // rape
				21 => "toomics", // #Toomics
				22 => "lezhin", // LEZHIN
				23 => "tomics", // tomics
				24 => "anytoon", // ANYTOON
				25 => "laezhin", // LAEZHIN
				26 => "girlfriend", // #girlfriend
				27 => "collegestudent", // #collegestudent
				28 => "alumni", // #alumni
				29 => "lovetriangle", // #lovetriangle
				30 => "parttimejob", // #parttimejob
				31 => "campus", // Campus
				32 => "school-life", // School Life
				33 => "humiliation", // #humiliation
				34 => "two-girl", // #two girl
				35 => "craving", // #craving
				36 => "aunt", // #aunt
				37 => "housekeeper", // #housekeeper
				38 => "ecchi", // Ecchi
				39 => "comedy", // Comedy
				40 => "noona", // noona
				41 => "sisters", // #Sisters
				42 => "sci-fi", // Sci-Fi
				43 => "supernatural", // Supernatural
				44 => "1", // #1코인할인
				45 => "m", // #레진M
				46 => "hypnosis", // hypnosis
				47 => "assistant", // assistant
				48 => "office", // office
				49 => "special-ability", // special ability
				50 => "awakening", // Awakening
				51 => "romance-drama-mature", // Romance - Drama - Mature
				52 => "comedy-romance-mature", // Comedy - Romance - Mature
				53 => "vanilla", // Vanilla
				54 => "revenge", // Revenge
				55 => "adult-manhwa-mature", // Adult - Manhwa - Mature
				56 => "comedy-romance-drama-harem", // Comedy - Romance - Drama - Harem
				57 => "romance-drama-harem", // Romance - Drama - Harem
				58 => "adult-romance-drama-smut-manhwa-mature", // Adult - Romance - Drama - Smut - Manhwa - Mature
				59 => "psychological", // Psychological
				60 => "fantasy-harem", // Fantasy - Harem
				61 => "adult-romance-manhwa-mature", // Adult - Romance - Manhwa - Mature
				62 => "romance-school-life-drama-harem", // Romance - School Life - Drama - Harem
				63 => "saimin", // saimin
				64 => "romance-drama-harem-mature", // Romance - Drama - Harem - Mature
				65 => "adult-romance-drama-harem", // Adult - Romance - Drama - Harem
				66 => "adult-romance-mature", // Adult - Romance - Mature
				67 => "dance", // Dance
				68 => "seniorjunior", // Senior/Junior
				69 => "vainilla", // vainilla
				70 => "adult-romance-seinen", // Adult - Romance - Seinen
				71 => "adult-romance-drama-seinen-harem-mature", // Adult - Romance - Drama - Seinen - Harem - Mature
				72 => "universidad", // universidad
				73 => "drama-harem-mature", // Drama - Harem - Mature
				74 => "club", // club
				75 => "bondage", // bondage
				76 => "18-adult-smut-manhwa-mature", // 18+ - Adult - Smut - Manhwa - Mature
				77 => "adult-drama-seinen-fantasy-harem", // Adult - Drama - Seinen - Fantasy - Harem
				78 => "sports", // Sports
				79 => "virgin", // virgin
				80 => "pingon-jaja", // pingon jaja
				81 => "romance-drama-fantasy-slice-of-life-raw", // Romance - Drama - Fantasy - Slice of Life - Raw
				82 => "secret-relationship", // Secret Relationship
				83 => "netori", // netori
				84 => "female-friend", // Female Friend
				85 => "neighbour", // Neighbour
				86 => "militar", // militar
				87 => "chantaje", // chantaje
				88 => "action", // Action
				89 => "mystery", // Mystery
				90 => "thriller", // Thriller
				91 => "friend", // #Friend
				92 => "young-woman", // #Young Woman
				93 => "first-experience", // #First Experience
				94 => "married-woman", // #Married Woman
				95 => "wife", // #Wife
				96 => "temptation", // #Temptation
				97 => "sexual-fantasy", // #Sexual Fantasy
				98 => "beauty", // #Beauty
				99 => "vida-universitaria", // vida universitaria
				100 => "bullying", // bullying
				101 => "university", // university
				102 => "big-pennis", // big pennis
				103 => "fantasy", // Fantasy
				104 => "adventure", // Adventure
				105 => "chef", // Chef
				106 => "succubus", // succubus
				107 => "cosplay", // cosplay
				108 => "comedy-romance-school-life-harem", // Comedy - Romance - School Life - Harem
				109 => "murin", // murin
				110 => "magic", // Magic
				111 => "romance-school-life-drama-mature", // Romance - School Life - Drama - Mature
				112 => "comedy-romance-school-life-drama-harem", // Comedy - Romance - School Life - Drama - Harem
				113 => "drama-family", // drama family
				114 => "netorare", // netorare
				115 => "cohabitation-drama-ntr-office", // cohabitation drama NTR office
				116 => "yuri", // Yuri
				117 => "mistery", // mistery
				118 => "4-koma", // 4-Koma
				_ => "",
			})
		},
		..Default::default()
	}
}
