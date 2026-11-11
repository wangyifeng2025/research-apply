//! 用系统里的中文字体生成 A4 PDF。
//!
//! 申报书要按纸质表格排版，标准 PDF 字体没有汉字。这里嵌入一份字体子集：
//! 只保留这份申报书实际用到的字形，避免把整套宋体写进文件。

use std::collections::HashMap;
use std::sync::OnceLock;

use subsetter::{GlyphRemapper, subset};
use ttf_parser::{Face, GlyphId};

const PAGE_W: f32 = 595.28;
const PAGE_H: f32 = 841.89;

#[derive(Debug)]
pub struct PdfError(pub String);

struct SharedFont {
    bytes: &'static [u8],
    index: u32,
    face: Face<'static>,
}

pub struct Document {
    font: &'static SharedFont,
    running: String,
    pages: Vec<Vec<Draw>>,
    y: f32,
    margin_l: f32,
    margin_r: f32,
    margin_t: f32,
    margin_b: f32,
    glyphs: HashMap<char, (u16, u16)>,
}

enum Draw {
    Text {
        x: f32,
        y: f32,
        size: f32,
        gray: f32,
        text: String,
    },
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        fill: Option<f32>,
        stroke: bool,
    },
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
}

#[derive(Clone, Copy)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Clone)]
pub struct Cell {
    pub text: String,
    pub span: usize,
    pub align: Align,
    pub header: bool,
    pub shade: bool,
}

pub fn cell(text: impl Into<String>) -> Cell {
    Cell {
        text: text.into(),
        span: 1,
        align: Align::Left,
        header: false,
        shade: false,
    }
}

pub fn head(text: impl Into<String>) -> Cell {
    Cell {
        text: text.into(),
        span: 1,
        align: Align::Center,
        header: true,
        shade: false,
    }
}

pub fn head_span(text: impl Into<String>, span: usize) -> Cell {
    let mut item = head(text);
    item.span = span;
    item
}

pub fn mid(text: impl Into<String>) -> Cell {
    Cell {
        align: Align::Center,
        ..cell(text)
    }
}

pub fn money_cell(text: impl Into<String>) -> Cell {
    Cell {
        align: Align::Right,
        ..cell(text)
    }
}

pub fn span_cell(text: impl Into<String>, span: usize) -> Cell {
    Cell { span, ..cell(text) }
}

pub fn span_mid(text: impl Into<String>, span: usize) -> Cell {
    Cell {
        span,
        align: Align::Center,
        ..cell(text)
    }
}

pub fn shade_cell(text: impl Into<String>, span: usize) -> Cell {
    Cell {
        text: text.into(),
        span,
        align: Align::Center,
        header: false,
        shade: true,
    }
}

impl Document {
    pub fn new(running_header: &str) -> Result<Self, PdfError> {
        let font = shared_font().ok_or_else(|| {
            PdfError(
                "服务器没有可用的中文字体，无法导出申报书。请安装宋体、冬青黑体、思源宋体或 Noto CJK，或设置 PDF_FONT"
                    .to_owned(),
            )
        })?;
        let mut document = Self {
            font,
            running: running_header.to_owned(),
            pages: Vec::new(),
            y: 0.0,
            margin_l: 46.0,
            margin_r: 40.0,
            margin_t: 42.0,
            margin_b: 40.0,
            glyphs: HashMap::new(),
        };
        document.new_page();
        Ok(document)
    }

    pub fn gap(&mut self, height: f32) {
        if self.y + height > PAGE_H - self.margin_b && self.y > self.margin_t + 24.0 {
            self.new_page();
            return;
        }
        self.y += height;
    }

    pub fn left_line(&mut self, text: &str, size: f32) {
        let line_h = size * 1.55;
        self.ensure(line_h);
        let baseline = PAGE_H - self.y - size;
        self.text_at(self.margin_l, baseline, size, text, 0.0);
        self.y += line_h;
    }

    pub fn center(&mut self, text: &str, size: f32) {
        let line_h = size * 1.65;
        self.ensure(line_h);
        let width = self.width(text, size);
        let x = ((PAGE_W - width) / 2.0).max(self.margin_l);
        let baseline = PAGE_H - self.y - size;
        self.text_at(x, baseline, size, text, 0.0);
        self.y += line_h;
    }

