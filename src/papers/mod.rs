pub(crate) mod senior;
use anyhow::Result;
use tera::Tera;
use lazy_static::lazy_static;

lazy_static!{
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::new("templates/**/*.html").unwrap();
        tera.autoescape_on(vec![".html"]);
        tera
    };
}

pub trait Answers {
    fn default() -> Self; // 初始化结构体
    fn read_answers(&mut self, paper_path: &str) -> Result<()>; // 从文件读取答案
    fn export_to_html(&self, output_path: &str) -> Result<String>; // 导出为html文件
    fn export_to_pdf(&self, output_path: &str) -> Result<()>; // 导出为pdf文件
    fn export_to_json(&self, output_path: &str) -> Result<String>; // 导出为json文件
}

pub trait Answer {
    fn default() -> Self; // 初始化结构体
    fn parse_from_json(json_path: &str) -> Result<Box<Self>>; // 从json文件中解析
    fn clean_data(&mut self) -> Result<()>; // 清理数据
}
