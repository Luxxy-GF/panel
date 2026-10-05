#![allow(clippy::default_constructed_unit_structs)]
#![allow(unused_imports)]

use shared::extensions::{ConstructedExtension, distr::MetadataToml};
use std::sync::Arc;

pub fn list() -> Vec<ConstructedExtension> {
    vec![
        ConstructedExtension {
            metadata_toml: MetadataToml {
                package_name: "xyz.luxxy.incus".to_string(),
                name: "Incus native instances".to_string(),
                panel_version: semver::VersionReq::parse(">=1.2.4, <1.3.0").unwrap(),
                license_text: None,
            },
            package_name: "xyz.luxxy.incus",
            description: "Native Incus system containers, virtual machines and interactive terminals",
            authors: &["Luxxy"],
            version: semver::Version::parse("0.1.0").unwrap(),
            extension: Arc::new(xyz_luxxy_incus::ExtensionStruct::default()),
        },
    ]
}
