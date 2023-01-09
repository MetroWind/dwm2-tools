//! Data structures and logic about the DMW2 game data.
//!
//! This crate deals with the generic DWM2 game data, not a particular
//! play through. Therefore when it refers to “monster name”, it
//! refers to the name of the monster type (e.g. “Slime”), not the
//! name you made up for your monsters.

#![allow(non_snake_case)]

#[macro_use]
pub mod error;
pub mod monster;
pub mod breed;
pub mod xml;

use crate::error::Error;

/// All DWM2 game data. This is the entry point of the whole library.
pub struct GameData
{
    /// Data about monsters
    pub monster_data: monster::Info,
    /// All breed formulae
    pub breed_formulae: Vec<breed::Formula>,
}

impl GameData
{
    /// Create a GameData from XML.
    pub fn fromXML(x: &[u8]) -> Result<Self, Error>
    {
        let mut monster_data = monster::Info::default();
        let mut breed_formulae = Vec::new();

        let mut p = xml::Parser::new();
        p.addTagHandler("families", |_, tag| {
            monster_data = monster::Info::fromXML(tag)?;
            Ok(())
        });
        p.addTagHandler("breed", |_, tag| {
            breed_formulae.push(breed::Formula::fromXML(tag)?);
            Ok(())
        });
        p.parse(x)?;
        drop(p);
        Ok(Self { monster_data, breed_formulae })
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
</monster-data>
"#;
        let data = GameData::fromXML(xml.as_bytes())?;
        assert_eq!(data.breed_formulae.len(), 1);
        assert_eq!(data.monster_data.monsters.len(), 2);
        assert_eq!(data.monster_data.families.len(), 1);
        assert_eq!(data.monster_data.families[0].members.len(), 2);
        Ok(())
    }
}