    pub fn heading(&mut self, text: &str) {
        self.gap(11.0);
        let size = 13.0;
        self.ensure(24.0);
        let baseline = PAGE_H - self.y - size;
        self.text_at(self.margin_l, baseline, size, text, 0.0);
        self.y += size + 4.0;
        let y = PAGE_H - self.y;
        self.line(self.margin_l, y, PAGE_W - self.margin_r, y);
        self.gap(7.0);
    }

    pub fn subhead(&mut self, text: &str) {
        self.gap(4.0);
        let size = 11.0;
        self.ensure(18.0);
        let baseline = PAGE_H - self.y - size;
        self.text_at(self.margin_l, baseline, size, text, 0.0);
        self.y += size + 5.0;
    }

    pub fn paragraph(&mut self, text: &str, size: f32) {
        self.write_lines(text, size, 0.0, self.content_width());
    }

    pub fn note(&mut self, text: &str) {
        self.write_lines(text, 8.5, 0.28, self.content_width());
    }

    pub fn field(&mut self, label: &str, value: &str) {
        let size = 12.0;
        let line_h = 22.0;
        self.ensure(line_h);
        let baseline = PAGE_H - self.y - 14.0;
        self.text_at(self.margin_l, baseline, size, label, 0.0);
        let x = self.margin_l + self.width(label, size) + 6.0;
        let right = PAGE_W - self.margin_r;
        self.text_at(x, baseline, size, value, 0.0);
        self.line(x, baseline - 3.0, right, baseline - 3.0);
        self.y += line_h;
    }

    pub fn boxed(&mut self, text: &str, min_height: f32) {
        let size = 10.0;
        let inner = (self.content_width() - 12.0).max(20.0);
        let lines = if text.trim().is_empty() {
            vec![String::new()]
        } else {
            self.wrap(text, size, inner)
        };
        let line_h = size * 1.45;
        let mut start = 0;
        let mut floor = min_height;
        while start < lines.len() {
            if PAGE_H - self.margin_b - self.y < line_h + 14.0 && self.y > self.margin_t + 24.0 {
                self.new_page();
                continue;
            }
            let available = PAGE_H - self.margin_b - self.y;
            let capacity = units((available - 12.0) / line_h).max(1);
            let take = (lines.len() - start).min(capacity).max(1);
            let mut height = 12.0 + take as f32 * line_h;
            if start + take >= lines.len() {
                height = height.max(floor);
            }
            if self.y + height > PAGE_H - self.margin_b && self.y > self.margin_t + 24.0 {
                self.new_page();
                continue;
            }
            let top = PAGE_H - self.y;
            self.rect(
                self.margin_l,
                top - height,
                self.content_width(),
                height,
                None,
                true,
            );
            for (offset, line) in lines.iter().skip(start).take(take).enumerate() {
                let baseline = top - 8.0 - size - offset as f32 * line_h;
                self.text_at(self.margin_l + 6.0, baseline, size, line, 0.0);
            }
            self.y += height;
            start += take;
            floor = 0.0;
        }
    }

    pub fn table(&mut self, fractions: &[f32], rows: Vec<Vec<Cell>>) {
        if rows.is_empty() || fractions.is_empty() {
            return;
        }
        let sum: f32 = fractions.iter().copied().sum();
        if sum <= 0.0 {
            return;
        }
        let width = self.content_width();
        let widths: Vec<f32> = fractions.iter().map(|item| item / sum * width).collect();
        let header = rows
            .first()
            .filter(|row| !row.is_empty() && row.iter().all(|item| item.header))
            .cloned();
        for (index, row) in rows.into_iter().enumerate() {
            let height = self.row_height(&widths, &row);
            let limit = PAGE_H - self.margin_b;
            if self.y + height > limit && self.y > self.margin_t + 24.0 {
                self.new_page();
                if index > 0
                    && let Some(header) = &header
                {
                    let header_height = self.row_height(&widths, header);
                    self.paint_row(&widths, header, header_height);
                }
            }
            self.paint_row(&widths, &row, height);
        }
    }

