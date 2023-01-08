use std::io::BufRead;
use std::str;

use quick_xml::events::{Event, BytesEnd, BytesStart, BytesText};
use quick_xml::{Reader, Writer};

use crate::error::Error;
use crate::monster;
use crate::xml;

#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum Parent
{
    Family(String),
    Monster(String),
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct ParentRequirement
{
    pub parent: Parent,
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

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Formula
{
    base: Vec<ParentRequirement>,
    mate: Vec<ParentRequirement>,
    offspring: String,
}

impl Formula
{
    pub fn fromXMLReader<R: BufRead>(opening: &BytesStart,
                                     reader: xml::Reader<R>) ->
        Result<Self, Error>
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
        parser.parse(Some(opening), reader)?;
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

    impl Formula
    {
        fn fromXMLReaderClean<R: BufRead>(reader: xml::Reader<R>) ->
            Result<Self>
        {
            let mut buffer = Vec::new();
            let ev = reader.borrow_mut().read_event_into(&mut buffer)?;
            match ev
            {
                Event::Start(e) => {
                    let f = Self::fromXMLReader(&e, reader)?;
                    Ok(f)
                },
                _ => Err(anyhow::Error::from(rterr!("Fail"))),
            }
        }
    }

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
        let p = Formula::fromXMLReaderClean(
            xml::newReader(quick_xml::Reader::from_str(xml)))?;
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
        let p = Formula::fromXMLReaderClean(xml::newReader(
            quick_xml::Reader::from_str(xml)))?;
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
        let p = Formula::fromXMLReaderClean(xml::newReader(
            quick_xml::Reader::from_str(xml)))?;
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
