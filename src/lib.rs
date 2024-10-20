pub mod papers;

use anyhow::Result;
use winreg::{
    enums::{HKEY_LOCAL_MACHINE},
    RegKey,
};
use walkdir::WalkDir;
use serde::{Deserialize, Serialize};
use std::{env, fs};
use std::path::Path;
use std::time::SystemTime;
use chrono::{DateTime, Datelike};
pub use crate::papers::{Answer, Answers};
use crate::PaperType::SeniorCommonPaper;

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


// 定义一个枚举类型，表示试卷类型
#[derive(Debug, Deserialize, Serialize)]
pub enum PaperType {
    // 表示高中普通试卷
    SeniorCommonPaper,
    Unknown, // 表示未知试卷类型
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Paper {
    pub paper_id: String,
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

    pub fn read_answers(&self, paper_type: PaperType) -> Result<impl Answers> {
        match paper_type{
            SeniorCommonPaper => {
                let mut answers = papers::senior::CommonAnswers::default();
                answers.read_answers(&self.paper_path)?;
                Ok(answers)
            }
            _ => {panic!("Unsupported paper type: {:?}", paper_type)}
        }
    }
}

