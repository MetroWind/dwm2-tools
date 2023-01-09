//! Data structures and logic about the DMW2 breed formulae

use std::str;

use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::xml;

/// A parent in a breed formula.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum Parent
{
    /// Any monster in this family could be the parent. Value
    /// indicates the name of the family.
    Family(String),
    /// The parent is this monster, whose name is the value.
    Monster(String),
}

/// A parent in a breed formula, with some extra requirements.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct ParentRequirement
{
    /// The parent in the breed formula
    pub parent: Parent,
    /// The minimal +level of this parent. Usually this is just 0. But
    /// for some formulae (e.g. Slime + Slime = KingSlime) this is
    /// non-zero.
    pub min_plus: u8,
}

impl ParentRequirement
{
    fn fromXMLTag(e: &BytesStart) -> Result<Self, Error>
    {
        let min_plus = if let Some(attr) = e.try_get_attribute("min_plus")
            .map_err(|_| rterr!("Failed to get attribute 'min_plus'."))?
        {
            str::from_utf8(attr.value.as_ref()).map_err(
                |_| rterr!("Failed to decode min +level in XML"))?.parse()
                .map_err(|_| rterr!("Invalid min +level in XML"))?
        }
        else
        {
            0
        };

        if let Some(attr) = e.try_get_attribute("monster").map_err(
            |_| rterr!("Failed to get attribute 'monster'."))?
        {
            let monster = String::from_utf8(attr.value.into_owned()).map_err(
                |_| rterr!("Failed to decode monster name in XML"))?;
            Ok(Self {
                parent: Parent::Monster(monster),
                min_plus
            })
        }
        else if let Some(attr) = e.try_get_attribute("family").map_err(
            |_| rterr!("Failed to get attribute 'family'."))?
        {
            let family = String::from_utf8(attr.value.into_owned()).map_err(
                |_| rterr!("Failed to decode family name in XML"))?;
            Ok(Self {
                parent: Parent::Family(family),
                min_plus
            })
        }
        else
        {
            Err(rterr!("Invalid parent definition in XML"))
        }
    }
}

/// A breed formula
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Formula
{
    /// The base monster in the formula.
    pub base: Vec<ParentRequirement>,
    /// The mate monster in the formula.
    pub mate: Vec<ParentRequirement>,
    /// The name of the offspring monster.
    pub offspring: String,
}

impl Formula
{
    pub fn fromXML(x: &[u8]) -> Result<Self, Error>
    {
        let mut offspring = String::new();
        let mut base = Vec::new();
        let mut mate = Vec::new();
        let mut parser = xml::Parser::new();

        parser.addBeginHandler("breed", |_, e: &BytesStart| {
            if let Some(attr) = e.try_get_attribute("target").map_err(
                |_| rterr!("Failed to get attribute 'target'."))?
            {
                offspring = String::from_utf8(attr.value.into_owned()).map_err(
                    |_| rterr!("Failed to decode breed target name in XML"))?;
                Ok(())
            }
            else
            {
                Err(rterr!("Found breed formula without target"))
            }
        });

        parser.addBeginHandler("breed-requirement", |path, e: &BytesStart| {
            if let Some(tag) = path.get(path.len() - 2)
            {
                match tag.as_str()
                {
                    "base" => base.push(ParentRequirement::fromXMLTag(e)?),
                    "mate" => mate.push(ParentRequirement::fromXMLTag(e)?),
                    _ => return Err(rterr!("Invalid tag '{}'", tag)),
                }
            }
            else
            {
                return Err(rterr!("Invalid breed requirement"));
            }
            Ok(())
        });
        parser.parse(x)?;
        drop(parser);
        let result = Self { base, mate, offspring };
        Ok(result)
    }
}

// ========== Tests =================================================>

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;

    #[test]
    fn deserializeFormula() -> Result<()>
    {
        let xml = r#"<breed target="KingSlime">
      <base>
        <breed-requirement monster="SpotKing"/>
      </base>
      <mate>
        <breed-requirement monster="DeadNoble"/>
        <breed-requirement monster="Divinegon"/>
        <breed-requirement monster="Gigantes"/>
      </mate>
    </breed>"#;
        let p = Formula::fromXML(xml.as_bytes())?;
        assert_eq!(
            p,
            Formula {
                base: vec![ParentRequirement{ parent: Parent::Monster("SpotKing".to_owned()), min_plus: 0 }],
                mate: vec![ParentRequirement{ parent: Parent::Monster("DeadNoble".to_owned()), min_plus: 0 },
                           ParentRequirement{ parent: Parent::Monster("Divinegon".to_owned()), min_plus: 0 },
                           ParentRequirement{ parent: Parent::Monster("Gigantes".to_owned()), min_plus: 0},],
                offspring: "KingSlime".to_owned(),
            });

        let xml = r#"<breed target="Octogon">
      <base>
        <breed-requirement monster="Octoreach"/>
      </base>
      <mate>
        <breed-requirement monster="Octoreach" min_plus="4"/>
      </mate>
    </breed>"#;
        let p = Formula::fromXML(xml.as_bytes())?;
        assert_eq!(
            p,
            Formula {
                base: vec![ParentRequirement{ parent: Parent::Monster("Octoreach".to_owned()), min_plus: 0 }],
                mate: vec![ParentRequirement{ parent: Parent::Monster("Octoreach".to_owned()), min_plus: 4 }],
                offspring: "Octogon".to_owned(),
            });

        let xml = r#"<breed target="Moray">
      <base>
        <breed-requirement family="water"/>
      </base>
      <mate>
        <breed-requirement family="dragon"/>
      </mate>
    </breed>"#;
        let p = Formula::fromXML(xml.as_bytes())?;
        assert_eq!(
            p,
            Formula {
                base: vec![ParentRequirement{ parent: Parent::Family("water".to_owned()), min_plus: 0 }],
                mate: vec![ParentRequirement{ parent: Parent::Family("dragon".to_owned()), min_plus: 0 }],
                offspring: "Moray".to_owned(),
            });

        Ok(())
    }
}
