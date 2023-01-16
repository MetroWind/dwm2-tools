//! A simple XML parser based on quick-xml.
//!
//! This is basically an interface on top of quick-xml. It provides an
//! API to let the user write callback functions for specific parts of
//! the document, and hides the reader from the user.
//!
//! Because of its simple nature, it has a set of limitations:
//!
//! * It only parses from a byte buffer. If the XML is from a file,
//! the user needs to read the file into a buffer before using the
//! parser.
//!
//! * The XML has to be encoded in UTF-8.
//!
//! ## Example
//!
//! ```
//! # use std::str;
//! # use data::xml::Parser;
//! struct A { a: String }
//!
//! let mut aaa: A = A { a: String::new() };
//! let mut iin = 0;
//! let mut out = 0;
//! let mut d = String::new();
//!
//! let mut p = Parser::new();
//! p.addBeginHandler("a", |_, _| {
//!     iin += 1;
//!     Ok(())
//! });
//! p.addEndHandler("a", |_, _| {
//!     out += 1;
//!     Ok(())
//! });
//! p.addTextHandler("a", |_, t| {
//!     aaa.a = t.to_owned();
//!     Ok(())
//! });
//! p.addTagHandler("d", |_, t| {
//!     d = str::from_utf8(t).unwrap().to_owned();
//!     Ok(())
//! });
//! p.parse(r#"
//! <c>
//!     <a>aaa</a>
//!     <b/>
//!     <d>
//!         <e/>
//!     </d>
//! </c>
//! "#.as_bytes()).unwrap();
//! drop(p);
//! assert_eq!(iin, 1);
//! assert_eq!(out, 1);
//! assert_eq!(aaa.a, "aaa".to_owned());
//! assert_eq!(d, r#"<d>
//!         <e/>
//!     </d>"#);
//! ```

use std::str;
use std::collections::HashMap;
use std::str::FromStr;

use quick_xml::events::{Event, BytesEnd, BytesStart};

use error::Error;

type BeginHandler<'a> = Box<dyn FnMut(&[String], &BytesStart) ->
                            Result<(), Error> + 'a>;
type EndHandler<'a> = Box<dyn FnMut(&[String], &BytesEnd) ->
                          Result<(), Error> + 'a>;
type TextHandler<'a> = Box<dyn FnMut(&[String], &str) ->
                           Result<(), Error> + 'a>;
type TagHandler<'a> = Box<dyn FnMut(&[String], &[u8]) ->
                           Result<(), Error> + 'a>;
type BeginHandlerMap<'a> = HashMap<&'static str, BeginHandler<'a>>;
type EndHandlerMap<'a> = HashMap<&'static str, EndHandler<'a>>;
type TextHandlerMap<'a> = HashMap<&'static str, TextHandler<'a>>;
type TagHandlerMap<'a> = HashMap<&'static str, TagHandler<'a>>;

/// A simple event callback-based XML parser.
pub struct Parser<'a>
{
    begin_handlers: BeginHandlerMap<'a>,
    end_handlers: EndHandlerMap<'a>,
    text_handlers: TextHandlerMap<'a>,
    tag_handlers: TagHandlerMap<'a>,
}

impl<'a> Parser<'a>
{
    /// Create a parser with no callbacks.
    pub fn new() -> Self
    {
        Self
        {
            begin_handlers: HashMap::new(),
            end_handlers: HashMap::new(),
            text_handlers: HashMap::new(),
            tag_handlers: HashMap::new(),
        }
    }

