use figlet_rs::FIGfont;

pub fn load_embedded_figlet_fonts() -> Result<(FIGfont, FIGfont), String> {
    // Embed the font contents directly into the binary
    let font_time = include_str!("../fonts/font_time.flf");
    let font_date = include_str!("../fonts/font_date.flf");

    let figfont_time =
        FIGfont::from_content(font_time).map_err(|e| format!("Failed to load font_time: {}", e))?;
    let figfont_date =
        FIGfont::from_content(font_date).map_err(|e| format!("Failed to load font_date: {}", e))?;

    Ok((figfont_time, figfont_date))
}

pub fn render_figlet_text<'a>(font: &'a FIGfont, text: &'a str) -> figlet_rs::FIGure<'a> {
    font.convert(text)
        .unwrap_or_else(|| font.convert("ERR").unwrap())
}
