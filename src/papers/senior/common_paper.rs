use std::{fs};
use std::fs::File;
use std::path::Path;
use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::{Value};
use tera::Context;
use walkdir::WalkDir;
use crate::papers::TEMPLATES;
use super::super::{Answers, Answer};

#[derive(Debug, Serialize)]
pub struct CommonAnswers {
    pub choose: Vec<Choose>,
    pub fill: Fill,
    pub picture: Picture,
    pub read: Read,
    pub dialogue: Dialogues,
}
impl Answers for CommonAnswers {
    fn default() -> Self {
        CommonAnswers {
            choose: Vec::new(),
            fill: Fill::default(),
            picture: Picture::default(),
            read: Read::default(),
            dialogue: Dialogues::default(),
        }
    }

    fn read_answers(&mut self, paper_path: &str) -> Result<()> {
        // let mut answers = Answers::default();
        for (idx, answer_path) in WalkDir::new(paper_path).max_depth(1).into_iter().enumerate() {
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
                self.choose.push(*choose)
            } else if idx == 10 { // 填空题
                let fill = Fill::parse_from_json(json_path)?;
                // println!("{}: {:?}", idx, fill);
                self.fill = *fill;
            } else if idx == 11 { // 听后转述
                let picture = Picture::parse_from_json(json_path)?;
                // println!("{}: {:?}", idx, picture);
                self.picture = *picture;
            } else if idx == 12 { // 朗读
                let read = Read::parse_from_json(json_path)?;
                // println!("{}: {:?}", idx, read);
                self.read = *read;
            } else if idx == 13 { // 回答问题
                let dialogues = Dialogues::parse_from_json(json_path)?;
                // println!("{}: {:#?}", idx, dialogues);
                self.dialogue = *dialogues;
            }
        }
        Ok(())
    }
    fn export_to_html(&self, output_path: &str) -> Result<String> {
        let chooses: Vec<(String, Vec<(String, &str)>)> = {
            let mut chooses = Vec::new();
            for choose in &self.choose{
                for question in &choose.questions{
                    let stem = question.stem.clone();
                    let mut choices = vec![];
                    for option in &question.options{
                        let idx = &option[0..1];
                        let option = option.clone()
                            .trim_start_matches("A. ").to_string()
                            .trim_start_matches("B. ").to_string()
                            .trim_start_matches("C. ").to_string();
                        if idx == &question.answer[0..1] {
                            choices.push((option, "correct"))
                        } else { choices.push((option, "")) }
                    }
                    chooses.push((stem, choices))
                }
            }
            chooses
        }; // Vec<(题干, Vec<(选项, 类名)>)>

        let fills = self.fill.answer.clone()
            .iter_mut().map(|x| x.drain(2..).collect::<String>())
            .collect::<Vec<String>>();

        let dialogues = {
            let mut dialogues = vec![];
            for dialogue in &self.dialogue.dialogues{
                let question = dialogue.question.clone()
                    .trim_start_matches("Question 1. ").to_string()
                    .trim_start_matches("Question 2. ").to_string()
                    .trim_start_matches("Question 3. ").to_string()
                    .trim_start_matches("1. ").to_string()
                    .trim_start_matches("2. ").to_string()
                    .trim_start_matches("3. ").to_string();
                let answer = dialogue.answers.clone();
                dialogues.push((question, answer))
            }
            dialogues
        };

        let mut context = Context::new();
        context.insert("chooses_1", &chooses[..4]);
        context.insert("chooses_2", &chooses[4..]);
        context.insert("fills", &fills);
        context.insert("pictures", &self.picture.answer);
        context.insert("dialogues", &dialogues);
        let rendered = TEMPLATES.render("senior/CommonPaper.html", &context)?;


        if output_path == ":memory:" {
            Ok(rendered)
        } else {
            fs::write(output_path, rendered)?;
            Ok("success".to_string())
        }
    }

    fn export_to_pdf(&self, _output_path: &str) -> Result<()> {
        todo!()
    }

    fn export_to_json(&self, output_path: &str) -> Result<String> {
        let sdata = serde_json::to_string_pretty(&self)?;
        if output_path == ":memory:" {
            return Ok(sdata);
        }
        serde_json::to_writer(File::create(output_path)?, &self)?;
        Ok("Success to generate json file".parse()?)
    }
}



