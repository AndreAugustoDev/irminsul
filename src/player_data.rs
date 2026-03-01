use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
pub use auto_artifactarium::Achievement;
pub use auto_artifactarium::r#gen::protos::{AvatarInfo, Item};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::good::{self, fake_uninitialized_4th_line};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExportSettings {
    pub include_characters: bool,
    pub include_artifacts: bool,
    pub include_weapons: bool,
    pub include_materials: bool,
    pub fake_initialize_4th_line: bool,

    pub min_character_level: u32,
    pub min_character_ascension: u32,
    pub min_character_constellation: u32,

    pub min_artifact_level: u32,
    pub min_artifact_rarity: u32,

    pub min_weapon_level: u32,
    pub min_weapon_refinement: u32,
    pub min_weapon_ascension: u32,
    pub min_weapon_rarity: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnimeGameData {
    #[serde(rename = "textMap")]
    pub text_map: HashMap<String, String>,
}

impl AnimeGameData {
    pub fn new(path: &Path) -> Result<Self> {
        let json = fs::read_to_string(path)?;
        let data = serde_json::from_str(&json)?;
        Ok(data)
    }

    pub fn get_character(&self, id: u32) -> Result<&String> {
        self.text_map
            .get(&id.to_string())
            .ok_or_else(|| anyhow::anyhow!("Character not found"))
    }

    pub fn get_artifact(&self, id: u32) -> Result<&String> {
        self.text_map
            .get(&id.to_string())
            .ok_or_else(|| anyhow::anyhow!("Artifact not found"))
    }

    pub fn get_weapon(&self, id: u32) -> Result<&String> {
        self.text_map
            .get(&id.to_string())
            .ok_or_else(|| anyhow::anyhow!("Weapon not found"))
    }

    pub fn get_material(&self, id: u32) -> Result<&String> {
        self.text_map
            .get(&id.to_string())
            .ok_or_else(|| anyhow::anyhow!("Material not found"))
    }

    pub fn get_property(&self, id: u32) -> Result<&String> {
        self.text_map
            .get(&id.to_string())
            .ok_or_else(|| anyhow::anyhow!("Property not found"))
    }

    pub fn get_affix(&self, id: u32) -> Result<&String> {
        self.text_map
            .get(&id.to_string())
            .ok_or_else(|| anyhow::anyhow!("Affix not found"))
    }
}

pub struct PlayerData {
    game_data: AnimeGameData,
    achievements: Vec<Achievement>,
    characters: Vec<AvatarInfo>,
    items: Vec<Item>,

    character_equip_guid_map: HashMap<u64, u32>,
}

impl PlayerData {
    pub fn new(game_data: AnimeGameData) -> Self {
        Self {
            game_data,
            achievements: Vec::new(),
            characters: Vec::new(),
            items: Vec::new(),
            character_equip_guid_map: HashMap::new(),
        }
    }

    pub fn process_achievements(&mut self, achievements: &[Achievement]) {
        self.achievements = achievements.into();
    }

    pub fn process_characters(&mut self, avatars: &[AvatarInfo]) {
        self.character_equip_guid_map.clear();
        for avatar in avatars {
            for guid in &avatar.equip_guid_list {
                self.character_equip_guid_map
                    .insert(*guid, avatar.avatar_id);
            }
        }
        self.characters = avatars.into();
    }

    pub fn process_items(&mut self, items: &[Item]) {
        self.items = items.into();
    }

    pub fn export_genshin_optimizer(&self, settings: &ExportSettings) -> Result<String> {
        let mut good = good::Good {
            format: "GOOD".to_string(),
            version: 3,
            source: "Irminsul".to_string(),
            characters: Vec::new(),
            artifacts: Vec::new(),
            weapons: Vec::new(),
            materials: HashMap::new(),
        };

        if settings.include_characters {
            good.characters = self.export_genshin_optimizer_characters(settings);
        }

        if settings.include_artifacts {
            good.artifacts = if settings.fake_initialize_4th_line {
                let artifacts = self.export_genshin_optimizer_artifacts(settings);
                fake_uninitialized_4th_line(artifacts)
            } else {
                self.export_genshin_optimizer_artifacts(settings)
            };
        }

        if settings.include_weapons {
            good.weapons = self.export_genshin_optimizer_weapons(settings);
        }

        if settings.include_materials {
            good.materials = self.export_genshin_optimizer_materials();
        }

        let json = serde_json::to_string(&good)?;
        tracing::trace!("{json}");
        Ok(json)
    }

