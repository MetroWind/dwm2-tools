use serde::Serialize;

use data::GameData;
use data::skill::{self, Skill};

#[derive(Serialize)]
pub struct SkillReference
{
    pub name: String,
    pub is_current: bool,
    pub is_relevant: bool,
    pub monsters: Vec<String>,
}

impl SkillReference
{
    fn fromSkill(skill: &Skill, is_current: bool, is_relevant: bool,
                 data: &GameData) -> Self
    {
        Self {
            name: skill.name.clone(),
            is_current,
            is_relevant,
            monsters: data.monstersWithSkill(&skill.name)
                .map(|m| m.name.clone()).collect(),
        }
    }
}

pub type SkillUpgradePath = Vec<SkillReference>;

#[derive(Serialize)]
pub struct SkillCombination
{
    pub target: SkillReference,
    pub constituents: Vec<SkillUpgradePath>,
}

impl SkillCombination
{
    fn fromSkill(target: &Skill, current_skill: &Skill, data: &GameData) ->
        Option<Self>
    {
        if target.combine_from.is_empty()
        {
            return None;
        }

        let constituents: Vec<SkillUpgradePath> = target.combine_from
            .iter().map(|name| {
                let skill = data.skill(name).unwrap();
                data.skillUpgradePath(skill).iter().map(|s| {
                    SkillReference::fromSkill(
                        s,
                        &current_skill.name == &s.name,
                        &s.name == name,
                        data)
                }).collect()
            }).collect();
        Some(Self {
            target: SkillReference::fromSkill(
                target, target.name == current_skill.name, false, data),
            constituents,
        })
    }
}

#[derive(Serialize)]
pub struct SkillDetail
{
    pub name: String,
    pub monsters: Vec<String>,
    pub requirements: skill::Requirements,
    pub upgrade_path: SkillUpgradePath,
    pub combines_to: Vec<SkillCombination>,
    pub combines_from: Option<SkillCombination>,
}

impl SkillDetail
{
    pub fn fromSkill(skill: &Skill, data: &GameData) -> Option<Self>
    {
        let monsters: Vec<String> = data.monstersWithSkill(&skill.name)
            .map(|m| m.name.clone()).collect();
        let upgrade_path_skills = data.skillUpgradePath(skill);
        let upgrade_path: SkillUpgradePath = if upgrade_path_skills.len() > 1
        {
            upgrade_path_skills.iter().map(|s| SkillReference::fromSkill(
                s, s.name == skill.name, false, data))
                .collect()
        }
        else
        {
            Vec::new()
        };
        let combines_to: Vec<SkillCombination> = data.skillCombinesInto(skill)
            .map(|target| SkillCombination::fromSkill(target, skill, data)
                 .unwrap()).collect();
        let combines_from: Option<SkillCombination> =
            if skill.combine_from.is_empty()
        {
            None
        }
        else
        {
            Some(SkillCombination::fromSkill(skill, skill, data).unwrap())
        };

        Some(Self {
            name: skill.name.clone(),
            monsters,
            requirements: skill.requirements.clone(),
            upgrade_path,
            combines_to,
            combines_from,
        })
    }
}
