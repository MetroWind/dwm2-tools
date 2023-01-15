//! Data structures and logic about the DMW2 game data.
//!
//! This crate deals with the generic DWM2 game data, not a particular
//! play through. Therefore when it refers to “monster name”, it
//! refers to the name of the monster type (e.g. “Slime”), not the
//! name you made up for your monsters.

#![allow(non_snake_case)]

use std::collections::HashMap;

#[macro_use] extern crate error;
pub mod monster;
pub mod breed;
pub mod xml;
pub mod skill;

use monster::{Monster, Family};
use breed::Formula;
use skill::Skill;
use crate::error::Error;

/// All DWM2 game data. This is the entry point of the whole library.
#[derive(Default, Clone)]
pub struct GameData
{
    /// Data about monsters
    pub monster_data: monster::Info,
    /// All breed formulae
    pub breed_formulae: Vec<Formula>,
    /// All skills
    skills: HashMap<String, Skill>,
}

impl GameData
{
    /// Create a GameData from XML.
    pub fn fromXML(x: &[u8]) -> Result<Self, Error>
    {
        let mut monster_data = monster::Info::default();
        let mut breed_formulae = Vec::new();
        let mut skills = HashMap::new();

        let mut p = xml::Parser::new();
        p.addTagHandler("families", |_, tag| {
            monster_data = monster::Info::fromXML(tag)?;
            Ok(())
        });
        p.addTagHandler("breed", |_, tag| {
            breed_formulae.push(Formula::fromXML(tag)?);
            Ok(())
        });
        p.addTagHandler("skill-data", |_, tag| {
            let s = Skill::fromXML(tag)?;
            skills.insert(s.name.clone(), s);
            Ok(())
        });
        p.parse(x)?;
        drop(p);
        Ok(Self { monster_data, breed_formulae, skills })
    }

    /// Find familiy by name.
    pub fn family(&self, name: &str) -> Option<&Family>
    {
        self.monster_data.families.iter()
            .find(|f| f.name == name)
    }

    /// Find monster by name.
    pub fn monster(&self, name: &str) -> Option<&Monster>
    {
        self.monster_data.monsters.iter()
            .find(|m| m.name == name)
    }

    /// Find skill by name.
    pub fn skill(&self, name: &str) -> Option<&Skill>
    {
        self.skills.get(name)
    }

    /// Find all monsters having a skill natrually.
    pub fn monstersBySkill<'a>(&'a self, skill_name: &'a str) ->
        impl Iterator<Item = &Monster> + 'a
    {
        self.monster_data.monsters.iter().filter(
            move |m| m.abilities.iter().any(|a| a == skill_name))
    }

    /// Find all monsters in a family.
    pub fn monstersInFamily<'a>(&'a self, family: &'a Family) ->
        impl Iterator<Item = &Monster> + 'a
    {
        self.monster_data.monsters.iter().filter(
            |m| family.members.contains(&m.name))
    }

    /// Find all the formulae a monster or a family is used in.
    pub fn usedInFormulae<'a>(&'a self, parent: &'a breed::Parent) ->
        impl Iterator<Item = &Formula> + 'a
    {
        self.breed_formulae.iter().filter(
            move |form| form.base.iter().find(|p| &p.parent == parent).is_some()
                || form.mate.iter().find(|p| &p.parent == parent).is_some())
    }

    /// Find all the formulae that produces a specific monster.
    pub fn breedFromFormulae<'a>(&'a self, offspring: &'a str) ->
        impl Iterator<Item = &Formula> + 'a
    {
        self.breed_formulae.iter()
            .filter(move |form| &form.offspring == offspring)
    }
}

// ========== Tests =================================================>

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;

    #[test]
    fn readXML() -> Result<()>
    {
        let xml = r#"<monster-data>
  <families>
    <family name="slime">
      <monsters>
        <monster name="SpotSlime" in_story="true">
          <spawn-locations>
            <location>
              <map>Oasis Key World</map>
              <description>Near Door Shrine</description>
            </location>
          </spawn-locations>
          <abilities>
            <ability>CallHelp</ability>
            <ability>LushLicks</ability>
            <ability>Imitate</ability>
          </abilities>
          <growth agl="17" int="8" maxlvl="35" atk="17" mp="1" exp="10" hp="17" def="4"/>
        </monster>
        <monster name="BoxSlime" in_story="true">
          <spawn-locations>
            <location>
              <map>Sky Key World</map>
              <description>Heaven Helm Cave Floor 1</description>
            </location>
          </spawn-locations>
          <abilities>
            <ability>Blaze</ability>
            <ability>Upper</ability>
            <ability>Ramming</ability>
          </abilities>
          <growth agl="14" int="13" maxlvl="50" atk="14" mp="10" exp="11" hp="11" def="19"/>
        </monster>
      </monsters>
    </family>
  </families>
  <breeds>
    <breed target="Zoma">
      <base>
        <breed-requirement monster="DracoLord1"/>
        <breed-requirement monster="DracoLord2"/>
      </base>
      <mate>
        <breed-requirement monster="Sidoh"/>
      </mate>
    </breed>
  </breeds>
  <skills-data>
    <skill-data name="VacuSlash">
      <skill-requirement lvl="11" hp="77" mp="34" atk="66" def="0" agl="0" int="76"/>
      <combine-from>
        <skill>WindBeast</skill>
        <skill>ChargeUp</skill>
      </combine-from>
    </skill-data>
    <skill-data name="Vacuum">
      <skill-requirement lvl="19" hp="112" mp="0" atk="114" def="0" agl="132" int="0"/>
      <precursor>WindBeast</precursor>
    </skill-data>
  </skills-data>
</monster-data>
"#;
        let data = GameData::fromXML(xml.as_bytes())?;
        assert_eq!(data.breed_formulae.len(), 1);
        assert_eq!(data.monster_data.monsters.len(), 2);
        assert_eq!(data.monster_data.families.len(), 1);
        assert_eq!(data.monster_data.families[0].members.len(), 2);
        assert_eq!(data.skills.len(), 2);
        assert_eq!(data.skills["VacuSlash"].combine_from,
                   vec![String::from("WindBeast"), String::from("ChargeUp")]);
        assert_eq!(data.skills["VacuSlash"].upgrade_from, None);
        assert_eq!(data.skills["Vacuum"].upgrade_from,
                   Some(String::from("WindBeast")));
        Ok(())
    }
}