    pub fn finish(mut self) -> Result<Vec<u8>, PdfError> {
        self.add_footers();
        let mut remapper = GlyphRemapper::new();
        let mut old_ids: Vec<u16> = self.glyphs.values().map(|(gid, _)| *gid).collect();
        old_ids.sort_unstable();
        old_ids.dedup();
        for gid in old_ids {
            remapper.remap(gid);
        }
        let subset_bytes = subset(self.font.bytes, self.font.index, &remapper)
            .map_err(|error| PdfError(format!("中文字体子集生成失败：{error}")))?;
        let mut cid_of = HashMap::<char, u16>::new();
        let mut unicode_of = HashMap::<u16, char>::new();
        let upem = u32::from(self.font.face.units_per_em());
        let mut widths = vec![1000_u32; usize::from(remapper.num_gids())];
        for (ch, (old, advance)) in &self.glyphs {
            let Some(cid) = remapper.get(*old) else {
                continue;
            };
            if *old != 0 {
                cid_of.insert(*ch, cid);
                unicode_of.entry(cid).or_insert(*ch);
            }
            if let Some(slot) = widths.get_mut(usize::from(cid)) {
                *slot = u32::from(*advance) * 1000 / upem.max(1);
            }
        }
        Ok(write_pdf(
            &self.pages,
            &cid_of,
            &unicode_of,
            &widths,
            &subset_bytes,
            &self.font.face,
        ))
    }

    fn write_lines(&mut self, text: &str, size: f32, gray: f32, max_width: f32) {
        let lines = self.wrap(text, size, max_width);
        let line_h = size * 1.5;
        for line in lines {
            self.ensure(line_h);
            let baseline = PAGE_H - self.y - size;
            self.text_at(self.margin_l, baseline, size, &line, gray);
            self.y += line_h;
        }
    }

    fn row_height(&mut self, widths: &[f32], cells: &[Cell]) -> f32 {
        let mut height: f32 = 18.0;
        let mut column = 0;
        for item in cells {
            let span = item.span.max(1);
            let cell_width = widths.iter().skip(column).take(span).sum::<f32>();
            let lines = self.wrap(&item.text, 9.0, (cell_width - 8.0).max(8.0));
            let text_height = 7.0 + lines.len() as f32 * 12.2;
            height = height.max(text_height);
            column += span;
        }
        height
    }

    fn paint_row(&mut self, widths: &[f32], cells: &[Cell], height: f32) {
        let top = PAGE_H - self.y;
        let bottom = top - height;
        let mut x = self.margin_l;
        let mut column = 0;
        for item in cells {
            let span = item.span.max(1);
            let cell_width = widths.iter().skip(column).take(span).sum::<f32>();
            let fill = if item.shade {
                Some(0.9)
            } else if item.header {
                Some(0.945)
            } else {
                None
            };
            self.rect(x, bottom, cell_width, height, fill, true);
            let inner = (cell_width - 8.0).max(8.0);
            let lines = self.wrap(&item.text, 9.0, inner);
            let line_h = 12.2;
            let block = lines.len() as f32 * line_h;
            let mut cursor = top - ((height - block) / 2.0).max(3.5);
            for line in &lines {
                let text_width = self.width(line, 9.0);
                let text_x = match item.align {
                    Align::Left => x + 4.0,
                    Align::Center => x + (cell_width - text_width) / 2.0,
                    Align::Right => x + cell_width - 4.0 - text_width,
                };
                self.text_at(text_x, cursor - 9.0, 9.0, line, 0.0);
                cursor -= line_h;
            }
            x += cell_width;
            column += span;
        }
        self.y += height;
    }

    fn ensure(&mut self, height: f32) {
        if self.y + height <= PAGE_H - self.margin_b {
            return;
        }
        if self.y <= self.margin_t + 24.0 {
            return;
        }
        self.new_page();
    }

    fn new_page(&mut self) {
        self.pages.push(Vec::new());
        self.y = self.margin_t;
        if self.pages.len() > 1 {
            let text = self.running.clone();
            self.text_at(self.margin_l, PAGE_H - 28.0, 8.0, &text, 0.35);
            self.y = self.margin_t + 10.0;
        }
    }

    fn content_width(&self) -> f32 {
        PAGE_W - self.margin_l - self.margin_r
    }

