// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use crate::{
    allocator::TermAllocator,
    hash::{TermHash, TermHasher, TermSize},
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BooleanTerm {
    value: u32,
}
impl TermSize for BooleanTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for BooleanTerm {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher.hash(&self.value, allocator)
    }
}
impl From<bool> for BooleanTerm {
    fn from(value: bool) -> Self {
        Self {
            value: value as u32,
        }
    }
}
impl Into<bool> for BooleanTerm {
    fn into(self) -> bool {
        let Self { value, .. } = self;
        value != 0
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn boolean() {
        assert_eq!(
            TermType::Boolean(BooleanTerm::from(false)).as_bytes(),
            [TermTypeDiscriminants::Boolean as u32, 0],
        );
        assert_eq!(
            TermType::Boolean(BooleanTerm::from(true)).as_bytes(),
            [TermTypeDiscriminants::Boolean as u32, 1],
        );
    }
}
