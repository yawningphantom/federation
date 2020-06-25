use super::*;
use crate::Name;

impl<'a> Name<'a> for Definition<'a> {
    fn name(&self) -> Option<&'a str> {
        match self {
            Definition::Schema(_) => None,
            Definition::Type(t) => t.name(),
            Definition::TypeExtension(te) => te.name(),
            Definition::Directive(d) => Some(d.name),
            Definition::Operation(o) => o.name,
            Definition::Fragment(f) => Some(f.name),
        }
    }
}