    /// Add a callback for an opening tag. If the parser encounters an
    /// opening tag whose name coincides with the value of `tag`, it
    /// calls `handler` with the opening tag event. Self-closing tags
    /// also trigger begin handlers.
    pub fn addBeginHandler<F>(&mut self, tag: &'static str, handler: F)
        where F: FnMut(&[String], &BytesStart) -> Result<(), Error> + 'a
    {
        self.begin_handlers.insert(tag, Box::new(handler));
    }

    /// Add a callback for an end tag. If the parser encounters an end
    /// tag whose name coincides with the value of `tag` (not
    /// including the starting `/`), it calls `handler` with the end
    /// tag event. Self-closing tags also trigger end handlers.
    pub fn addEndHandler<F>(&mut self, tag: &'static str, handler: F)
        where F: FnMut(&[String], &BytesEnd) -> Result<(), Error> + 'a
    {
        self.end_handlers.insert(tag, Box::new(handler));
    }

    /// Add a callback for text element directly inside some tag. If
    /// the parser encounters a text element where it’s enclosing tag
    /// coincides with the value of `tag`, it calls `handler` with the
    /// decoded text string.
    ///
    /// Note that this does not mean the text needs to be the only or
    /// the last element in the enclosing tag.
    pub fn addTextHandler<F>(&mut self, tag: &'static str, handler: F)
        where F: FnMut(&[String], &str) -> Result<(), Error> + 'a
    {
        self.text_handlers.insert(tag, Box::new(handler));
    }

    /// Add a callback for a whole tag. If the parser encounters an
    /// opening element (including self-closing tags) whose name
    /// coincides with the value of `tag`, it calls `handler` with the
    /// content of the whole tag, including the opening and the
    /// closing tag. The parser then skips the whole tag.
    ///
    /// This is useful if the user wants to delegate the parsing of a
    /// tag to another parser.
    pub fn addTagHandler<F>(&mut self, tag: &'static str, handler: F)
        where F: FnMut(&[String], &[u8]) -> Result<(), Error> + 'a
    {
        self.tag_handlers.insert(tag, Box::new(handler));
    }

    /// Parse the XML in the byte buffer `x`, triggering the callbacks
    /// in the process. It is important to note that *this buffer
    /// should only contains one root tag*.
    pub fn parse(&mut self, x: &[u8]) -> Result<(), Error>
    {
        let mut reader = quick_xml::Reader::from_str(unsafe {
            str::from_utf8_unchecked(x)
        });
        let mut path: Vec<String> = Vec::new();
        let mut stop: bool = false;

        while !stop
        {
            let pos_before = reader.buffer_position();
            match reader.read_event() {
                Ok(Event::Start(e)) =>
                {
                    let tag: &str = str::from_utf8(e.name().into_inner())
                        .map_err(
                            |_| xmlerr!("Failed to decode UTF-8 from XML"))?;
                    path.push(tag.to_owned());
                    if let Some(f) = self.begin_handlers.get_mut(tag)
                    {
                        f(&path, &e)?;
                    }

                    if let Some(f) = self.tag_handlers.get_mut(tag)
                    {
                        reader.read_to_end(e.to_end().name()).map_err(
                            |_| xmlerr!("Failed to find end tag of {}.", tag))?;
                        f(&path, &x[pos_before..reader.buffer_position()])?;
                        path.pop();
                    }
                },
                Ok(Event::Empty(e)) =>
                {
                    let tag: &str = str::from_utf8(e.name().into_inner())
                        .map_err(
                            |_| xmlerr!("Failed to decode UTF-8 from XML"))?;
                    path.push(tag.to_owned());
                    if let Some(f) = self.begin_handlers.get_mut(tag)
                    {
                        f(&path, &e)?;
                    }
                    if let Some(f) = self.end_handlers.get_mut(tag)
                    {
                        f(&path, &e.to_end())?;
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
                            |_| xmlerr!("Failed to decode UTF-8 from XML"))?;
                    if let Some(name) = path.last()
                    {
                        if *name != tag
                        {
                            return Err(
                                xmlerr!("Invalid XML. Expecting {}, got {}.",
                                       name, tag));
                        }
                    }
                    else
                    {
                        return Err(
                            xmlerr!("Invalid XML. XML should end, got {}.",
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
                            let t = str::from_utf8(inner.as_ref()).map_err(
                                |_| xmlerr!("Failed to decode text element in \
                                            {}", tag))?;
                            f(&path, t)?;
                        }
                    }
                    else
                    {
                        // If the XML ends in whitespace, this branch
                        // will trigger, which is fine.
                    }
                },
                Ok(_) => {},
                Err(e) =>
                {
                    return Err(xmlerr!("Failed to parse XML: {}", e));
                },
            }
        }
        Ok(())
    }
}

/// Return the value of the attribute `attr` from the opening tag
/// `tag`.
pub fn getTagAttr<T: FromStr>(tag: &BytesStart, attr: &str) ->
    Result<Option<T>, Error>
{
    if let Some(at) = tag.try_get_attribute(attr)
        .map_err(|_| xmlerr!("Failed to get attribute '{}'.", attr))?
    {
        let value: T = str::from_utf8(at.value.as_ref()).map_err(
            |_| xmlerr!("Failed to decode value of attribute '{}'.", attr))?
            .parse().map_err(
                |_| xmlerr!("Invalid value of attirbute '{}'.", attr))?;
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

    struct A { a: String }

    #[test]
    fn parsing() -> Result<(), Error>
    {
        let mut aaa: A = A { a: String::new() };
        let mut iin = 0;
        let mut out = 0;
        let mut d = String::new();

        let mut p = Parser::new();
        p.addBeginHandler("a", |_, _| {
            iin += 1;
            Ok(())
        });
        p.addEndHandler("a", |_, _| {
            out += 1;
            Ok(())
        });
        p.addTextHandler("a", |_, t| {
            aaa.a = t.to_owned();
            Ok(())
        });
        p.addTagHandler("d", |_, t| {
            d = str::from_utf8(t).unwrap().to_owned();
            Ok(())
        });
        p.parse(r#"
<c>
    <a>aaa</a>
    <b/>
    <d>
        <e/>
    </d>
</c>
"#.as_bytes())?;
        drop(p);
        assert_eq!(iin, 1);
        assert_eq!(out, 1);
        assert_eq!(aaa.a, "aaa".to_owned());
        assert_eq!(d, r#"<d>
        <e/>
    </d>"#);
        Ok(())
    }
}