#[derive(Debug, Serialize)]
pub struct Choose {
    content: String, // 听力材料
    questions: Vec<ChooseQuestion>, // 题目
}
#[derive(Debug, Serialize)]
struct ChooseQuestion {
    stem: String, // 题干
    options: Vec<String>, // 选项
    answer: String, // 答案
}
impl Answer for Choose {
    fn default() -> Choose {
        Choose {
            content: String::new(),
            questions: Vec::new(),
        }
    }
    fn parse_from_json(json_path: &str) -> Result<Box<Self>> {
        let file_content = fs::read_to_string(json_path)?; // 读取json文件
        let data: Value = serde_json::from_str(&file_content)?; // 解析json文件

        let mut choose = Choose::default();

        let content = match data["info"]["st_nr"].as_str() {
            Some(content) => Ok(content),
            None => Err(anyhow!("Name field not found or not a string"))
        }?; // 获取听力材料(一对二的听力材料在这个位置)
        choose.content = content.to_string();

        let mut questions: Vec<ChooseQuestion> = Vec::new(); // 保存题目
        // 获取题目
        for question in data["info"]["xtlist"].as_array().unwrap().to_vec() {
            if choose.content.is_empty() {
                // 如果之前获取的听力材料是空的(一对一的听力材料在这个位置)
                let content = match question["xt_value"].as_str() {
                    Some(content) => Ok(content),
                    None => Err(anyhow!("Content not found"))
                }?;
                choose.content = content.to_string();
            }

            let stem = match question["xt_nr"].as_str() {
                Some(stem) => Ok(stem),
                None => Err(anyhow!("Stem not found"))
            }?;

            let mut options: Vec<String> = Vec::new(); // 存储选项
            for option in question["xxlist"].as_array().unwrap().to_vec() {
                let choice = match option["xx_mc"].as_str() {
                    Some(option) => Ok(option),
                    None => Err(anyhow!("Option not found"))
                }?; // 获取选项的字母
                let content = match option["xx_nr"].as_str() {
                    Some(content) => Ok(content),
                    None => Err(anyhow!("Content not found"))
                }?; // 获取选项的内容
                options.push(format!("{}.{}", choice, content)); // 将选项添加到列表中
            }

            let answer = match question["answer"].as_str() {
                Some(answer) => Ok(answer),
                None => Err(anyhow!("Answer not found"))
            }?; // 获取答案

            questions.push(ChooseQuestion {
                stem: stem.to_string(),
                options,
                answer: answer.to_string(),
            })
        }

        choose.questions = questions;


        choose.clean_data()?; // 清理数据
        Ok(Box::from(choose))
    }

    fn clean_data(&mut self) -> Result<()> {
        self.content = self.content.trim_start_matches("</p><p>").parse()?;
        self.content = self.content.split("</br>").map(|s| s.trim())
            .collect::<Vec<&str>>().join("\n");

        for question in self.questions.iter_mut() {
            question.stem = question.stem
                .trim_start_matches("ets_th1 ").to_string()
                .trim_start_matches("ets_th2 ").to_string()
        }
        Ok(())
    }
}


#[derive(Debug, Serialize)]
pub struct Fill {
    answer: Vec<String>, // 答案(不止一个)
}
impl Answer for Fill {
    fn default() -> Fill {
        Fill {
            answer: Vec::new(),
        }
    }

    fn parse_from_json(json_path: &str) -> Result<Box<Self>> {
        let file_content = fs::read_to_string(json_path)?;
        let data: Value = serde_json::from_str(&file_content)?;

        let mut fill = Fill::default();
        for answer in data["info"]["std"].as_array().unwrap().to_vec() {
            let idx = match answer["xth"].as_str() {
                None => Err(anyhow!("Index not found")),
                Some(idx) => Ok(idx),
            }?;
            let value = match answer["value"].as_str() {
                None => Err(anyhow!("Value not found")),
                Some(value) => Ok(value),
            }?;
            fill.answer.push(format!("{}.{}", idx, value));
        }

        Ok(Box::from(fill))
    }

    fn clean_data(&mut self) -> Result<()> {
        todo!()
    }
}


