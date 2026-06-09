use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PdfReport {
    pub title: String,
    pub evidence_id: String,
    pub operator: String,
    pub case_name: String,
    pub device: String,
    pub date: String,
    pub sections: Vec<ReportSection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportSection {
    pub heading: String,
    pub content: String,
}

/// Generate a forensic PDF report
pub fn generate_pdf_report(report: &PdfReport) -> Result<Vec<u8>, String> {
    use printpdf::*;

    let (doc, page_idx, layer_idx) = PdfDocument::new(
        &report.title,
        Mm(210.0), // A4 width
        Mm(297.0), // A4 height
        "Evidence Layer",
    );

    let current_layer = doc.get_page(page_idx).get_layer(layer_idx);
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| e.to_string())?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).map_err(|e| e.to_string())?;

    let mut page = current_layer;
    let mut y = Mm(275.0);
    let line_height = Mm(5.0);

    let new_page = |doc: &PdfDocumentReference, title: &str| {
        let (idx, layer) = doc.add_page(Mm(210.0), Mm(297.0), title);
        (doc.get_page(idx).get_layer(layer), Mm(275.0))
    };

    let ensure_space = |page: &mut PdfLayerReference, y: &mut Mm, needed: f32, doc: &PdfDocumentReference| {
        if y.0 < needed {
            let (new_layer, new_y) = new_page(doc, "Evidence Layer");
            *page = new_layer;
            *y = new_y;
            true
        } else {
            false
        }
    };

    let render_wrapped = |page: &PdfLayerReference, text: &str, x: Mm, mut y: Mm, size: f32, font: &IndirectFontRef| -> Mm {
        let max_chars = 88usize;
        for line in wrap_text(text, max_chars) {
            page.use_text(&line, size, x, y, font);
            y -= Mm(5.0);
        }
        y
    };

    page.use_text(&report.title, 18.0, Mm(20.0), y, &font_bold);
    y -= Mm(10.0);

    let meta = vec![
        ("Evidence ID:", &report.evidence_id),
        ("Operator:", &report.operator),
        ("Case:", &report.case_name),
        ("Device:", &report.device),
        ("Date:", &report.date),
    ];
    for (label, value) in meta {
        if ensure_space(&mut page, &mut y, 35.0, &doc) { page.use_text("Continued", 8.0, Mm(175.0), Mm(280.0), &font); }
        y = render_wrapped(&page, &format!("{} {}", label, value), Mm(20.0), y, 10.0, &font);
    }

    y -= Mm(5.0);

    for section in &report.sections {
        if ensure_space(&mut page, &mut y, 40.0, &doc) { page.use_text("Continued", 8.0, Mm(175.0), Mm(280.0), &font); }
        page.use_text(&section.heading, 12.0, Mm(20.0), y, &font_bold);
        y -= line_height;
        y = render_wrapped(&page, &section.content, Mm(20.0), y, 10.0, &font);
        y -= Mm(3.0);
    }

    // Encrypted PDF if needed
    let bytes = doc.save_to_bytes().map_err(|e| e.to_string())?;
    Ok(bytes)
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for para in text.split('\n') {
        let words: Vec<&str> = para.split_whitespace().collect();
        if words.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut line = String::new();
        for word in words {
            if line.is_empty() {
                line.push_str(word);
            } else if line.len() + 1 + word.len() <= max_chars {
                line.push(' ');
                line.push_str(word);
            } else {
                lines.push(line);
                line = word.to_string();
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }
    if lines.is_empty() { vec![String::new()] } else { lines }
}
