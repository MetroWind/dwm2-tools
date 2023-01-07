use serde::Deserialize;

/// A location in a map
#[derive(Deserialize)]
pub struct MapLocation
{
    /// Name of the map. Example: Oasis Key World.
    pub map: String,
    /// More detailed location within the map.
    pub desc: Option<String>,
}

/// Monster growth data
#[derive(Deserialize)]
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

/// The info of a monster
#[derive(Deserialize)]
pub struct Monster
{
    pub name: String,
    pub in_story: bool,
    pub locations: Vec<MapLocation>,
    pub abilities: Vec<String>,
    pub growth: Growth,
}

/// A family of monsters
#[derive(Deserialize)]
pub struct Family
{
    /// Name of the family in lower case.
    pub name: String,
    /// List of name of monsters in this family.
    pub members: Vec<Monster>,
}
