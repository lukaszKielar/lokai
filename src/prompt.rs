use tui_textarea::TextArea;

#[derive(Debug, Clone, Default)]
pub struct Prompt<'a> {
    pub text_area: TextArea<'a>,
}

impl<'a> Prompt<'a> {
    pub fn clear(&mut self) {
        self.text_area.select_all();
        self.text_area.cut();
        self.text_area.set_yank_text("");
    }
}
