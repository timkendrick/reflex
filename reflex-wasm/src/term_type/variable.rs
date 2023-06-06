// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use crate::{
    allocator::TermAllocator,
    hash::{TermHash, TermHasher, TermSize},
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct VariableTerm {
    pub stack_offset: u32,
}
impl TermSize for VariableTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for VariableTerm {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher.hash(&self.stack_offset, allocator)
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn variable() {
        assert_eq!(
            TermType::Variable(VariableTerm {
                stack_offset: 12345,
            })
            .as_bytes(),
            [TermTypeDiscriminants::Variable as u32, 12345],
        );
    }
}
