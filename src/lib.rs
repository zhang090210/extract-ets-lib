use anyhow::{anyhow, Result};
use winreg::{
    enums::{HKEY_LOCAL_MACHINE},
    RegKey,
};
use walkdir::WalkDir;
use serde::Serialize;
use std::{env, fs};
use std::path::Path;
use std::time::SystemTime;
use chrono::{DateTime, Datelike};
use crate::get_res::{Answer, Answers, Choose, Dialogues, Fill, Picture, Read};

const LOW_ETS_VERSION: [u8; 3] = [5, 7, 1];

#[derive(Debug)]
#[warn(dead_code)]
pub struct ETS {
    resource_path: String,
}

impl ETS {
    pub fn default() -> Result<Self> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE); //定义根(HKEY_LOCAL_MACHINE)
        let cur_ver = hklm.open_subkey("SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\ETS")?; // 尝试获取子健
        let install_version: String = cur_ver.get_value("DisplayVersion")?; // 获取安装版本

        let version = &install_version.split('.').collect::<Vec<&str>>();
        for idx in 0..version.len() {
            if version[idx].parse::<u8>()? < LOW_ETS_VERSION[idx] {
                return Err(
                    anyhow::anyhow!(
                        "ETS version is too low, please update to version {}",
                        LOW_ETS_VERSION.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(".")
                    )
                );
            }
        }

        Ok(ETS { resource_path: env::var("APPDATA")? + r"\ETS" })
    }

    pub fn get_papers(&self) -> Result<Vec<Paper>> {
        let mut papers: Vec<Paper> = Vec::new();
        for paper_path in WalkDir::new(&self.resource_path).max_depth(1) {
            let paper_path = paper_path?;
            let paper_path = match paper_path.path().to_str() {
                Some(p) => p,
                None => continue,
            };
            if paper_path == self.resource_path || paper_path.split('\\').last().unwrap() == "common" {
                continue;
            }
            let paper = Paper::read_from_path(paper_path)?;
            papers.push(paper)
        }

        Ok(
            papers
        )
    }
}

#[derive(Debug, Serialize)]
pub struct Paper {
    paper_id: String,
    pub paper_path: String,
    pub paper_time: (i32, u32, u32), // (year, month, day)
}

impl Paper {
    fn default() -> Paper {
        Paper {
            paper_id: String::new(),
            paper_path: String::new(),
            paper_time: (0, 0, 0),
        }
    }

    pub fn read_from_path(paper_path: &str) -> Result<Paper> {
        let paper_id = paper_path.split('\\').last().unwrap().to_string();
        let paper_path = paper_path;

        let path = Path::new(paper_path); // 假设我们有一个路径

        // 获取文件或目录的元数据
        let metadata = fs::metadata(path)?;

        // 获取修改时间
        let modified = metadata.modified()?;

        // 将SystemTime转换为chrono的DateTime类型
        let modified_date = modified.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64;
        let datetime = DateTime::from_timestamp(modified_date, 0).unwrap();
        let paper_time = (datetime.year(), datetime.month(), datetime.day());

        Ok(
            Paper {
                paper_id,
                paper_path: paper_path.to_string(),
                paper_time,
                ..Paper::default()
            }
        )
    }

    pub fn read_answers(&self) -> Result<Answers> {
        let mut answers = Answers::default();
        for (idx, answer_path) in WalkDir::new(&self.paper_path).max_depth(1).into_iter().enumerate() {
            let answer_path = answer_path?;
            let answer_path = answer_path.path().to_str().unwrap();
            let json_path = Path::new(answer_path).join("content.json");
            let json_path = match json_path.to_str() {
                Some(path) => Ok(path),
                None => Err(anyhow!("Can't found json file")),
            }?;
            if idx > 0 && idx < 10 { // 选择题
                let choose = Choose::parse_from_json(json_path)?;
                // println!("{}: {:?}", idx, choose);
                answers.choose.push(*choose)
            } else if idx == 10 { // 填空题
                let fill = Fill::parse_from_json(json_path)?;
                // println!("{}: {:?}", idx, fill);
                answers.fill = *fill;
            } else if idx == 11 { // 听后转述
                let picture = Picture::parse_from_json(json_path)?;
                // println!("{}: {:?}", idx, picture);
                answers.picture = *picture;
            } else if idx == 12 { // 朗读
                let read = Read::parse_from_json(json_path)?;
                // println!("{}: {:?}", idx, read);
                answers.read = *read;
            } else if idx == 13 { // 回答问题
                let dialogues = Dialogues::parse_from_json(json_path)?;
                // println!("{}: {:#?}", idx, dialogues);
                answers.dialogue = *dialogues;
            }
        }
        Ok(answers)
    }
}
