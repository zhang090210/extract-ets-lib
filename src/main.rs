use anyhow::Result;
use extract_ets_lib::{ETS, PaperType, papers::Answers, Paper};

fn main() -> Result<()> {
    // let ets = ETS::default()?;
    // let papers = ets.get_papers()?;
    // for paper in papers {
    //     // println!("{:?}", paper);
    //     // // 打印年月日
    //     // println!("Last modified: {}/{}/{}", paper.paper_time.0, paper.paper_time.1, paper.paper_time.2);
    //     let answers = paper.read_answers(PaperType::SeniorCommonPaper)?;
    //     // println!("Answers: {:#?}", answers);
    //
    //     let _ = answers.export_to_json("test.json")?;
    //     answers.export_to_html("test.html")?;
    //     break
    // }

    let paper = Paper::read_from_path(r"C:\Users\shang\AppData\Roaming\ETS\165519")?;
    let answers = paper.read_answers(PaperType::SeniorCommonPaper)?;
    answers.export_to_html("test.html")?;

    Ok(())
}