    fn text_at(&mut self, x: f32, y: f32, size: f32, text: &str, gray: f32) {
        if text.is_empty() {
            return;
        }
        for ch in text.chars() {
            self.remember(ch);
        }
        if let Some(page) = self.pages.last_mut() {
            page.push(Draw::Text {
                x,
                y,
                size,
                gray,
                text: text.to_owned(),
            });
        }
    }

    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, fill: Option<f32>, stroke: bool) {
        if let Some(page) = self.pages.last_mut() {
            page.push(Draw::Rect {
                x,
                y,
                w,
                h,
                fill,
                stroke,
            });
        }
    }

    fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        if let Some(page) = self.pages.last_mut() {
            page.push(Draw::Line { x1, y1, x2, y2 });
        }
    }

    fn add_footers(&mut self) {
        let total = self.pages.len();
        let labels: Vec<(f32, String)> = (0..total)
            .map(|index| {
                let label = format!("第 {} 页 / 共 {} 页", index + 1, total);
                let width = self.width(&label, 8.5);
                ((PAGE_W - width) / 2.0, label)
            })
            .collect();
        for (page, (x, label)) in self.pages.iter_mut().zip(labels) {
            page.push(Draw::Text {
                x,
                y: 22.0,
                size: 8.5,
                gray: 0.35,
                text: label,
            });
        }
    }

    fn width(&mut self, text: &str, size: f32) -> f32 {
        let upem = f32::from(self.font.face.units_per_em().max(1));
        text.chars()
            .map(|ch| f32::from(self.remember(ch).1) * size / upem)
            .sum()
    }

    fn wrap(&mut self, text: &str, size: f32, max_width: f32) -> Vec<String> {
        let mut lines = Vec::new();
        for paragraph in text.split('\n') {
            if paragraph.is_empty() {
                lines.push(String::new());
                continue;
            }
            let mut line = String::new();
            let mut line_width = 0.0;
            let mut word = String::new();
            let mut word_width = 0.0;
            for ch in paragraph.chars() {
                let ch_width = self.char_width(ch, size);
                if ch.is_whitespace() && ch != '\u{00a0}' {
                    self.push_token(
                        &mut LineBuf {
                            line: &mut line,
                            width: &mut line_width,
                            lines: &mut lines,
                        },
                        &word,
                        word_width,
                        size,
                        max_width,
                    );
                    word.clear();
                    word_width = 0.0;
                    if line.is_empty() {
                        continue;
                    }
                    if line_width + ch_width > max_width {
                        lines.push(std::mem::take(&mut line));
                        line_width = 0.0;
                    } else {
                        line.push(ch);
                        line_width += ch_width;
                    }
                } else if breaks_anywhere(ch) {
                    self.push_token(
                        &mut LineBuf {
                            line: &mut line,
                            width: &mut line_width,
                            lines: &mut lines,
                        },
                        &word,
                        word_width,
                        size,
                        max_width,
                    );
                    word.clear();
                    word_width = 0.0;
                    push_char(
                        &mut line,
                        &mut line_width,
                        ch,
                        ch_width,
                        max_width,
                        &mut lines,
                    );
                } else {
                    word.push(ch);
                    word_width += ch_width;
                }
            }
            self.push_token(
                &mut LineBuf {
                    line: &mut line,
                    width: &mut line_width,
                    lines: &mut lines,
                },
                &word,
                word_width,
                size,
                max_width,
            );
            lines.push(line);
        }
        if lines.is_empty() {
            lines.push(String::new());
        }
        lines
    }

    fn push_token(
        &mut self,
        buf: &mut LineBuf<'_>,
        word: &str,
        word_width: f32,
        size: f32,
        max_width: f32,
    ) {
        if word.is_empty() {
            return;
        }
        if *buf.width + word_width <= max_width || (buf.line.is_empty() && word_width <= max_width)
        {
            buf.line.push_str(word);
            *buf.width += word_width;
            return;
        }
        if !buf.line.is_empty() {
            buf.lines.push(std::mem::take(buf.line));
            *buf.width = 0.0;
        }
        if word_width <= max_width {
            buf.line.push_str(word);
            *buf.width = word_width;
            return;
        }
        for ch in word.chars() {
            let ch_width = self.char_width(ch, size);
            push_char(buf.line, buf.width, ch, ch_width, max_width, buf.lines);
        }
    }

    fn char_width(&mut self, ch: char, size: f32) -> f32 {
        let upem = f32::from(self.font.face.units_per_em().max(1));
        f32::from(self.remember(ch).1) * size / upem
    }

    fn remember(&mut self, ch: char) -> (u16, u16) {
        if let Some(found) = self.glyphs.get(&ch) {
            return *found;
        }
        let gid = self
            .font
            .face
            .glyph_index(ch)
            .map(|glyph| glyph.0)
            .unwrap_or(0);
        let advance = self
            .font
            .face
            .glyph_hor_advance(GlyphId(gid))
            .unwrap_or_else(|| self.font.face.units_per_em() / 2);
        self.glyphs.insert(ch, (gid, advance));
        (gid, advance)
    }
}

