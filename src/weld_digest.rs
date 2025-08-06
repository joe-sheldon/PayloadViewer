use std::collections::HashMap;
use std::fs;
use bevy::asset::Asset;
use bevy::math::Vec3;
use bevy::prelude::TypePath;
use serde::Deserialize;

#[derive(Deserialize, Asset, TypePath, Default, Clone)]
pub struct Joint {
    pub joint_number: String,
    pub joint_design: String,
    pub center: Option<Vec3>,
    pub members: Vec<String>
}

#[derive(Deserialize, Asset, TypePath, Default, Clone)]
pub struct Component {
    pub design_id: String,
    pub part_number: String,
    pub description: String,
    pub geom_path: Option<String>,
    pub loc: Option<Vec3>,
    pub faces: Option<Vec<[f32;3]>>,
}

#[derive(Deserialize, Asset, TypePath, Clone, Default)]
pub struct PayloadDigest {
    pub name: String,
    pub rev: String,
    pub joints: HashMap<String, Joint>,
    pub components: HashMap<String, Component>,
}

pub fn read_payload_digest(directory: String) -> Result<PayloadDigest, String> {
    let digest_file_path = directory + "/" + "payload_digest.json";
    let digest_file_content: String = fs::read_to_string(digest_file_path).unwrap();
    
    let payload_digest: PayloadDigest = serde_json::from_str(&digest_file_content)
        .expect("JSON was not well-formatted");
    
    Ok(payload_digest)
}