    pub fn export_genshin_optimizer_characters(
        &self,
        settings: &ExportSettings,
    ) -> Vec<good::Character> {
        self.characters
            .iter()
            .filter_map(|character| {
                if character.avatar_type != 1 {
                    return None;
                }

                let name = self.game_data.get_character(character.avatar_id).ok()?;
                let level = character.prop_map.get(&4001).map(|prop| prop.val as u32)?;
                let ascension = character.prop_map.get(&1002).map(|prop| prop.val as u32)?;
                let constellation = character.talent_id_list.len() as u32;

                let auto = 1;
                let skill = 1;
                let burst = 1;

                if level < settings.min_character_level
                    || ascension < settings.min_character_ascension
                    || constellation < settings.min_character_constellation
                {
                    return None;
                }

                Some(good::Character {
                    key: good::to_good_key(name),
                    level,
                    constellation,
                    ascension,
                    talent: good::TalentLevel { auto, skill, burst },
                })
            })
            .collect()
    }

    pub fn export_genshin_optimizer_artifacts(
        &self,
        settings: &ExportSettings,
    ) -> Vec<good::Artifact> {
        self.items
            .iter()
            .filter_map(|item| {
                if !item.has_equip() {
                    return None;
                }
                let equip = item.equip();
                let location = self
                    .character_equip_guid_map
                    .get(&item.guid)
                    .and_then(|id| {
                        self.game_data
                            .get_character(*id)
                            .ok()
                            .map(|location| good::to_good_key(location).to_string())
                    })
                    .unwrap_or_default();

                if !equip.has_reliquary() {
                    return None;
                }
                let artifact_data = self.game_data.get_artifact(item.item_id).ok()?;
                let artifact = equip.reliquary();
                let mut substats: IndexMap<String, (f32, f32)> = IndexMap::new();
                for substat_id in artifact.append_prop_id_list.iter() {
                    let Some(substat) = self.game_data.get_affix(*substat_id).ok() else {
                        continue;
                    };
                    let entry = substats.entry(substat.to_string()).or_insert((0., 0.));
                    entry.0 += 0.;
                }
                let substats = substats
                    .into_iter()
                    .map(|(property, (value, initial_value))| good::Substat {
                        key: property.to_string(),
                        value,
                        initial_value,
                    })
                    .collect();
                let unactivated_substats = artifact
                    .unactivated_prop_id_list
                    .iter()
                    .filter_map(|substat_id| {
                        let substat = self.game_data.get_affix(*substat_id).ok()?;
                        Some(good::Substat {
                            key: substat.to_string(),
                            value: 0.,
                            initial_value: 0.,
                        })
                    })
                    .collect();
                let total_rolls = artifact.append_prop_id_list.len() as u32;

                let level = artifact.level - 1;
                let rarity = 5;
                let astral_mark = artifact.starred;
                let elixer_crafted = !artifact.elixer_choices.is_empty();
                let main_stat_key = self
                    .game_data
                    .get_property(artifact.main_prop_id)
                    .ok()?
                    .to_string();

                if level < settings.min_artifact_level || rarity < settings.min_artifact_rarity {
                    return None;
                }

                Some(good::Artifact {
                    set_key: good::to_good_key(artifact_data),
                    slot_key: artifact_data.to_string(),
                    level,
                    rarity,
                    main_stat_key,
                    location,
                    lock: equip.is_locked,
                    substats,
                    total_rolls,
                    astral_mark,
                    elixer_crafted,
                    unactivated_substats,
                })
            })
            .collect()
    }

    pub fn export_genshin_optimizer_weapons(&self, settings: &ExportSettings) -> Vec<good::Weapon> {
        self.items
            .iter()
            .filter_map(|item| {
                if !item.has_equip() {
                    return None;
                }
                let equip = item.equip();
                let location = self
                    .character_equip_guid_map
                    .get(&item.guid)
                    .and_then(|id| {
                        self.game_data
                            .get_character(*id)
                            .ok()
                            .map(|location| good::to_good_key(location).to_string())
                    })
                    .unwrap_or_default();
                if !equip.has_weapon() {
                    return None;
                }
                let weapon_data = self.game_data.get_weapon(item.item_id).ok()?;
                let weapon = equip.weapon();
                let refinement = weapon
                    .affix_map
                    .values()
                    .cloned()
                    .next()
                    .unwrap_or_default()
                    + 1;

                let level = weapon.level;
                let ascension = weapon.promote_level;

                if level < settings.min_weapon_level
                    || refinement < settings.min_weapon_refinement
                    || ascension < settings.min_weapon_ascension
                    || 5 < settings.min_weapon_rarity
                {
                    return None;
                }

                Some(good::Weapon {
                    key: good::to_good_key(weapon_data),
                    level,
                    ascension,
                    refinement,
                    location,
                    lock: equip.is_locked,
                })
            })
            .collect()
    }

    pub fn export_genshin_optimizer_materials(&self) -> HashMap<String, u32> {
        self.items
            .iter()
            .filter_map(|item| {
                if !item.has_material() {
                    return None;
                }
                let material = item.material();
                let name = self.game_data.get_material(item.item_id).ok()?;

                Some((good::to_good_key(name), material.count))
            })
            .collect()
    }
}