#[derive(Debug, Serialize)]
pub struct Picture {
    content: String, // 听力材料
    answer: Vec<String>, // 答案(不止一个)
    keypoints: Vec<String>, //关键点(不止一个)
}
impl Answer for Picture {
    fn default() -> Picture {
        Picture {
            content: String::new(), // 听力材料
            answer: Vec::new(), // 答案(不止一个)
            keypoints: Vec::new(), //关键点(不止一个)
        }
    }

    fn parse_from_json(json_path: &str) -> Result<Box<Self>> {
        let file_content = fs::read_to_string(json_path)?;
        let data: Value = serde_json::from_str(&file_content)?;

        let mut picture = Picture::default();

        picture.content = match data["info"]["value"].as_str() {
            Some(content) => Ok(content),
            None => Err(anyhow!("content not found")),
        }?.to_string();
        for answer in data["info"]["std"].as_array().unwrap().to_vec() {
            let value = match answer["value"].as_str() {
                Some(value) => Ok(value),
                None => Err(anyhow!("value not found")),
            }?;
            picture.answer.push(value.to_string());
        }
        picture.keypoints = match data["info"]["keypoint"].as_str() {
            Some(keypoint) => Ok(keypoint),
            None => Err(anyhow!("keypoint not found")),
        }?
            .split("</br>").map(|s| s.to_string()).collect::<Vec<String>>();


        picture.clean_data()?;
        Ok(Box::from(picture))
    }

    fn clean_data(&mut self) -> Result<()> {
        self.content = self.content.trim_start_matches("</p>").parse()?;
        self.content.push_str("</p>");
        self.content = self.content.replace("<p>", "").replace("</p>", "").replace("</br>", "\n");

        for answer in self.answer.iter_mut() {
            *answer = answer.trim_start_matches("</p><p>").parse()?
        }
        Ok(())
    }
}


#[derive(Debug, Serialize)]
pub struct Read {
    content: String, //朗读内容
}
impl Answer for Read {
    fn default() -> Read {
        Read {
            content: String::new(), // 听力材料
        }
    }

    fn parse_from_json(json_path: &str) -> Result<Box<Self>> {
        let file_content = fs::read_to_string(json_path)?;
        let data: Value = serde_json::from_str(&file_content)?;

        let mut read = Read::default();
        read.content = match data["info"]["value"].as_str() {
            Some(value) => Ok(value.to_string()),
            None => Err(anyhow!("value not found")),
        }?;

        read.clean_data()?;
        Ok(Box::from(read))
    }

    fn clean_data(&mut self) -> Result<()> {
        self.content = self.content.trim_start_matches("</p><p>").parse()?;
        Ok(())
    }
}


#[derive(Debug, Serialize)]
pub struct Dialogue {
    question: String, //问题
    answers: Vec<String>, //答案(不止一个)
    keywords: String, // 关键词
}
impl Dialogue {
    fn default() -> Dialogue {
        Dialogue {
            question: String::new(),
            answers: Vec::new(),
            keywords: String::new(),
        }
    }
}


#[derive(Debug, Serialize)]
pub struct Dialogues {
    dialogues: Vec<Dialogue>,
}
impl Answer for Dialogues {
    fn default() -> Dialogues {
        Dialogues {
            dialogues: vec![],
        }
    }

    fn parse_from_json(json_path: &str) -> Result<Box<Self>> {
        let file_content = fs::read_to_string(json_path)?;
        let data: Value = serde_json::from_str(&file_content)?;

        let mut dialogues: Dialogues = Dialogues::default();
        for dialogue_obj in data["info"]["question"].as_array().unwrap().to_vec() {
            let mut dialogue = Dialogue::default();
            dialogue.question = dialogue_obj["ask"].as_str().unwrap().to_string();
            dialogue.keywords = dialogue_obj["keywords"].as_str().unwrap().to_string();
            for answer in dialogue_obj["std"].as_array().unwrap().to_vec() {
                dialogue.answers.push(answer["value"].as_str().unwrap().to_string());
            }
            dialogues.dialogues.push(dialogue);
        }

        Ok(Box::from(dialogues))
    }

    fn clean_data(&mut self) -> Result<()> {
        todo!()
    }
}