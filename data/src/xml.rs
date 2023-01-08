use std::io::{Read, BufRead};
use std::str;
use std::collections::HashMap;
use std::str::FromStr;
use std::cell::RefCell;
use std::rc::Rc;

use quick_xml::events::{Event, BytesEnd, BytesStart, BytesText};

use crate::error::Error;

type BeginHandler<'a> = Box<dyn FnMut(&[String], &BytesStart) ->
                            Result<(), Error> + 'a>;
type EndHandler<'a> = Box<dyn FnMut(&[String], &BytesEnd) ->
                          Result<(), Error> + 'a>;
type TextHandler<'a> = Box<dyn FnMut(&[String], &BytesText) ->
                           Result<(), Error> + 'a>;
type BeginHandlerMap<'a> = HashMap<&'static str, BeginHandler<'a>>;
type EndHandlerMap<'a> = HashMap<&'static str, EndHandler<'a>>;
type TextHandlerMap<'a> = HashMap<&'static str, TextHandler<'a>>;

pub type Reader<R> = Rc<RefCell<quick_xml::Reader<R>>>;

pub fn newReader<R: BufRead>(r: quick_xml::Reader<R>) -> Reader<R>
{
    Rc::new(RefCell::new(r))
}

pub struct Parser<'a>
{
    begin_handlers: BeginHandlerMap<'a>,
    end_handlers: EndHandlerMap<'a>,
    text_handlers: TextHandlerMap<'a>,
}

impl<'a> Parser<'a>
{
    pub fn new() -> Self
    {
        Self
        {
            begin_handlers: HashMap::new(),
            end_handlers: HashMap::new(),
            text_handlers: HashMap::new(),
        }
    }

    pub fn addBeginHandler<F>(&mut self, tag: &'static str, handler: F)
        where F: FnMut(&[String], &BytesStart) -> Result<(), Error> + 'a
    {
        self.begin_handlers.insert(tag, Box::new(handler));
    }

    pub fn addEndHandler<F>(&mut self, tag: &'static str, handler: F)
        where F: FnMut(&[String], &BytesEnd) -> Result<(), Error> + 'a
    {
        self.end_handlers.insert(tag, Box::new(handler));
    }
    pub fn addTextHandler<F>(&mut self, tag: &'static str, handler: F)
        where F: FnMut(&[String], &BytesText) -> Result<(), Error> + 'a
    {
        self.text_handlers.insert(tag, Box::new(handler));
    }

    pub fn parse<R: BufRead>(&mut self, opening: Option<&BytesStart>,
                             reader: Reader<R>) -> Result<(), Error>
    {
        let mut path: Vec<String> = Vec::new();
        if let Some(e) = opening
        {
            let tag: &str = str::from_utf8(e.name().into_inner())
                .map_err(|_| rterr!("Failed to decode UTF-8 from XML"))?;
            path.push(tag.to_owned());
            if let Some(f) = self.begin_handlers.get_mut(tag)
            {
                f(&path, &e)?;
            }
        }

        let mut stop: bool = false;
        let mut buffer = Vec::new();

        while !stop
        {
            match reader.borrow_mut().read_event_into(&mut buffer) {
                Ok(Event::Start(e)) =>
                {
                    let tag: &str = str::from_utf8(e.name().into_inner())
                        .map_err(
                            |_| rterr!("Failed to decode UTF-8 from XML"))?;
                    path.push(tag.to_owned());
                    if let Some(f) = self.begin_handlers.get_mut(tag)
                    {
                        f(&path, &e)?;
                    }
                },
                Ok(Event::Empty(e)) =>
                {
                    let tag: &str = str::from_utf8(e.name().into_inner())
                        .map_err(
                            |_| rterr!("Failed to decode UTF-8 from XML"))?;
                    path.push(tag.to_owned());
                    if let Some(f) = self.begin_handlers.get_mut(tag)
                    {
                        f(&path, &e)?;
                    }
                    path.pop();
                    if path.is_empty()
                    {
                        stop = true;
                    }
                },
                Ok(Event::End(e)) =>
                {
                    let tag: &str = str::from_utf8(e.name().into_inner())
                        .map_err(
                            |_| rterr!("Failed to decode UTF-8 from XML"))?;
                    if let Some(name) = path.last()
                    {
                        if *name != tag
                        {
                            return Err(
                                rterr!("Invalid XML. Expecting {}, got {}.",
                                       name, tag));
                        }
                    }
                    else
                    {
                        return Err(
                            rterr!("Invalid XML. XML should end, got {}.",
                                   tag));
                    }

                    if let Some(f) = self.end_handlers.get_mut(tag)
                    {
                        f(&path, &e)?;
                    }
                    path.pop();
                    if path.is_empty()
                    {
                        stop = true;
                    }
                },
                Ok(Event::Text(inner)) =>
                {
                    if let Some(tag) = path.last()
                    {
                        let tag: &str = tag;
                        if let Some(f) = self.text_handlers.get_mut(tag)
                        {
                            f(&path, &inner)?;
                        }
                    }
                    else
                    {
                        return Err(rterr!("Invalid XML. Rouge text found."));
                    }
                },
                Ok(_) => {},
                Err(e) =>
                {
                    return Err(rterr!("Failed to parse XML: {}", e));
                },
            }
        }
        Ok(())
    }
}

pub fn getTagAttr<T: FromStr>(tag: &BytesStart, attr: &str) ->
    Result<Option<T>, Error>
{
    if let Some(at) = tag.try_get_attribute(attr)
        .map_err(|_| rterr!("Failed to get attribute '{}'.", attr))?
    {
        let value: T = str::from_utf8(at.value.as_ref()).map_err(
            |_| rterr!("Failed to decode value of attribute '{}'.", attr))?
            .parse().map_err(
                |_| rterr!("Invalid value of attirbute '{}'.", attr))?;
        Ok(Some(value))
    }
    else
    {
        Ok(None)
    }
}

// ========== Tests =================================================>

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;

    struct A { a: String }

    #[test]
    fn parsing() -> Result<()>
    {
        let mut aaa: A = A { a: String::new() };
        let mut iin = 0;
        let mut out = 0;
        let mut p = Parser::new();
        p.addBeginHandler("a", |_, _| {
            iin += 1;
            Ok(())
        });
        p.addEndHandler("a", |_, _| {
            out += 1;
            Ok(())
        });
        p.addTextHandler("a", |_, t: &BytesText| {
            aaa.a = str::from_utf8(t.clone().into_inner().as_ref()).unwrap()
                .to_owned();
            Ok(())
        });
        p.parse(None, newReader(quick_xml::Reader::from_str(
            "<c><a>aaa</a><b/></c>")))?;
        drop(p);
        assert_eq!(iin, 1);
        assert_eq!(out, 1);
        assert_eq!(aaa.a, "aaa".to_owned());
        Ok(())
    }
}
