use std::io::BufRead;
use std::str;
use std::cell::RefCell;
use std::rc::Rc;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::xml::{self, getTagAttr};
use crate::error::Error;

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
    /// Parse the location tag.
    ///
    /// A location tag looks like this:
    ///
    /// ```xml
    /// <location>
    ///   <map>Oasis Key World</map>
    ///   <description>Near Door Shrine</description>
    /// </location>
    /// ```
    ///
    /// When the reader finds a location tag and calls
    /// `MapLocation::fromXMLReader()`, it already consumes the tag.
    /// So the parser in fromXMLReader() will only see the closing
    /// tag. This is invalid XML. So here the `opening` argument will
    /// be the opening tag, so the parser sees it.
    fn fromXMLReader<R: BufRead>(opening: &BytesStart, reader: xml::Reader<R>)
                                 -> Result<Self, Error>
    {
        let mut map = String::new();
        let mut desc = String::new();
        let mut p = xml::Parser::new();
        p.addTextHandler("map", |_, text| {
            map = str::from_utf8(text.clone().into_inner().as_ref()).map_err(
                |_| rterr!("Failed to decode map"))?.to_owned();
            Ok(())
        });
        p.addTextHandler("description", |_, text| {
            desc = str::from_utf8(text.clone().into_inner().as_ref()).map_err(
                |_| rterr!("Failed to decode description"))?.to_owned();
            Ok(())
        });
        p.parse(Some(opening), reader)?;
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
            || rterr!("Agility value not found"))?;
        let inteligence: u8 =  xml::getTagAttr(tag, "int")?.ok_or_else(
            || rterr!("Inteligence value not found"))?;
        let max_level: u8 =  xml::getTagAttr(tag, "maxlvl")?.ok_or_else(
            || rterr!("Max level value not found"))?;
        let attack: u8 =  xml::getTagAttr(tag, "atk")?.ok_or_else(
            || rterr!("Attack value not found"))?;
        let defense: u8 =  xml::getTagAttr(tag, "def")?.ok_or_else(
            || rterr!("Defense value not found"))?;
        let mp: u8 =  xml::getTagAttr(tag, "mp")?.ok_or_else(
            || rterr!("MP value not found"))?;
        let exp: u8 =  xml::getTagAttr(tag, "exp")?.ok_or_else(
            || rterr!("EXP value not found"))?;
        let hp: u8 =  xml::getTagAttr(tag, "hp")?.ok_or_else(
            || rterr!("HP value not found"))?;
        Ok(Self {
            agility, inteligence, max_level, attack, defense, mp, exp, hp,
        })
    }
}


/// The info of a monster
pub struct Monster
{
    pub name: String,
    pub in_story: bool,
    pub locations: Vec<MapLocation>,
    pub abilities: Vec<String>,
    pub growth: Growth,
}

impl Monster
{
    fn fromXMLReader<R: BufRead>(opening: &BytesStart, reader: xml::Reader<R>)
                                 -> Result<Self, Error>
    {
        let mut name = String::new();
        let mut in_story = false;
        let mut locations = Vec::new();
        let mut abilities = Vec::new();
        let mut growth = Growth::default();

        let mut p = xml::Parser::new();
        p.addBeginHandler("monster", |_, tag| {
            name = xml::getTagAttr(tag, "name")?.ok_or_else(
                || rterr!("Monster name value not found"))?;
            in_story = xml::getTagAttr(tag, "in_story")?.ok_or_else(
                || rterr!("In_story value not found"))?;
            Ok(())
        });
        p.addBeginHandler("location", |_, tag| {
            locations.push(MapLocation::fromXMLReader(tag, reader.clone())?);
            Ok(())
        });
        p.addTextHandler("ability", |_, text| {
            abilities.push(str::from_utf8(text.clone().into_inner().as_ref())
                           .map_err(|_| rterr!("Failed to read ability"))?
                           .to_owned());
            Ok(())
        });
        p.addBeginHandler("growth", |_, tag| {
            growth = Growth::fromXMLTag(tag)?;
            Ok(())
        });
        p.parse(Some(opening), reader.clone())?;
        drop(p);

        Ok(Self { name, in_story, locations, abilities, growth })
    }
}

/// A family of monsters
#[derive(Default)]
pub struct Family
{
    /// Name of the family in lower case.
    pub name: String,
    /// List of name of monsters in this family.
    pub members: Vec<String>,
}

/// All monster data
#[derive(Default)]
pub struct Info
{
    pub monsters: Vec<Monster>,
    pub families: Vec<Family>,
}

impl Info
{
    pub fn fromXMLReader<R: BufRead>(opening: &BytesStart, reader: xml::Reader<R>)
                                 -> Result<Self, Error>
    {
        let family = Rc::new(RefCell::new(Family::default()));
        let mut monsters = Vec::new();
        let mut families = Vec::new();

        let mut p = xml::Parser::new();
        p.addBeginHandler("family", |_, tag| {
            family.borrow_mut().name = getTagAttr(tag, "name")?
                .ok_or_else(|| rterr!("Family name not found"))?;
            Ok(())
        });
        p.addEndHandler("family", |_, _| {
            families.push(family.take());
            *family.borrow_mut() = Family::default();
            Ok(())
        });
        p.addBeginHandler("monster", |_, tag| {
            let monster = Monster::fromXMLReader(tag, reader.clone())?;
            family.borrow_mut().members.push(monster.name.clone());
            monsters.push(monster);
            Ok(())
        });

        p.parse(Some(opening), reader.clone())?;
        drop(p);
        Ok(Self { monsters, families })
    }
}
