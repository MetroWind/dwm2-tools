use std::fmt;
use std::str::FromStr;
use std::hash::{Hash, Hasher};
use std::io::prelude::*;
use std::collections::HashSet;

use regex::Regex;

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Sex { Male, Female, Any }

impl fmt::Display for Sex
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::Male => write!(f, "M"),
            Self::Female => write!(f, "F"),
            Self::Any => write!(f, "?"),
        }
    }
}

impl FromStr for Sex
{
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        match s
        {
            "M" => Ok(Self::Male),
            "F" => Ok(Self::Female),
            "?" => Ok(Self::Any),
            _ => Err(error!(FormatError, "Invalid sex: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Role { Base, Mate }

/// A monster of some kind, like “a female RainHawk”. In a breed plan,
/// a monster is uniquely identified (only) by the name, the sex, and
/// the index.
#[derive(Debug, Clone)]
struct Monster
{
    /// Name of the kind of monster, e.g. “RainHawk”.
    name: String,
    sex: Sex,
    /// This is used to distiguish two monsters of the same kind in a
    /// breed plan. If the plan has 2 slimes, one can have index 0
    /// (default) and the other can have index 1.
    index: u16,
    /// The minimal plus level required of this monster. 0 means no
    /// requirements.
    plus_level_min: u16,
}

impl Monster
{
    #[allow(dead_code)]
    fn new(name: &str, sex: Sex) -> Self
    {
        Self {
            name: name.to_owned(),
            sex: sex,
            index: 0,
            plus_level_min: 0,
         }
    }
}

impl PartialEq for Monster
{
    fn eq(&self, other: &Self) -> bool
    {
        self.name == other.name && self.sex == other.sex &&
            self.index == other.index
    }
}

impl Eq for Monster {}

impl Hash for Monster
{
    fn hash<H: Hasher>(&self, state: &mut H)
    {
        self.name.hash(state);
        self.sex.hash(state);
        self.index.hash(state);
    }
}

impl fmt::Display for Monster
{
    /// How to display a monster as a string. Note that if the sex is
    /// `Any`, it’s not included in the string. Similarly, 0 plus
    /// level requirements and/or 0 index is not included.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let sex_str = if self.sex == Sex::Any
        {
            String::new()
        }
        else
        {
            format!("({})", self.sex)
        };

        let index_str = if self.index > 0 { format!("/{}", self.index) }
        else {String::new()};

        write!(f, "{}{}{}", self.name, sex_str, index_str)
    }
}

impl FromStr for Monster
{
    type Err = Error;

    /// How to parse a Monster out of a string. This is the inverse of `fmt()`.
    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        let pattern = Regex::new(
            r"([a-zA-Z0-9]+)(\([MF?]\))?(/[0-9]+)?(\+[0-9]+)?"
        ).unwrap();
        if let Some(groups) = pattern.captures(s)
        {
            // Make sure it’s a complete match.
            let whole = groups.get(0).unwrap();
            if whole.start() != 0 || whole.end() != s.len()
            {
                return Err(error!(FormatError,
                                  "Invalid monster specification: {}",
                                  s));
            }

            Ok(Self {
                name: groups.get(1).ok_or_else(
                    || error!(FormatError, "Name not specified for monster"))?
                    .as_str().to_owned(),
                sex: if let Some(m) = groups.get(2)
                {
                    m.as_str()[1..2].parse()?
                }
                else
                {
                    Sex::Any
                },
                plus_level_min: if let Some(m) = groups.get(4)
                {
                    m.as_str()[1..].parse().map_err(
                        |_| error!(FormatError, "Invalid +lvl"))?
                }
                else
                {
                    0
                },
                index : if let Some(m) = groups.get(3)
                {
                    m.as_str()[1..].parse().map_err(
                        |_| error!(FormatError, "Invalid index"))?
                }
                else
                {
                    0
                }
            })
        }
        else
        {
            Err(error!(FormatError, "Invalid monster specification: {}", s))
        }
    }
}

/// Monster visualization spec. This defines how the monster is
/// displayed in the generated breed plan.
#[derive(Debug, Clone)]
struct MonsterVis
{
    monster: Monster,
    role: Option<Role>,
    /// This is the in-game name of the monster, given by the player.
    /// Different from `Monster::name`.
    name: Option<String>,
}

impl MonsterVis
{
    fn fromMonster(m: Monster, role: Option<Role>, name: Option<String>) -> Self
    {
        Self {
            monster: m,
            role: role,
            name: name,
        }
    }

    /// Generate a label for this monster in the dot file.
    fn label(&self) -> String
    {
        let plus_str = if self.monster.plus_level_min > 0
        {
            format!("+{}", self.monster.plus_level_min)
        }
        else
        {
            String::new()
        };

        let custom_name_str = if let Some(n) = &self.name
        {
            format!("<br/><font point-size=\"10\">“{}”</font>", n)
        }
        else
        {
            String::new()
        };

        self.monster.name.clone() + &plus_str + &custom_name_str
    }

    fn toDotSpec(&self) -> String
    {
        let color = match self.monster.sex
        {
            Sex::Male => "#70a1ff",
            Sex::Female => "#ff4757",
            Sex::Any => "#eccc68",
        };

        let border_str = if self.role == Some(Role::Base)
        {
            String::from(", penwidth=2")
        }
        else
        {
            String::new()
        };

        format!("\"{}\"[label=<{}>, style=\"filled\", fillcolor=\"{}\"{}, \
                 URL=\"https://darksair.org/dwm2-breed/monster/{}\"];",
                self.monster.to_string(), self.label(), color, border_str,
                self.monster.name)
    }

    /// Update this visualization spec from other visualization spec
    /// of the same monster. Set role and in-game name from `new` if
    /// self does not have them. Set the plus level requirement from
    /// `new` if that of `new` is higher in that of self.
    fn update(&mut self, new: Self)
    {
        if self.role == None
        {
            self.role = new.role;
        }
        if new.monster.plus_level_min > self.monster.plus_level_min
        {
            self.monster.plus_level_min = new.monster.plus_level_min;
        }
        if self.name == None
        {
            self.name = new.name;
        }
    }
}

impl FromStr for MonsterVis
{
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        let pattern = Regex::new(r"(.+):[ \t]+(.+)").unwrap();
        if let Some(groups) = pattern.captures(s)
        {
            // Make sure it’s a complete match.
            let whole = groups.get(0).unwrap();
            if whole.start() != 0 || whole.end() != s.len()
            {
                return Err(error!(FormatError,
                                  "Invalid monster specification: {}",
                                  s));
            }

            let monster: Monster = groups.get(1).unwrap().as_str().parse()?;
            let name = groups.get(2).unwrap().as_str();
            Ok(Self {
                monster: monster,
                role: None,
                name: Some(String::from(name)),
            })
        }
        else
        {
            Err(error!(FormatError, "Invalid monster specification: {}", s))
        }
    }
}

impl PartialEq for MonsterVis
{
    fn eq(&self, other: &Self) -> bool
    {
        self.monster == other.monster
    }
}

impl Eq for MonsterVis {}

impl Hash for MonsterVis
{
    fn hash<H: Hasher>(&self, state: &mut H)
    {
        self.monster.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Breed
{
    base: Monster,
    mate: Monster,
    outcome: Monster,
}

impl FromStr for Breed
{
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        let s = s.trim();

        let (lhs, rhs) = s.split_once('=')
            .ok_or_else(|| error!(FormatError, "Invalid breed: {}", s))?;

        let outcome_str = rhs.trim();

        let (base_str, mate_str) = lhs.split_once('+')
            .ok_or_else(|| error!(FormatError, "Invalid breed: {}", s))?;

        Ok(Self {
            base: base_str.trim().parse()?,
            mate: mate_str.trim().parse()?,
            outcome: outcome_str.parse()?,
        })
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
enum BreedOrSpec
{
    Breed(Breed),
    Spec(MonsterVis),
}

impl FromStr for BreedOrSpec
{
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        if let Ok(breed) = s.parse::<Breed>()
        {
            Ok(Self::Breed(breed))
        }
        else
        {
            Ok(Self::Spec(s.parse()?))
        }
    }
}

pub struct BreedPlan
{
    steps: Vec<Breed>,
    specs: HashSet<MonsterVis>,
}

impl BreedPlan
{
    pub fn new() -> Self
    {
        Self { steps: Vec::new(), specs: HashSet::new() }
    }

    fn addStep(&mut self, breed: Breed)
    {
        self.steps.push(breed);
    }

    fn addSpec(&mut self, spec: MonsterVis) -> bool
    {
        if self.specs.contains(&spec)
        {
            false
        }
        else
        {
            self.specs.insert(spec);
            true
        }
    }

    pub fn fromStream(stream: &mut dyn BufRead) -> Result<Self, Error>
    {
        let mut plan = Self::new();
        for line in stream.lines()
        {
            let line = line.map_err(
                |e| rterr!("Failed to read a line: {}", e))?;
            if line.trim().is_empty()
            {
                continue;
            }
            if line.chars().next() == Some('#')
            {
                continue;
            }

            match line.parse::<BreedOrSpec>()?
            {
                BreedOrSpec::Breed(b) => { plan.addStep(b); },
                BreedOrSpec::Spec(vis) => {
                    let monster_str = vis.monster.to_string();
                    if !plan.addSpec(vis)
                    {
                        println!("WARNING: duplicated monster spec for {}, \
                                  ignoring...",
                                 monster_str);
                    }
                },
            }
        }
        Ok(plan)
    }

    pub fn toDot(&self) -> String
    {
        let mut lines: Vec<String> = Vec::new();
        let mut monsters: HashSet<MonsterVis> = self.specs.clone();

        lines.push(String::from("digraph G {"));
        lines.push(String::from("node[shape=\"box\"];"));
        for breed in &self.steps
        {
            let base_vis = MonsterVis::fromMonster(
                breed.base.clone(), Some(Role::Base), None);
            let mate_vis = MonsterVis::fromMonster(
                breed.mate.clone(), Some(Role::Mate), None);
            let result_vis = MonsterVis::fromMonster(
                breed.outcome.clone(), None, None);

            match monsters.take(&base_vis)
            {
                Some(mut m) => {
                    m.update(base_vis);
                    monsters.insert(m);
                },
                None => { monsters.insert(base_vis); },
            }
            match monsters.take(&mate_vis)
            {
                Some(mut m) => {
                    m.update(mate_vis);
                    monsters.insert(m);
                },
                None => { monsters.insert(mate_vis); },
            }
            match monsters.take(&result_vis)
            {
                Some(mut m) => {
                    m.update(result_vis);
                    monsters.insert(m);
                },
                None => { monsters.insert(result_vis); },
            }
            lines.push(format!("\"{}\" -> \"{}\";", breed.base.to_string(),
                               breed.outcome.to_string()));
            lines.push(format!("\"{}\" -> \"{}\";", breed.mate.to_string(),
                               breed.outcome.to_string()));
        }

        for m in monsters
        {
            lines.push(m.toDotSpec());
        }

        lines.push(String::from("}"));
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests
{
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn printMonster()
    {
        assert_eq!(Monster::new("Zapbird", Sex::Female).to_string(),
                   "Zapbird(F)");
        assert_eq!(Monster
                   {
                       name: String::from("Zapbird"),
                       sex: Sex::Any,
                       index: 1,
                       plus_level_min: 0,
                   }.to_string(),
                   "Zapbird/1");
        assert_eq!(Monster
                   {
                       name: String::from("Zapbird"),
                       sex: Sex::Male,
                       index: 1,
                       plus_level_min: 2,
                   }.to_string(),
                   "Zapbird(M)/1");
    }

    #[test]
    fn parseMonster() -> Result<(), Error>
    {
        assert_eq!("Zapbird(M)/3+2".parse::<Monster>()?,
                   Monster
                   {
                       name: String::from("Zapbird"),
                       sex: Sex::Male,
                       index: 3,
                       plus_level_min: 2,
                   });
        assert_eq!("Zapbird".parse::<Monster>()?,
                   Monster
                   {
                       name: String::from("Zapbird"),
                       sex: Sex::Any,
                       index: 0,
                       plus_level_min: 0,
                   });
        assert_eq!("Zapbird/2".parse::<Monster>()?,
                   Monster
                   {
                       name: String::from("Zapbird"),
                       sex: Sex::Any,
                       index: 2,
                       plus_level_min: 0,
                   });
        assert_eq!("Zapbird+5".parse::<Monster>()?,
                   Monster
                   {
                       name: String::from("Zapbird"),
                       sex: Sex::Any,
                       index: 0,
                       plus_level_min: 5,
                   });
        Ok(())
    }

    #[test]
    fn invalidMonster() -> Result<(), Error>
    {
        assert!("Zapbird\\1".parse::<Monster>().is_err());
        assert!("".parse::<Monster>().is_err());
        assert!("Zapbird+".parse::<Monster>().is_err());
        assert!("Zapbird\\4+1".parse::<Monster>().is_err());
        assert!("Zapbird+2/1".parse::<Monster>().is_err());
        assert!("Zapbird+2(F)".parse::<Monster>().is_err());
        Ok(())
    }

    #[test]
    fn parseBreed() -> Result<(), Error>
    {
        assert_eq!("Base + Mate = Result".parse::<Breed>()?,
                Breed {
                    base: Monster::new("Base", Sex::Any),
                    mate: Monster::new("Mate", Sex::Any),
                    outcome: Monster::new("Result", Sex::Any),
                });

        // Regression: allow extra spaces around '='
        assert_eq!("Blizzardy + Phoenix  = RainHawk".parse::<Breed>()?,
                Breed {
                    base: Monster::new("Blizzardy", Sex::Any),
                    mate: Monster::new("Phoenix", Sex::Any),
                    outcome: Monster::new("RainHawk", Sex::Any),
                });

        // Bonus: allow no spaces too
        assert_eq!("Base+Mate=Result".parse::<Breed>()?,
                Breed {
                    base: Monster::new("Base", Sex::Any),
                    mate: Monster::new("Mate", Sex::Any),
                    outcome: Monster::new("Result", Sex::Any),
                });

        Ok(())
    }

    #[test]
    fn invalidBreed() -> Result<(), Error>
    {
        assert!("Base + = Result".parse::<Breed>().is_err());
        assert!("Base + + = Result".parse::<Breed>().is_err());
        assert!("Base + Result".parse::<Breed>().is_err());
        assert!("".parse::<Breed>().is_err());
        Ok(())
    }
}
