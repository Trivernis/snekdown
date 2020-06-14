use crate::parsing::elements::{Block, Element, Inline, Line, ListItem};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub trait FreezeVariables {
    fn freeze_variables(&mut self) -> Option<Arc<RwLock<TemplateVariable>>>;
}

pub trait GetTemplateVariables {
    fn get_template_variables(&self) -> Vec<Arc<RwLock<TemplateVariable>>>;
}

#[derive(Clone, Debug)]
pub struct Template {
    pub(crate) text: Vec<Element>,
    pub(crate) variables: HashMap<String, Arc<RwLock<TemplateVariable>>>,
}

#[derive(Clone, Debug)]
pub struct TemplateVariable {
    pub(crate) prefix: String,
    pub(crate) name: String,
    pub(crate) suffix: String,
    pub(crate) value: Option<Element>,
}

impl Template {
    pub fn render(&self, replacements: HashMap<String, Element>) -> Vec<Element> {
        replacements.iter().for_each(|(k, r)| {
            if let Some(v) = self.variables.get(k) {
                v.write().unwrap().set_value(r.clone())
            }
        });
        let elements = self
            .text
            .iter()
            .map(|e| {
                let mut e = e.clone();
                if let Some(template) = e.freeze_variables() {
                    Element::Inline(Box::new(Inline::TemplateVar(template)))
                } else {
                    e
                }
            })
            .collect();
        self.variables
            .iter()
            .for_each(|(_, v)| v.write().unwrap().reset());

        elements
    }
}

impl TemplateVariable {
    pub fn set_value(&mut self, value: Element) {
        self.value = Some(value)
    }
    pub fn reset(&mut self) {
        self.value = None
    }
}

impl GetTemplateVariables for Inline {
    fn get_template_variables(&self) -> Vec<Arc<RwLock<TemplateVariable>>> {
        match self {
            Inline::TemplateVar(temp) => vec![Arc::clone(temp)],
            Inline::Colored(col) => col.value.get_template_variables(),
            Inline::Superscript(sup) => sup.value.get_template_variables(),
            Inline::Striked(striked) => striked.value.get_template_variables(),
            Inline::Underlined(under) => under.value.get_template_variables(),
            Inline::Italic(it) => it.value.get_template_variables(),
            Inline::Bold(bo) => bo.value.get_template_variables(),
            _ => Vec::new(),
        }
    }
}

impl GetTemplateVariables for Line {
    fn get_template_variables(&self) -> Vec<Arc<RwLock<TemplateVariable>>> {
        match self {
            Line::Text(line) => line
                .subtext
                .iter()
                .map(|s| s.get_template_variables())
                .flatten()
                .collect(),
            Line::Centered(center) => center
                .line
                .subtext
                .iter()
                .map(|s| s.get_template_variables())
                .flatten()
                .collect(),
            _ => Vec::new(),
        }
    }
}

impl GetTemplateVariables for Block {
    fn get_template_variables(&self) -> Vec<Arc<RwLock<TemplateVariable>>> {
        match self {
            Block::Section(sec) => sec
                .elements
                .iter()
                .map(|e| e.get_template_variables())
                .flatten()
                .collect(),
            Block::Paragraph(par) => par
                .elements
                .iter()
                .map(|l| l.get_template_variables())
                .flatten()
                .collect(),
            Block::Quote(q) => q
                .text
                .iter()
                .map(|t| {
                    t.subtext
                        .iter()
                        .map(|i| i.get_template_variables())
                        .flatten()
                })
                .flatten()
                .collect(),
            Block::List(list) => list
                .items
                .iter()
                .map(|item| item.get_template_variables())
                .flatten()
                .collect(),
            _ => Vec::new(),
        }
    }
}

impl GetTemplateVariables for Element {
    fn get_template_variables(&self) -> Vec<Arc<RwLock<TemplateVariable>>> {
        match self {
            Element::Block(block) => block.get_template_variables(),
            Element::Inline(inline) => inline.get_template_variables(),
            Element::Line(line) => line.get_template_variables(),
        }
    }
}

