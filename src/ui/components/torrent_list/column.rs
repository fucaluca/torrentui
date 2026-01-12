use ratatui::{style::Style, text::Line, widgets::Cell};

#[derive(Default)]
pub struct Column<'a>(Cell<'a>);

#[derive(Default)]
pub struct ColumnBuilder<'a> {
    top: Line<'a>,
    bottom: Line<'a>,
    divider: Line<'a>,
}

impl<'a> Column<'a> {
    pub fn builder() -> ColumnBuilder<'a> {
        ColumnBuilder::default()
    }
}

impl<'a> ColumnBuilder<'a> {
    pub fn top(mut self, top: Line<'a>) -> Self {
        self.top = top;
        self
    }
    pub fn bottom(mut self, bottom: Line<'a>) -> Self {
        self.bottom = bottom;
        self
    }
    pub fn divider(mut self, divider: Line<'a>) -> Self {
        self.divider = divider;
        self
    }
    pub fn build(self) -> Column<'a> {
        let text = Cell::from(vec![self.top, self.bottom, self.divider]);
        Column(text)
    }
}

impl<'a> From<ColumnBuilder<'a>> for Cell<'a> {
    fn from(builder: ColumnBuilder<'a>) -> Self {
        builder.build().0
    }
}