fn push_char(
    line: &mut String,
    line_width: &mut f32,
    ch: char,
    ch_width: f32,
    max_width: f32,
    lines: &mut Vec<String>,
) {
    if *line_width + ch_width > max_width && !line.is_empty() {
        lines.push(std::mem::take(line));
        *line_width = 0.0;
    }
    line.push(ch);
    *line_width += ch_width;
}

fn breaks_anywhere(ch: char) -> bool {
    let code = ch as u32;
    (0x2E80..=0x9FFF).contains(&code)
        || (0xF900..=0xFAFF).contains(&code)
        || (0xFF00..=0xFFEF).contains(&code)
        || (0x20000..=0x3FFFF).contains(&code)
        || (0x3000..=0x303F).contains(&code)
}

struct LineBuf<'a> {
    line: &'a mut String,
    width: &'a mut f32,
    lines: &'a mut Vec<String>,
}

fn units(value: f32) -> usize {
    if value.is_finite() && value > 0.0 {
        let whole = value.floor();
        if whole > 10_000.0 {
            10_000
        } else {
            whole as usize
        }
    } else {
        1
    }
}

fn shared_font() -> Option<&'static SharedFont> {
    static FONT: OnceLock<Option<SharedFont>> = OnceLock::new();
    FONT.get_or_init(load_font).as_ref()
}

fn load_font() -> Option<SharedFont> {
    let (bytes, index) = find_font()?;
    let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    let face = Face::parse(leaked, index).ok()?;
    face.glyph_index('中')?;
    Some(SharedFont {
        bytes: leaked,
        index,
        face,
    })
}

fn find_font() -> Option<(Vec<u8>, u32)> {
    if let Ok(path) = std::env::var("PDF_FONT") {
        let index = std::env::var("PDF_FONT_INDEX")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        let bytes = std::fs::read(&path).ok()?;
        return Some((bytes, index));
    }
    for (path, index) in FONT_CANDIDATES {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        let Ok(face) = Face::parse(&bytes, *index) else {
            continue;
        };
        if face.glyph_index('中').is_some() {
            return Some((bytes, *index));
        }
    }
    None
}

