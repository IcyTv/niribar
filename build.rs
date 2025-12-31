use std::path::Path;

use heck::ToPascalCase;

const LUCIDE_ICONS: &[&str] = &[
	"moon",
	"log-out",
	"power",
	"rotate-ccw",
	"clock",
	"calendar",
	"network",
	"wifi",
	"wifi-off",
	"bluetooth",
	"bluetooth-connected",
	"bluetooth-off",
	"bluetooth-searching",
	"arrow-up-down",
	"file-terminal",
	"circle-x",
	"volume",
	"volume-1",
	"volume-2",
	"volume-x",
	"volume-off",
	"search",
	"pipette",
	"settings",
	"camera",
	"circle",
	"bell",
	"mic",
	"mic-off",
	"folder",
	"folder-down",
	"folder-code",
	"briefcase-business",
	"terminal",
	"shuffle",
	"skip-back",
	"play",
	"pause",
	"skip-forward",
	"repeat",
	"headphones",
	"headset",
	"chevron-left",
	"chevron-right",
	"cloud",
	"cloud-drizzle",
	"cloud-fog",
	"cloud-hail",
	"cloud-lightning",
	"cloud-moon",
	"cloud-moon-rain",
	"cloud-rain",
	"cloud-rain-wind",
	"cloud-snow",
	"cloud-sun",
	"cloud-sun-rain",
	"cloud-alert",
	"cloudy",
	"snowflake",
	"sun",
	"sun-snow",
	"wind",
	"haze",
	"moon-star",
	"thermometer",
	"droplets",
];

const BRAND_ICONS: &[&str] = &["discord", "spotify", "firefox", "nixos"];

fn main() {
	println!("cargo:rerun-if-env-changed=LUCIDE_ICONS_PATH");
	println!("cargo:rerun-if-env-changed=SIMPLE_ICONS_PATH");

	let out_dir = std::env::var("OUT_DIR").unwrap();
	let dest_path = Path::new(&out_dir);

	if !dest_path.exists() {
		let _ = std::fs::create_dir_all(dest_path).ok();
	}

	let mut xml = String::from(
		"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<gresources>\n  <gresource prefix=\"/de/icytv/niribar/icons\">\n",
	);

	let mut variants = String::new();
	let mut matches = String::new();

	let mut process_icon = |name: &str, base_path: &str| {
		let src_dir = Path::new(base_path);
		let filename = format!("{}.svg", name);
		let src_path = src_dir.join(&filename);

		if !src_path.exists() {
			panic!("Icon '{}' not found at {:?}", name, src_path);
		}

		let enum_name = name.to_pascal_case();
		let resource_alias = format!("scalable/actions/{}-symbolic.svg", name); // Force symbolic for UI consistency
		let icon_name = format!("{}-symbolic", name);

		xml.push_str(&format!(
			"    <file alias=\"{}\">{}</file>\n",
			resource_alias,
			src_path.to_str().expect("Invalid path")
		));

		variants.push_str(&format!("    {},\n", enum_name));
		matches.push_str(&format!("            Self::{} => \"{}\",\n", enum_name, icon_name));
	};

	let lucide_path = std::env::var("LUCIDE_ICONS_PATH").expect(
		"LUCIDE_ICONS_PATH environment variable not set. Please set it to the path of the lucide icons repository.",
	);
	for icon_name in LUCIDE_ICONS {
		process_icon(icon_name, &lucide_path);
	}

	let simple_icons_path = std::env::var("SIMPLE_ICONS_PATH").expect(
		"SIMPLE_ICONS_PATH environment variable not set. Please set it to the path of the simple-icons repository.",
	);
	for icon_name in BRAND_ICONS {
		process_icon(icon_name, &simple_icons_path);
	}

	xml.push_str("  </gresource>\n</gresources>\n");

	let xml_path = dest_path.join("lucide.xml");
	std::fs::write(&xml_path, xml).unwrap();

	let paths: &[&Path] = &[];
	glib_build_tools::compile_resources(paths, xml_path.to_str().unwrap(), "lucide.gresource");

	let code = format!(
		"#[derive(Debug, Clone, Copy)]\npub enum Icon {{\n{}\n}}\nimpl Icon {{\n    pub fn name(&self) -> &'static \
		 str {{\n        match self {{\n{}\n        }}\n    }}\n}}",
		variants, matches
	);
	std::fs::write(dest_path.join("icons.rs"), code).unwrap();

	println!("cargo:rerun-if-changed=**/*.blp");

	glib_build_tools::compile_resources(&[".", "./assets/"], "assets/resources.xml", "assets.gresource");
}
