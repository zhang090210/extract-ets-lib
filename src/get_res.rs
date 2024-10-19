use std::fs;
use std::fs::File;
use std::io::Write;
use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::{Value};
use wkhtmltopdf::{Orientation, PdfApplication, Size};

#[derive(Debug, Serialize)]
pub struct Answers {
    pub choose: Vec<Choose>,
    pub fill: Fill,
    pub picture: Picture,
    pub read: Read,
    pub dialogue: Dialogues,
}

impl Answers {
    pub fn default() -> Self {
        Answers {
            choose: Vec::new(),
            fill: Fill::default(),
            picture: Picture::default(),
            read: Read::default(),
            dialogue: Dialogues::new(),
        }
    }
    pub fn export_to_html(&self, output_path: &str) -> Result<String> {
        let mut dom = "".to_string();

        dom += &"<!DOCTYPE html><html lang='zh-CN'><head><meta charset='UTF-8'></head>".to_string();

        dom += "<h2>一、选择</h2>";
        for i1 in 0..self.choose.len() {
            let question_len = self.choose[i1].questions.len();
            let content = &self.choose[i1].content;
            dom += &format!("<p><strong>听力材料: </strong>{}</p>", content.replace("<p>", "").replace("</p>", ""));
            for i2 in 0..question_len {
                dom += &format!("<p>question {}: {}</p>", i2 + 1, self.choose[i1].questions[i2].stem.replace("ets_th1 ", ""));
                for option in &self.choose[i1].questions[i2].options {
                    dom += &format!("<p>{}</p>", option);
                }
                dom += &format!("<p>答案: {}</p>", self.choose[i1].questions[i2].answer);
            }
        }

        dom += "<h2>二、填空</h2>";
        for i1 in 0..self.fill.answer.len() {
            dom += &format!("{}<br/>", self.fill.answer[i1]);
        }
        dom.trim_end_matches("<br/>").to_string();

        dom += "<h2>三、听后转述</h2>";
        dom += &format!("<p><strong>听力材料: </strong>{}</p>", self.picture.content);
        dom += &"<p>答案: </p>".to_string();
        for answer in &self.picture.answer {
            dom += &format!("<p>{}</p>", answer);
        }
        dom += &"<p>关键点: </p>".to_string();
        for keypoint in &self.picture.keypoints {
            dom += &format!("{}", keypoint);
        }

        dom += "<h2>四、朗读</h2>";
        dom += &format!("<p><strong>文章: </strong>{}</p>", self.read.content);

        dom += "<h2>五、回答问题</h2>";
        for dialog in &self.dialogue.dialogues {
            dom += &format!("<b>{}</b>", dialog.question);
            for answer in &dialog.answers {
                dom += &format!("<p>{}</p>", answer);
            }
            dom += &format!("<p><strong>keywords: </strong>{}</p>", dialog.keywords)
        }

        if output_path != ":memory:" {
            let mut html = File::create(output_path)?;
            html.write_all(dom.as_bytes())?;
            html.sync_all()?;
            Ok("Success to generate html file".parse()?)
        } else {
            Ok(dom)
        }
    }

    pub fn export_pdf(&self, pdf_app: &PdfApplication, output_path: &str) -> Result<()> {
        let content = self.export_to_html(":memory:")?;

        let mut pdfout = pdf_app.builder()
            .orientation(Orientation::Portrait)
            .margin(Size::Inches(1))
            .build_from_html(&content)
            .expect("failed to build pdf");

        pdfout.save(output_path).expect(format!("failed to save {}", output_path).as_str());

        Ok(())
    }
}

pub trait Answer {
    fn parse_from_json(json_path: &str) -> Result<Box<Self>>;
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
impl Choose {
    pub fn default() -> Choose {
        Choose {
            content: String::new(),
            questions: Vec::new(),
        }
    }
}
impl Answer for Choose {
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
        Ok(Box::from(choose))
    }
}


#[derive(Debug, Serialize)]
pub struct Fill {
    answer: Vec<String>, // 答案(不止一个)
}

impl Fill {
    pub fn default() -> Fill {
        Fill {
            answer: Vec::new(),
        }
    }
}
impl Answer for Fill {
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
}

#[derive(Debug, Serialize)]
pub struct Picture {
    content: String, // 听力材料
    answer: Vec<String>, // 答案(不止一个)
    keypoints: Vec<String>, //关键点(不止一个)
}

impl Picture {
    fn default() -> Picture {
        Picture {
            content: String::new(), // 听力材料
            answer: Vec::new(), // 答案(不止一个)
            keypoints: Vec::new(), //关键点(不止一个)
        }
    }
}
impl Answer for Picture {
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

        Ok(Box::from(picture))
    }
}

#[derive(Debug, Serialize)]
pub struct Read {
    content: String, //朗读内容
}

impl Read {
    fn default() -> Read {
        Read {
            content: String::new(), // 听力材料
        }
    }
}

impl Answer for Read {
    fn parse_from_json(json_path: &str) -> Result<Box<Self>> {
        let file_content = fs::read_to_string(json_path)?;
        let data: Value = serde_json::from_str(&file_content)?;

        let mut read = Read::default();
        read.content = match data["info"]["value"].as_str() {
            Some(value) => Ok(value.to_string()),
            None => Err(anyhow!("value not found")),
        }?;
        Ok(Box::from(read))
    }
}

#[derive(Debug, Serialize)]
pub struct Dialogues {
    dialogues: Vec<Dialogue>,
}
#[derive(Debug, Serialize)]
pub struct Dialogue {
    question: String, //问题
    answers: Vec<String>, //答案(不止一个)
    keywords: String, // 关键词
}

impl Dialogues {
    fn new() -> Dialogues {
        Dialogues {
            dialogues: Vec::new()
        }
    }
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

impl Answer for Dialogues {
    fn parse_from_json(json_path: &str) -> Result<Box<Self>> {
        let file_content = fs::read_to_string(json_path)?;
        let data: Value = serde_json::from_str(&file_content)?;

        let mut dialogues: Dialogues = Dialogues::new();
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
}
