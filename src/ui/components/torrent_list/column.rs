use ratatui::{
    layout::Alignment,
    text::{Line, Text},
};

#[derive(Default)]
pub struct Column<'a>(Text<'a>);

#[derive(Default)]
pub struct ColumnBuilder<'a> {
    top: Line<'a>,
    bottom: Line<'a>,
    divider: Line<'a>,
    alignment: Alignment,
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
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
    pub fn build(self) -> Column<'a> {
        let cell = Text::from(vec![self.top, self.bottom, self.divider]).alignment(self.alignment);
        Column(cell)
    }
}

impl<'a> From<ColumnBuilder<'a>> for Text<'a> {
    fn from(builder: ColumnBuilder<'a>) -> Self {
        builder.build().0
    }
}