const FONT_CANDIDATES: &[(&str, u32)] = &[
    ("/System/Library/Fonts/Supplemental/Songti.ttc", 0),
    ("/System/Library/Fonts/Hiragino Sans GB.ttc", 0),
    ("/System/Library/Fonts/Supplemental/Arial Unicode.ttf", 0),
    ("/usr/share/fonts/opentype/noto/NotoSerifCJK-Regular.ttc", 0),
    ("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 0),
    ("/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc", 0),
    ("/usr/share/fonts/truetype/wqy/wqy-microhei.ttc", 0),
    ("/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc", 0),
];

fn write_pdf(
    pages: &[Vec<Draw>],
    cid_of: &HashMap<char, u16>,
    unicode_of: &HashMap<u16, char>,
    widths: &[u32],
    font_bytes: &[u8],
    face: &Face<'_>,
) -> Vec<u8> {
    let mut writer = PdfWriter::new();
    let mut page_ids = Vec::new();
    let mut content_ids = Vec::new();
    for page in pages {
        content_ids.push(writer.stream("", &emit_page(page, cid_of).into_bytes()));
    }
    let font_file_id = writer.stream(&format!("/Length1 {}", font_bytes.len()), font_bytes);
    let descriptor_id = writer.obj(&font_descriptor(font_file_id, face));
    let cid_font_id = writer.obj(&cid_font(descriptor_id, widths));
    let to_unicode_id = writer.stream("", to_unicode(unicode_of).as_bytes());
    let type0_id = writer.obj(&format!(
        "<< /Type /Font /Subtype /Type0 /BaseFont /STSongSubset /Encoding /Identity-H /DescendantFonts [{cid_font_id} 0 R] /ToUnicode {to_unicode_id} 0 R >>"
    ));
    let pages_id = writer.reserve();
    for content_id in &content_ids {
        page_ids.push(writer.obj(&format!(
            "<< /Type /Page /Parent {pages_id} 0 R /MediaBox [0 0 {w} {h}] /Contents {content_id} 0 R /Resources << /Font << /F1 {type0_id} 0 R >> >> >>",
            w = pt(PAGE_W),
            h = pt(PAGE_H)
        )));
    }
    let kids = page_ids
        .iter()
        .map(|id| format!("{id} 0 R"))
        .collect::<Vec<_>>()
        .join(" ");
    writer.fill(
        pages_id,
        &format!(
            "<< /Type /Pages /Count {} /Kids [{kids}] >>",
            page_ids.len()
        ),
    );
    let catalog_id = writer.obj(&format!("<< /Type /Catalog /Pages {pages_id} 0 R >>"));
    writer.finish(catalog_id)
}

fn emit_page(ops: &[Draw], cid_of: &HashMap<char, u16>) -> String {
    let mut out = String::new();
    for op in ops {
        match op {
            Draw::Rect {
                x,
                y,
                w,
                h,
                fill,
                stroke,
            } => {
                if let Some(gray) = fill {
                    push_line(
                        &mut out,
                        &format!(
                            "{g} {g} {g} rg {x} {y} {w} {h} re f",
                            g = pt(*gray),
                            x = pt(*x),
                            y = pt(*y),
                            w = pt(*w),
                            h = pt(*h)
                        ),
                    );
                }
                if *stroke {
                    push_line(
                        &mut out,
                        &format!(
                            "0 0 0 RG 0.5 w {x} {y} {w} {h} re S",
                            x = pt(*x),
                            y = pt(*y),
                            w = pt(*w),
                            h = pt(*h)
                        ),
                    );
                }
            }
            Draw::Line { x1, y1, x2, y2 } => {
                push_line(
                    &mut out,
                    &format!(
                        "0 0 0 RG 0.6 w {x1} {y1} m {x2} {y2} l S",
                        x1 = pt(*x1),
                        y1 = pt(*y1),
                        x2 = pt(*x2),
                        y2 = pt(*y2)
                    ),
                );
            }
            Draw::Text {
                x,
                y,
                size,
                gray,
                text,
            } => {
                let hex = encode(text, cid_of);
                if hex.is_empty() {
                    continue;
                }
                push_line(
                    &mut out,
                    &format!(
                        "BT /F1 {size} Tf {g} {g} {g} rg 1 0 0 1 {x} {y} Tm <{hex}> Tj ET",
                        size = pt(*size),
                        g = pt(*gray),
                        x = pt(*x),
                        y = pt(*y)
                    ),
                );
            }
        }
    }
    out
}

fn encode(text: &str, cid_of: &HashMap<char, u16>) -> String {
    let mut hex = String::new();
    for ch in text.chars() {
        let Some(cid) = cid_of.get(&ch).copied() else {
            continue;
        };
        if cid == 0 {
            continue;
        }
        hex.push_str(&format!("{cid:04X}"));
    }
    hex
}

fn cid_font(descriptor_id: usize, widths: &[u32]) -> String {
    let mut width_list = String::new();
    for (cid, width) in widths.iter().enumerate() {
        if cid == 0 {
            continue;
        }
        width_list.push_str(&format!("{cid} [{width}] "));
    }
    format!(
        "<< /Type /Font /Subtype /CIDFontType2 /BaseFont /STSongSubset /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /FontDescriptor {descriptor_id} 0 R /CIDToGIDMap /Identity /DW 1000 /W [{width_list}] >>"
    )
}

fn font_descriptor(font_file_id: usize, face: &Face<'_>) -> String {
    let upem = face.units_per_em().max(1);
    let bbox = face.global_bounding_box();
    let ascent = em(face.ascender(), upem);
    let descent = em(face.descender(), upem);
    let cap = face
        .capital_height()
        .map(|value| em(value, upem))
        .unwrap_or(ascent);
    format!(
        "<< /Type /FontDescriptor /FontName /STSongSubset /Flags 4 /FontBBox [{x0} {y0} {x1} {y1}] /ItalicAngle 0 /Ascent {ascent} /Descent {descent} /CapHeight {cap} /StemV 80 /FontFile2 {font_file_id} 0 R >>",
        x0 = em(bbox.x_min, upem),
        y0 = em(bbox.y_min, upem),
        x1 = em(bbox.x_max, upem),
        y1 = em(bbox.y_max, upem)
    )
}

fn em(value: i16, upem: u16) -> i32 {
    i32::from(value) * 1000 / i32::from(upem)
}

fn to_unicode(unicode_of: &HashMap<u16, char>) -> String {
    let mut pairs: Vec<(u16, char)> = unicode_of.iter().map(|(cid, ch)| (*cid, *ch)).collect();
    pairs.sort_unstable_by_key(|(cid, _)| *cid);
    let mut body = String::from(
        "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n/CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
    );
    for chunk in pairs.chunks(100) {
        body.push_str(&format!("{} beginbfchar\n", chunk.len()));
        for (cid, ch) in chunk {
            body.push_str(&format!("<{cid:04X}> <{}>\n", utf16_hex(*ch)));
        }
        body.push_str("endbfchar\n");
    }
    body.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
    body
}

fn utf16_hex(ch: char) -> String {
    let mut units = [0_u16; 2];
    let encoded = ch.encode_utf16(&mut units);
    encoded.iter().map(|unit| format!("{unit:04X}")).collect()
}

fn pt(value: f32) -> String {
    let scaled = (value * 100.0).round() / 100.0;
    let mut text = format!("{scaled:.2}");
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    if text == "-0" || text.is_empty() {
        "0".to_owned()
    } else {
        text
    }
}

fn push_line(out: &mut String, line: &str) {
    out.push_str(line);
    out.push('\n');
}

struct PdfWriter {
    body: Vec<u8>,
    offsets: Vec<Option<usize>>,
}

impl PdfWriter {
    fn new() -> Self {
        Self {
            body: b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec(),
            offsets: vec![None],
        }
    }

    fn reserve(&mut self) -> usize {
        self.offsets.push(None);
        self.offsets.len() - 1
    }

    fn obj(&mut self, content: &str) -> usize {
        let id = self.reserve();
        self.fill(id, content);
        id
    }

    fn fill(&mut self, id: usize, content: &str) {
        self.offsets[id] = Some(self.body.len());
        self.body.extend(format!("{id} 0 obj\n").as_bytes());
        self.body.extend(content.as_bytes());
        if !content.ends_with('\n') {
            self.body.push(b'\n');
        }
        self.body.extend(b"endobj\n");
    }

    fn stream(&mut self, extra: &str, data: &[u8]) -> usize {
        let id = self.reserve();
        self.offsets[id] = Some(self.body.len());
        let header = if extra.is_empty() {
            format!("{id} 0 obj\n<< /Length {} >>\nstream\n", data.len())
        } else {
            format!("{id} 0 obj\n<< {extra} /Length {} >>\nstream\n", data.len())
        };
        self.body.extend(header.as_bytes());
        self.body.extend(data);
        self.body.extend(b"\nendstream\nendobj\n");
        id
    }

    fn finish(mut self, root: usize) -> Vec<u8> {
        let xref = self.body.len();
        let count = self.offsets.len();
        self.body.extend(format!("xref\n0 {count}\n").as_bytes());
        self.body.extend(b"0000000000 65535 f \n");
        for offset in self.offsets.iter().skip(1) {
            let position = offset.unwrap_or(0);
            self.body
                .extend(format!("{position:010} 00000 n \n").as_bytes());
        }
        self.body.extend(
            format!("trailer\n<< /Size {count} /Root {root} 0 R >>\nstartxref\n{xref}\n%%EOF\n")
                .as_bytes(),
        );
        self.body
    }
}
