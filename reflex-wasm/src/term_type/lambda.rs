// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use crate::{
    allocator::TermAllocator,
    hash::{TermHash, TermHasher, TermSize},
    TermPointer,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct LambdaTerm {
    pub num_args: u32,
    pub body: TermPointer,
}
impl TermSize for LambdaTerm {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for LambdaTerm {
    fn hash(&self, hasher: TermHasher, allocator: &impl TermAllocator) -> TermHasher {
        hasher
            .hash(&self.num_args, allocator)
            .hash(&self.body, allocator)
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn lambda() {
        assert_eq!(
            TermType::Lambda(LambdaTerm {
                num_args: 12345,
                body: TermPointer(67890),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Lambda as u32, 12345, 67890],
        );
    }
}
