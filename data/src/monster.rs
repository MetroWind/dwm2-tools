//! Data structures and logic about the DMW2 monsters and families

use std::cell::RefCell;
use std::rc::Rc;

use quick_xml::events::BytesStart;

use error::Error;
use crate::xml::{self, getTagAttr};

/// A location in a map
pub struct MapLocation
{
    /// Name of the map. Example: Oasis Key World.
    pub map: String,
    /// More detailed location within the map.
    pub desc: Option<String>,
}

impl MapLocation
{
    /// Parse the location tag. A location tag looks like this:
    ///
    /// ```xml
    /// <location>
    ///   <map>Oasis Key World</map>
    ///   <description>Near Door Shrine</description>
    /// </location>
    /// ```
    fn fromXML(x: &[u8]) -> Result<Self, Error>
    {
        let mut map = String::new();
        let mut desc = String::new();
        let mut p = xml::Parser::new();
        p.addTextHandler("map", |_, text| {
            map = text.to_owned();
            Ok(())
        });
        p.addTextHandler("description", |_, text| {
            desc = text.to_owned();
            Ok(())
        });
        p.parse(x)?;
        drop(p);
        Ok(Self {
            map,
            desc: if desc.is_empty() { None } else { Some(desc) }
        })
    }
}

/// Monster growth data
#[derive(Default)]
pub struct Growth
{
    pub agility: u8,
    pub inteligence: u8,
    pub max_level: u8,
    pub attack: u8,
    pub defense: u8,
    pub mp: u8,
    pub exp: u8,
    pub hp: u8,
}

impl Growth
{
    fn fromXMLTag(tag: &BytesStart) -> Result<Self, Error>
    {
        let agility: u8 =  xml::getTagAttr(tag, "agl")?.ok_or_else(
            || xmlerr!("Agility value not found"))?;
        let inteligence: u8 =  xml::getTagAttr(tag, "int")?.ok_or_else(
            || xmlerr!("Inteligence value not found"))?;
        let max_level: u8 =  xml::getTagAttr(tag, "maxlvl")?.ok_or_else(
            || xmlerr!("Max level value not found"))?;
        let attack: u8 =  xml::getTagAttr(tag, "atk")?.ok_or_else(
            || xmlerr!("Attack value not found"))?;
        let defense: u8 =  xml::getTagAttr(tag, "def")?.ok_or_else(
            || xmlerr!("Defense value not found"))?;
        let mp: u8 =  xml::getTagAttr(tag, "mp")?.ok_or_else(
            || xmlerr!("MP value not found"))?;
        let exp: u8 =  xml::getTagAttr(tag, "exp")?.ok_or_else(
            || xmlerr!("EXP value not found"))?;
        let hp: u8 =  xml::getTagAttr(tag, "hp")?.ok_or_else(
            || xmlerr!("HP value not found"))?;
        Ok(Self {
            agility, inteligence, max_level, attack, defense, mp, exp, hp,
        })
    }
}


/// The info of a monster
pub struct Monster
{
    /// Name of the monster
    pub name: String,
    /// Whether the monster can be find in the story maps
    pub in_story: bool,
    /// Spawn location of the monster. Only monsters with `in_story ==
    /// true` has this.
    pub locations: Vec<MapLocation>,
    /// Natural abilities of the monster
    pub abilities: Vec<String>,
    /// Monster growth data
    pub growth: Growth,
}

impl Monster
{
    fn fromXML(x: &[u8]) -> Result<Self, Error>
    {
        let mut name = String::new();
        let mut in_story = false;
        let mut locations = Vec::new();
        let mut abilities = Vec::new();
        let mut growth = Growth::default();

        let mut p = xml::Parser::new();
        p.addBeginHandler("monster", |_, tag| {
            name = xml::getTagAttr(tag, "name")?.ok_or_else(
                || xmlerr!("Monster name value not found"))?;
            in_story = xml::getTagAttr(tag, "in_story")?.ok_or_else(
                || xmlerr!("In_story value not found"))?;
            Ok(())
        });
        p.addTagHandler("location", |_, tag| {
            locations.push(MapLocation::fromXML(tag)?);
            Ok(())
        });
        p.addTextHandler("ability", |_, text| {
            abilities.push(text.to_owned());
            Ok(())
        });
        p.addBeginHandler("growth", |_, tag| {
            growth = Growth::fromXMLTag(tag)?;
            Ok(())
        });
        p.parse(x)?;
        drop(p);

        Ok(Self { name, in_story, locations, abilities, growth })
    }
}

/// A family of monsters
#[derive(Default)]
pub struct Family
{
    /// Name of the family in *lower case*.
    pub name: String,
    /// List of name of monsters in this family.
    pub members: Vec<String>,
}

/// Data about a set of monsters and families.
#[derive(Default)]
pub struct Info
{
    /// A list of monsters
    pub monsters: Vec<Monster>,
    /// A list of families
    pub families: Vec<Family>,
}

impl Info
{
    pub fn fromXML(x: &[u8]) -> Result<Self, Error>
    {
        let family = Rc::new(RefCell::new(Family::default()));
        let mut monsters = Vec::new();
        let mut families = Vec::new();

        let mut p = xml::Parser::new();
        p.addBeginHandler("family", |_, tag| {
            family.borrow_mut().name = getTagAttr(tag, "name")?
                .ok_or_else(|| xmlerr!("Family name not found"))?;
            Ok(())
        });
        p.addEndHandler("family", |_, _| {
            families.push(family.take());
            *family.borrow_mut() = Family::default();
            Ok(())
        });
        p.addTagHandler("monster", |_, tag| {
            let monster = Monster::fromXML(tag)?;
            family.borrow_mut().members.push(monster.name.clone());
            monsters.push(monster);
            Ok(())
        });

        p.parse(x)?;
        drop(p);
        Ok(Self { monsters, families })
    }
}
