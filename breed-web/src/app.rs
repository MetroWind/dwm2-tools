use std::io::Read;
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use std::fs::File;

use log::info;
use log::error as log_error;
use tera::Tera;
use warp::{Filter, Reply};
use warp::http::status::StatusCode;
use warp::redirect::see_other;
use warp::reply::Response;

use error::Error;
use data::GameData;
use data::monster::Monster;
use data::breed::{Parent, Formula};
use crate::config::Configuration;

trait ToResponse
{
    fn toResponse(self) -> Response;
}

impl ToResponse for Result<String, Error>
{
    fn toResponse(self) -> Response
    {
        match self
        {
            Ok(s) => warp::reply::html(s).into_response(),
            Err(e) => {
                log_error!("{}", e);
                warp::reply::with_status(
                e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            },
        }
    }
}

impl ToResponse for Result<Response, Error>
{
    fn toResponse(self) -> Response
    {
        match self
        {
            Ok(s) => s.into_response(),
            Err(e) => {
                log_error!("{}", e);
                warp::reply::with_status(
                e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            }
        }
     }
}

fn handleIndex(data: &data::GameData, templates: &Tera) -> Result<String, Error>
{
    let mut context = tera::Context::new();
    context.insert("families", &data.monster_data.families);

    templates.render("index.html", &context).map_err(
        |e| rterr!("Failed to render template index.html: {}", e))
}

fn handleFamily(family_name: String, data: &data::GameData, templates: &Tera,)
                -> Result<String, Error>
{
    let family_name = urlencoding::decode(&family_name).map_err(
        |_| rterr!("Invalid family: {}", family_name))?.to_string();
    if let Some(family) = data.family(&family_name)
    {
        let mut context = tera::Context::new();
        context.insert("family", &family);

        let monsters: Vec<Monster> = data.monstersInFamily(&family)
            .map(|f| f.clone()).collect();
        context.insert("monsters", &monsters);

        let parent = Parent::Family(family_name);
        let forms: Vec<&Formula> = data.usedInFormulae(&parent).collect();
        context.insert("uses", &forms);

        templates.render("family.html", &context).map_err(
            |e| rterr!("Failed to render template index.html: {}", e))
    }
    else
    {
        Err(rterr!("Family not found: {}", family_name))
    }
}

fn handleMonster(monster_name: String, data: &data::GameData, templates: &Tera,)
                -> Result<String, Error>
{
    let monster_name = urlencoding::decode(&monster_name).map_err(
        |_| rterr!("Invalid monster: {}", monster_name))?.to_string();
    if let Some(monster) = data.monster(&monster_name)
    {
        let mut context = tera::Context::new();
        context.insert("monster", &monster);

        let family = data.family(&monster.family).ok_or_else(
            || rterr!("Invalid family '{}' of monster {}", monster.family,
                      monster.name))?;
        context.insert("family_name", &family.name);

        let breeds: Vec<&Formula> =
            data.breedFromFormulae(&monster_name).collect();
        context.insert("breeds", &breeds);

        let parent = Parent::Monster(monster_name);
        let uses: Vec<&Formula> = data.usedInFormulae(&parent).collect();
        context.insert("uses", &uses);

        templates.render("monster.html", &context).map_err(
            |e| rterr!("Failed to render template index.html: {}", e))
    }
    else
    {
        Err(rterr!("Monster not found: {}", monster_name))
    }

}

fn urlEncode(s: &str) -> String
{
    urlencoding::encode(s).to_string()
}

fn urlFor(name: &str, arg: &str) -> String
{
    match name
    {
        "index" => String::from("/"),
        "family" => String::from("/family/") + &urlEncode(arg),
        "monster" => String::from("/monster/") + &urlEncode(arg),
        "static" => String::from("/static/") + &urlEncode(arg),
        _ => String::from("/"),
    }
}

fn getTeraFuncArgs(args: &HashMap<String, tera::Value>, arg_name: &str) ->
    tera::Result<String>
{
    let value = args.get(arg_name);
    if value.is_none()
    {
        return Err(format!("Argument {} not found in function call.", arg_name)
                   .into());
    }
    let value: String = tera::from_value(value.unwrap().clone())?;
    Ok(value)
}

fn makeURLFor() -> impl tera::Function
{
    |args: &HashMap<String, tera::Value>| ->
        tera::Result<tera::Value> {
            let name = getTeraFuncArgs(args, "name")?;
            let arg = getTeraFuncArgs(args, "arg")?;
            Ok(tera::to_value(urlFor(&name, &arg)).unwrap())
    }
}

pub struct App
{
    data: data::GameData,
    templates: Tera,
    config: Configuration,
}

impl App
{
    pub fn new(config: Configuration) -> Result<Self, Error>
    {
        let mut result = Self {
            data: data::GameData::default(),
            templates: Tera::default(),
            config,
        };
        result.init()?;
        Ok(result)
    }

    fn init(&mut self) -> Result<(), Error>
    {
        {
            let data_path = Path::new(&self.config.data_dir)
                .join("monster-data.xml");
            let mut data_file = File::open(data_path).map_err(
                |_| rterr!("Failed to open data file"))?;
            let mut raw_data: Vec<u8> = Vec::new();
            data_file.read_to_end(&mut raw_data).map_err(
                |_| rterr!("Failed to read data file"))?;
            self.data = GameData::fromXML(&raw_data)?;
        }

        let template_path = PathBuf::from(&self.config.data_dir)
            .join("templates").canonicalize()
            .map_err(|_| rterr!("Invalid template dir"))?
            .join("**").join("*");
        info!("Template dir is {}", template_path.display());
        let template_dir = template_path.to_str().ok_or_else(
                || rterr!("Invalid template path"))?;
        self.templates = Tera::new(template_dir).map_err(
            |e| rterr!("Failed to compile templates: {}", e))?;
        self.templates.register_function("url_for", makeURLFor());
        // self.templates.register_filter("urlencode", filterEncodeURI);

        Ok(())
    }

    pub async fn serve(self) -> Result<(), Error>
    {
        let static_dir = PathBuf::from(&self.config.data_dir).join("static");
        info!("Static dir is {}", static_dir.display());
        let statics = warp::path("static").and(warp::fs::dir(static_dir));

        let data = self.data.clone();
        let temp = self.templates.clone();
        let index = warp::get().and(warp::path::end()).map(move || {
            handleIndex(&data, &temp).toResponse()
        });

        let data = self.data.clone();
        let temp = self.templates.clone();
        let family = warp::get().and(warp::path("family"))
            .and(warp::path::param()).map(move |param: String| {
                handleFamily(param, &data, &temp).toResponse()
            });

        let data = self.data.clone();
        let temp = self.templates.clone();
        let monster = warp::get().and(warp::path("monster"))
            .and(warp::path::param()).map(move |param: String| {
                handleMonster(param, &data, &temp).toResponse()
            });
        info!("Listening at {}:{}...", self.config.listen_address,
              self.config.listen_port);

        warp::serve(statics.or(index).or(family).or(monster)).run(
            std::net::SocketAddr::new(
                self.config.listen_address.parse().map_err(
                    |_| rterr!("Invalid listen address: {}",
                               self.config.listen_address))?,
                self.config.listen_port)).await;
        Ok(())
    }
}
