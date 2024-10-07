use anyhow::Result;
use wkhtmltopdf::PdfApplication;
use extract_ets_lib::ETS;

fn main() -> Result<()> {
    let mut pdf_app = PdfApplication::new()?;
    // println!("Hello, world!");
    let ets = ETS::default()?;
    let papers = ets.get_papers()?;
    for paper in papers {
        // println!("{:?}", paper);
        // // 打印年月日
        // println!("Last modified: {}/{}/{}", paper.paper_time.0, paper.paper_time.1, paper.paper_time.2);
        let answers = paper.read_answers()?;
        // println!("Answers: {:#?}", answers);
        // answers.export_to_html("answers.html")?;
        answers.export_pdf(&pdf_app, "answer.pdf")?;
        answers.export_pdf(&pdf_app, "test.pdf")?
    }

    Ok(())
}