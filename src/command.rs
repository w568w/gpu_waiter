use std::{borrow::Cow, usize};

use itertools::Itertools;

enum SegmentStatus {
    Plain(usize),
    Bracket(usize),
}

enum Segment {
    Plain(usize, usize),
    Bracket(usize, usize),
}

pub struct TemplateResult {
    pub command: String,
    pub template_count: usize,
    pub total_count: usize,
}

pub(crate) fn process_command_template(
    command_str: impl Into<Cow<'_, str>>,
    template_str: impl Into<Cow<'_, str>>,
) -> anyhow::Result<TemplateResult> {
    let template: Cow<'_, str> = template_str.into();
    let template = template.into_owned();
    let command: Cow<'_, str> = command_str.into();
    let mut result = String::with_capacity(command.len());
    
    // scan each substring with only "{" and "}"
    let mut segments = vec![];
    let mut status = None;
    for (i, c) in command.char_indices() {
        match (&status, c) {
            (None, '{') | (None, '}') => {
                status = Some(SegmentStatus::Bracket(i));
            }
            (None, _) => {
                status = Some(SegmentStatus::Plain(i));
            }
            (Some(SegmentStatus::Plain(s)), '{') | (Some(SegmentStatus::Plain(s)), '}') => {
                segments.push(Segment::Plain(*s, i));
                status = Some(SegmentStatus::Bracket(i));
            }
            (Some(SegmentStatus::Bracket(s)), _) if c != '{' && c != '}' => {
                segments.push(Segment::Bracket(*s, i));
                status = Some(SegmentStatus::Plain(i));
            }
            _ => {}
        }
    }
    match status {
        Some(SegmentStatus::Plain(s)) => {
            segments.push(Segment::Plain(s, usize::MAX));
        }
        Some(SegmentStatus::Bracket(s)) => {
            segments.push(Segment::Bracket(s, usize::MAX));
        }
        None => {}
    }

    // process each segment
    let mut template_count = 0;
    let mut total_count = 0;
    let mut command_chrs = command.chars();
    for segment in segments {
        match segment {
            Segment::Plain(start, end) => {
                command_chrs.by_ref().take(end - start).for_each(|c| result.push(c));
            }
            Segment::Bracket(start, end) => {
                let content = command_chrs.by_ref().take(end - start).collect::<String>();
                if content == "{" || content == "}" || content == "}{" {
                    result.push_str(&content);
                } else {
                    for chrs in &content.chars().chunks(2){
                        let chrs = chrs.collect::<String>();
                        match chrs.as_str() {
                            "{}" => {
                                result.push_str(&template);
                                template_count += 1;
                                total_count += 1;
                            }
                            "{{" => {
                                result.push('{');
                                template_count += 1;
                            }
                            "}}" => {
                                result.push('}');
                                template_count += 1;
                            }
                            _ => {
                                anyhow::bail!("Invalid bracket syntax in command: {}", content);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(TemplateResult {
        command: result,
        template_count,
        total_count,
    })
}
