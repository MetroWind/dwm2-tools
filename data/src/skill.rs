//! Data structures and logic for skills

use quick_xml::events::BytesStart;
use serde::Serialize;

use error::Error;
use crate::xml;

/// Stat requirement for a skill to be learnt
#[derive(Clone, Serialize, Default)]
pub struct Requirements
{
    pub agility: u32,
    pub intelligence: u32,
    pub level: u32,
    pub attack: u32,
    pub defense: u32,
    pub mp: u32,
    pub hp: u32,
}

impl Requirements
{
    fn fromXMLTag(tag: &BytesStart) -> Result<Self, Error>
    {
        let agility: u32 =  xml::getTagAttr(tag, "agl")?.ok_or_else(
            || xmlerr!("Agility value not found"))?;
        let intelligence: u32 =  xml::getTagAttr(tag, "int")?.ok_or_else(
            || xmlerr!("Inteligence value not found"))?;
        let level: u32 =  xml::getTagAttr(tag, "lvl")?.ok_or_else(
            || xmlerr!("Level value not found"))?;
        let attack: u32 =  xml::getTagAttr(tag, "atk")?.ok_or_else(
            || xmlerr!("Attack value not found"))?;
        let defense: u32 =  xml::getTagAttr(tag, "def")?.ok_or_else(
            || xmlerr!("Defense value not found"))?;
        let mp: u32 =  xml::getTagAttr(tag, "mp")?.ok_or_else(
            || xmlerr!("MP value not found"))?;
        let hp: u32 =  xml::getTagAttr(tag, "hp")?.ok_or_else(
            || xmlerr!("HP value not found"))?;
        Ok(Self {
            agility, intelligence, level, attack, defense, mp, hp,
        })
    }
}

/// A skill
#[derive(Default, Clone, Serialize)]
pub struct Skill
{
    /// Name of the skill
    pub name: String,
    /// Stats requirements of the skill
    pub requirements: Requirements,
    pub upgrade_from: Option<String>,
    pub upgrade_to: Option<String>,
    pub combine_from: Vec<String>,
}

impl Skill
{
    /// This will not populate the `upgrade_to` field.
    pub fn fromXML(x: &[u8]) -> Result<Self, Error>
    {
        let mut skill = Self::default();
        let combine_from = String::from("combine-from");
        let mut p = xml::Parser::new();
        p.addBeginHandler("skill-data", |_, tag| {
            skill.name = xml::getTagAttr(tag, "name")?.ok_or_else(
                || xmlerr!("Skill name value not found"))?;
            Ok(())
        });
        p.addTextHandler("precursor", |_, text| {
            skill.upgrade_from = Some(text.to_owned());
            Ok(())
        });
        p.addTextHandler("skill", |xml_path, text| {
            if xml_path.get(xml_path.len() - 2) == Some(&combine_from)
            {
                skill.combine_from.push(text.to_owned());
            }
            Ok(())
        });
        p.addBeginHandler("skill-requirements", |_, tag| {
            skill.requirements = Requirements::fromXMLTag(tag)?;
            Ok(())
        });

        p.parse(x)?;
        drop(p);
        Ok(skill)
    }
}