impl FreezeVariables for Inline {
    fn freeze_variables(&mut self) -> Option<Arc<RwLock<TemplateVariable>>> {
        match self {
            Inline::TemplateVar(temp) => {
                let temp = temp.read().unwrap();
                return Some(Arc::new(RwLock::new((*temp).clone())));
            }
            Inline::Colored(col) => {
                if let Some(temp) = col.value.freeze_variables() {
                    col.value = Box::new(Inline::TemplateVar(temp))
                }
            }
            Inline::Superscript(sup) => {
                if let Some(temp) = sup.value.freeze_variables() {
                    sup.value = Box::new(Inline::TemplateVar(temp))
                }
            }
            Inline::Striked(striked) => {
                if let Some(temp) = striked.value.freeze_variables() {
                    striked.value = Box::new(Inline::TemplateVar(temp))
                }
            }
            Inline::Underlined(under) => {
                if let Some(temp) = under.value.freeze_variables() {
                    under.value = Box::new(Inline::TemplateVar(temp))
                }
            }
            Inline::Italic(it) => {
                if let Some(temp) = it.value.freeze_variables() {
                    it.value = Box::new(Inline::TemplateVar(temp))
                }
            }
            Inline::Bold(bo) => {
                if let Some(temp) = bo.value.freeze_variables() {
                    bo.value = Box::new(Inline::TemplateVar(temp))
                }
            }
            _ => {}
        }
        None
    }
}

impl GetTemplateVariables for ListItem {
    fn get_template_variables(&self) -> Vec<Arc<RwLock<TemplateVariable>>> {
        let mut inner_vars: Vec<Arc<RwLock<TemplateVariable>>> = self
            .children
            .iter()
            .map(|child| child.get_template_variables())
            .flatten()
            .collect();
        inner_vars.append(&mut self.text.get_template_variables());

        inner_vars
    }
}

impl FreezeVariables for Line {
    fn freeze_variables(&mut self) -> Option<Arc<RwLock<TemplateVariable>>> {
        match self {
            Line::Text(text) => {
                text.subtext = text
                    .subtext
                    .iter_mut()
                    .map(|i| {
                        if let Some(t) = i.freeze_variables() {
                            Inline::TemplateVar(t)
                        } else {
                            (*i).clone()
                        }
                    })
                    .collect()
            }
            Line::Centered(center) => {
                center.line.subtext = center
                    .line
                    .subtext
                    .iter_mut()
                    .map(|i| {
                        if let Some(t) = i.freeze_variables() {
                            Inline::TemplateVar(t)
                        } else {
                            (*i).clone()
                        }
                    })
                    .collect()
            }
            _ => {}
        }
        None
    }
}

impl FreezeVariables for Block {
    fn freeze_variables(&mut self) -> Option<Arc<RwLock<TemplateVariable>>> {
        match self {
            Block::Section(s) => s.elements.iter_mut().for_each(|b| {
                b.freeze_variables();
            }),
            Block::Paragraph(p) => p.elements.iter_mut().for_each(|l| {
                l.freeze_variables();
            }),
            Block::Quote(q) => q.text.iter_mut().for_each(|t| {
                t.subtext = t
                    .subtext
                    .iter_mut()
                    .map(|i| {
                        if let Some(t) = i.freeze_variables() {
                            Inline::TemplateVar(t)
                        } else {
                            (*i).clone()
                        }
                    })
                    .collect()
            }),
            Block::List(list) => list.items.iter_mut().for_each(|item| {
                item.freeze_variables();
            }),
            _ => {}
        };

        None
    }
}

impl FreezeVariables for Element {
    fn freeze_variables(&mut self) -> Option<Arc<RwLock<TemplateVariable>>> {
        match self {
            Element::Block(b) => b.freeze_variables(),
            Element::Line(l) => l.freeze_variables(),
            Element::Inline(i) => return i.freeze_variables(),
        };

        None
    }
}

impl FreezeVariables for ListItem {
    fn freeze_variables(&mut self) -> Option<Arc<RwLock<TemplateVariable>>> {
        self.children.iter_mut().for_each(|child| {
            child.freeze_variables();
        });
        self.text.freeze_variables();
        None
    }
}
