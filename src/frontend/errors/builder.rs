use std::ops::Range;

use std::iter::{once, repeat_n};

use itertools::Itertools;

use crate::common::ScopeMethods;

#[derive(Clone, Debug)]
pub enum LineAttachment
{
    Highlight(char, Range<usize>, Option<String>),
}

#[derive(Clone, Debug)]
pub struct SourceLine(String, usize, Vec<Vec<LineAttachment>>);
impl SourceLine
{
    pub fn new(s: String, n: usize) -> Self
    {
        Self(s, n, vec![])
    }

    pub fn add_attachment(&mut self, attachment: LineAttachment)
    {
        for group in self.2.iter_mut()
        {
            match (attachment.clone(), group.first())
            {
                (LineAttachment::Highlight(..), Some(LineAttachment::Highlight(..))) =>
                {
                    group.push(attachment);
                    return;
                }
                _ =>
                {}
            }
        }

        self.2.push(vec![attachment])
    }
}

#[derive(Clone, Debug)]
pub struct ErrorBuilder
{
    error_type: Option<String>,
    summary: Option<String>,
    lines: Vec<SourceLine>,
}

impl ErrorBuilder
{
    pub fn new() -> Self
    {
        ErrorBuilder {
            error_type: None,
            summary: None,
            lines: vec![],
        }
    }

    pub fn with_summary(&mut self, s: String) -> &mut Self
    {
        self.summary = Some(s);
        self
    }

    pub fn with_type(&mut self, s: String) -> &mut Self
    {
        self.error_type = Some(s);
        self
    }

    pub fn with_source_line(&mut self, s: SourceLine) -> &mut Self
    {
        match self.lines.binary_search_by(|a| a.1.cmp(&s.1))
        {
            Ok(_) =>
            {}
            Err(pos) => self.lines.insert(pos, s),
        }

        self
    }

    pub fn result(self) -> String
    {
        let max_line_digits = self
            .lines
            .iter()
            .max_by(|x, y| x.1.cmp(&y.1))
            .map(|x| x.1.to_string().len())
            .unwrap_or(0);

        let mut result = String::new();

        let header = format!(
            "[{}] {}\n",
            self.error_type.unwrap_or_else(|| "Compile Error".into()),
            self.summary
                .unwrap_or_else(|| "Error in compilation".into())
        );

        result += header.as_str();

        let mut prev: Option<usize> = None;
        for line in self.lines
        {
            let line_number = line.1;
            if let Some(p) = prev
                && p < line_number - 1
            {
                result += "...\n"
            }

            result += &(Self::parse_source_line(line, max_line_digits) + "\n");
            prev = Some(line_number)
        }

        result
    }

    fn parse_source_line(source: SourceLine, max: usize) -> String
    {
        let SourceLine(raw, number, attachments) = source;

        // Add source line as first line
        let mut lines: Vec<Line> = vec![Line::new(Some(number), max)];
        lines[0].add_raw(raw);

        // Parse attachment and modify lines buffer
        for group in attachments
        {
            match group.first()
            {
                Some(LineAttachment::Highlight(..)) =>
                {
                    Self::parse_highlights(&mut lines, group, max)
                }
                _ => unreachable!(),
            }
        }

        // Add Buffer line
        lines.push(Line::new(None, max));

        // Join lines into String
        lines.into_iter().map(|x| x.data).join("\n")
    }

    fn parse_highlights(lines: &mut Vec<Line>, attachments: Vec<LineAttachment>, max: usize)
    {
        // Order the highlights in order of appearance along the line
        let ordered = attachments
            .into_iter()
            .map(|x| {
                let LineAttachment::Highlight(a, b, c) = x;
                (a, b, c)
            })
            .collect::<Vec<(char, Range<usize>, Option<String>)>>()
            .also_mut(|x| x.sort_by(|a, b| a.1.start.cmp(&b.1.start)));

        let depth = ordered.len();
        let mut buffer: Vec<Line> = repeat_n(Line::new(None, max), depth).collect();
        for (i, (chr, range, reason)) in ordered.iter().enumerate()
        {
            buffer[0].insert(
                range.start,
                repeat_n(chr, range.end - range.start).collect::<String>(),
            );
            if let Some(r) = reason
                && i < depth - 1
            {
                for (j, s) in (1..(depth - i)).zip(
                    repeat_n("|".into(), 0.max(depth as isize - i as isize - 2) as usize)
                        .chain(once(format!("⌊ {}", r))),
                )
                {
                    buffer[j].insert(range.start, s);
                }
            }
            else
            {
                buffer[0].insert(
                    range.end + 1,
                    reason.as_ref().cloned().unwrap_or_else(|| "".into()),
                );
            }
        }

        for (i, a) in buffer.into_iter().enumerate()
        {
            lines.insert(1 + i, a);
        }
    }
}

#[derive(Clone)]
struct Line
{
    data: String,
    prefix_len: usize,
}

impl Line
{
    fn new(number: Option<usize>, max_digits: usize) -> Self
    {
        let number_str = number.map(|x| x.to_string()).unwrap_or_else(|| "".into());

        Line {
            data: format!(
                " {}{} | ",
                repeat_n(' ', max_digits - number_str.len()).collect::<String>(),
                number_str
            ),
            prefix_len: max_digits + 4,
        }
    }

    fn add_raw(&mut self, s: String) -> &mut Self
    {
        self.data += s.as_str();
        self
    }

    fn add_offset(&mut self, offset: usize) -> &mut Self
    {
        self.data += &repeat_n(' ', offset).collect::<String>();
        self
    }

    fn insert(&mut self, i: usize, s: String) -> &mut Self
    {
        let upper_bound = i + s.len() + self.prefix_len;
        if self.data.len() < upper_bound
        {
            self.add_offset(upper_bound - self.data.len());
        }

        self.data
            .replace_range((i + self.prefix_len)..(i + s.len() + self.prefix_len), &s);
        self
    }
}

#[cfg(test)]
mod builder_tests
{
    use super::*;

    #[test]
    fn test()
    {
        let line = SourceLine("let tmp = 4 + 5;".into(), 4, vec![]).also_mut(|l| {
            l.add_attachment(LineAttachment::Highlight(
                '^',
                0..3,
                Some("Its really bad".into()),
            ));
            l.add_attachment(LineAttachment::Highlight(
                '-',
                4..7,
                Some("Its just awful".into()),
            ));
            l.add_attachment(LineAttachment::Highlight(
                '-',
                10..15,
                Some("How Terrible!".into()),
            ));
        });

        let second_line = SourceLine("for i in 0..10 { println(i) }".into(), 10, vec![]);

        let builder = ErrorBuilder::new().also_mut(|x| {
            x.with_source_line(line).with_source_line(second_line);
        });

        println!("{}", builder.result())
    }
}
